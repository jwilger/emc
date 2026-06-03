use std::fs;
use std::io::{self, Read};

use serde_json::{Value, json};

use crate::core::effect::{Effect, EffectPlan};
use crate::core::layout::list_workflows;
use crate::io::dto::parse_browser_index_workflows;
use crate::shell::ShellError;

pub fn serve_stdio() -> Result<(), ShellError> {
    let mut input = String::new();
    io::stdin()
        .read_to_string(&mut input)
        .map_err(|error| ShellError::message(error.to_string()))?;

    input
        .lines()
        .map(handle_input_line)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .flatten()
        .try_for_each(write_response)
}

fn handle_input_line(line: &str) -> Result<Option<Value>, ShellError> {
    let request = serde_json::from_str::<Value>(line)
        .map_err(|error| ShellError::message(format!("invalid MCP JSON-RPC message: {error}")))?;
    handle_request(&request)
}

fn handle_request(request: &Value) -> Result<Option<Value>, ShellError> {
    let Some(id) = request.get("id") else {
        return Ok(None);
    };

    match request.get("method").and_then(Value::as_str) {
        Some("initialize") => Ok(Some(success_response(id, initialize_result()))),
        Some("tools/list") => Ok(Some(success_response(id, tools_list_result()))),
        Some("tools/call") => tool_call_response(id, request),
        Some(method) => Ok(Some(error_response(
            id,
            -32601,
            format!("unknown MCP method {method}"),
        ))),
        None => Ok(Some(error_response(
            id,
            -32600,
            "MCP request is missing method",
        ))),
    }
}

fn initialize_result() -> Value {
    json!({
        "protocolVersion": "2025-11-25",
        "capabilities": {
            "tools": {}
        },
        "serverInfo": {
            "name": "emc",
            "version": env!("CARGO_PKG_VERSION")
        }
    })
}

fn tools_list_result() -> Value {
    json!({
        "tools": [
            {
                "name": "list_workflows",
                "description": "List imported workflows in the EMC event model.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            }
        ]
    })
}

fn tool_call_response(id: &Value, request: &Value) -> Result<Option<Value>, ShellError> {
    let Some(name) = request
        .get("params")
        .and_then(|params| params.get("name"))
        .and_then(Value::as_str)
    else {
        return Ok(Some(error_response(
            id,
            -32602,
            "MCP tool call is missing tool name",
        )));
    };

    match name {
        "list_workflows" => Ok(Some(success_response(
            id,
            tool_result(list_workflows_tool_text()?),
        ))),
        _ => Ok(Some(error_response(
            id,
            -32602,
            format!("unknown EMC MCP tool {name}"),
        ))),
    }
}

fn list_workflows_tool_text() -> Result<String, ShellError> {
    let index = fs::read_to_string("model/browser/data/index.json")
        .map_err(|error| ShellError::message(error.to_string()))?;
    let imported_workflows = parse_browser_index_workflows(&index)
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(report_text(list_workflows(imported_workflows)))
}

fn report_text(plan: EffectPlan) -> String {
    plan.effects()
        .iter()
        .filter_map(|effect| match effect {
            Effect::Report(line) => Some(line.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn tool_result(text: String) -> Value {
    json!({
        "content": [
            {
                "type": "text",
                "text": text
            }
        ],
        "isError": false
    })
}

fn success_response(id: &Value, result: Value) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "result": result
    })
}

fn error_response(id: &Value, code: i64, message: impl Into<String>) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": {
            "code": code,
            "message": message.into()
        }
    })
}

fn write_response(response: Value) -> Result<(), ShellError> {
    let line =
        serde_json::to_string(&response).map_err(|error| ShellError::message(error.to_string()))?;
    println!("{line}");
    Ok(())
}
