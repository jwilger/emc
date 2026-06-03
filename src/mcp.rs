use std::fs;
use std::io::{self, Read};

use serde_json::{Value, json};

use crate::core::effect::{Effect, EffectPlan, FileContents, ProjectPath};
use crate::core::layout::{list_workflows, show_workflow};
use crate::core::site::generate_site;
use crate::io::dto::{parse_browser_index_workflows, parse_workflow_slug};
use crate::shell::{ShellError, interpret_collect_reports};

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
            },
            {
                "name": "show_workflow",
                "description": "Show an imported workflow document by workflow slug.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "slug": {
                            "type": "string"
                        }
                    },
                    "required": ["slug"],
                    "additionalProperties": false
                }
            },
            {
                "name": "generate_site",
                "description": "Generate the human-browsable event model site.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "output": {
                            "type": "string"
                        }
                    },
                    "required": ["output"],
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
        "show_workflow" => Ok(Some(success_response(
            id,
            tool_result(show_workflow_tool_text(request)?),
        ))),
        "generate_site" => Ok(Some(success_response(
            id,
            tool_result(generate_site_tool_text(request)?),
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

fn show_workflow_tool_text(request: &Value) -> Result<String, ShellError> {
    let raw_slug = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .and_then(|arguments| arguments.get("slug"))
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("show_workflow requires slug"))?;
    let slug =
        parse_workflow_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))?;
    let workflow_path = format!(
        "model/browser/data/workflows/{}.eventmodel.json",
        slug.as_ref()
    );
    let workflow_document = fs::read_to_string(workflow_path)
        .map_err(|error| ShellError::message(error.to_string()))
        .and_then(|contents| {
            FileContents::try_new(contents).map_err(|error| ShellError::message(error.to_string()))
        })?;
    Ok(document_text(show_workflow(workflow_document)))
}

fn generate_site_tool_text(request: &Value) -> Result<String, ShellError> {
    let output = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .and_then(|arguments| arguments.get("output"))
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("generate_site requires output"))
        .and_then(|output| {
            ProjectPath::try_new(output.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(generate_site(output)).map(|reports| reports.join("\n"))
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

fn document_text(plan: EffectPlan) -> String {
    plan.effects()
        .iter()
        .filter_map(|effect| match effect {
            Effect::ReportDocument(contents) => Some(contents.as_ref()),
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
