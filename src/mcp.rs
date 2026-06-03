use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use serde_json::{Value, json};

use crate::core::connection::WorkflowConnection;
use crate::core::effect::{Effect, EffectPlan, ProjectPath};
use crate::core::review_gate::review_gate;
use crate::core::slice::NewSlice;
use crate::core::workflow::NewWorkflow;
use crate::io::dto::{
    parse_connection_kind, parse_model_description, parse_model_name, parse_slice_kind,
    parse_slice_slug, parse_transition_trigger_name, parse_workflow_slug,
};
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

pub fn serve_http(
    host: &str,
    port: u16,
    once: bool,
    auth_token: Option<&str>,
) -> Result<(), ShellError> {
    let auth_policy = auth_policy(host, auth_token)?;
    if auth_policy.is_required() && auth_token.is_none() {
        return Err(ShellError::message(
            "MCP HTTP non-local bind requires --auth-token",
        ));
    }

    let listener = TcpListener::bind(format!("{host}:{port}")).map_err(|error| {
        ShellError::message(format!("failed to bind MCP HTTP listener: {error}"))
    })?;
    let authority = listener
        .local_addr()
        .map_err(|error| ShellError::message(error.to_string()))?
        .to_string();
    println!("MCP HTTP listening on {authority}");

    if once {
        let (stream, _address) = listener
            .accept()
            .map_err(|error| ShellError::message(error.to_string()))?;
        handle_http_stream(stream, &authority, &auth_policy)
    } else {
        listener.incoming().try_for_each(|stream| {
            stream
                .map_err(|error| ShellError::message(error.to_string()))
                .and_then(|stream| handle_http_stream(stream, &authority, &auth_policy))
        })
    }
}

fn auth_policy<'token>(
    host: &str,
    auth_token: Option<&'token str>,
) -> Result<AuthPolicy<'token>, ShellError> {
    if is_localhost_bind(host) {
        Ok(AuthPolicy::Optional(auth_token))
    } else {
        auth_token
            .map(AuthPolicy::Required)
            .ok_or_else(|| ShellError::message("MCP HTTP non-local bind requires --auth-token"))
    }
}

fn is_localhost_bind(host: &str) -> bool {
    matches!(host, "127.0.0.1" | "localhost" | "::1")
}

#[derive(Clone, Copy)]
enum AuthPolicy<'token> {
    Optional(Option<&'token str>),
    Required(&'token str),
}

impl AuthPolicy<'_> {
    fn is_required(&self) -> bool {
        matches!(self, Self::Required(_token))
    }
}

fn handle_http_stream(
    stream: TcpStream,
    authority: &str,
    auth_policy: &AuthPolicy<'_>,
) -> Result<(), ShellError> {
    let mut reader = BufReader::new(stream);
    let request = read_http_request(&mut reader)?;
    let response = http_response_for_request(request, authority, auth_policy)?;
    let mut stream = reader.into_inner();
    stream
        .write_all(response.as_bytes())
        .map_err(|error| ShellError::message(error.to_string()))
}

struct HttpRequest {
    method: String,
    path: String,
    origin: Option<String>,
    authorization: Option<String>,
    body: String,
}

fn read_http_request(reader: &mut BufReader<TcpStream>) -> Result<HttpRequest, ShellError> {
    let mut request_line = String::new();
    reader
        .read_line(&mut request_line)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let mut request_parts = request_line.split_whitespace();
    let method = request_parts
        .next()
        .ok_or_else(|| ShellError::message("missing HTTP method"))?
        .to_owned();
    let path = request_parts
        .next()
        .ok_or_else(|| ShellError::message("missing HTTP path"))?
        .to_owned();

    let mut content_length = 0_usize;
    let mut origin = None;
    let mut authorization = None;
    loop {
        let mut header = String::new();
        reader
            .read_line(&mut header)
            .map_err(|error| ShellError::message(error.to_string()))?;
        if header == "\r\n" || header.is_empty() {
            break;
        }
        if let Some((name, value)) = header.split_once(':') {
            let normalized_name = name.trim().to_ascii_lowercase();
            let normalized_value = value.trim().to_owned();
            if normalized_name == "content-length" {
                content_length = normalized_value
                    .parse::<usize>()
                    .map_err(|error| ShellError::message(error.to_string()))?;
            } else if normalized_name == "origin" {
                origin = Some(normalized_value);
            } else if normalized_name == "authorization" {
                authorization = Some(normalized_value);
            }
        }
    }

    let mut body_bytes = vec![0_u8; content_length];
    reader
        .read_exact(&mut body_bytes)
        .map_err(|error| ShellError::message(error.to_string()))?;
    let body =
        String::from_utf8(body_bytes).map_err(|error| ShellError::message(error.to_string()))?;

    Ok(HttpRequest {
        method,
        path,
        origin,
        authorization,
        body,
    })
}

fn http_response_for_request(
    request: HttpRequest,
    authority: &str,
    auth_policy: &AuthPolicy<'_>,
) -> Result<String, ShellError> {
    if request.method != "POST" || request.path != "/mcp" {
        return Ok(http_response("404 Not Found", "{\"error\":\"not found\"}"));
    }
    if !origin_is_allowed(request.origin.as_deref(), authority) {
        return Ok(http_response(
            "403 Forbidden",
            "{\"error\":\"forbidden origin\"}",
        ));
    }
    if !authorization_is_allowed(request.authorization.as_deref(), auth_policy) {
        return Ok(http_response(
            "401 Unauthorized",
            "{\"error\":\"missing or invalid bearer token\"}",
        ));
    }

    let mcp_request = serde_json::from_str::<Value>(&request.body)
        .map_err(|error| ShellError::message(format!("invalid MCP HTTP JSON-RPC body: {error}")))?;
    let response = handle_request(&mcp_request)?
        .map(|response| serde_json::to_string(&response))
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?
        .unwrap_or_else(|| "{}".to_owned());
    Ok(http_response("200 OK", &response))
}

fn authorization_is_allowed(authorization: Option<&str>, auth_policy: &AuthPolicy<'_>) -> bool {
    match auth_policy {
        AuthPolicy::Optional(None) => true,
        AuthPolicy::Optional(Some(token)) | AuthPolicy::Required(token) => {
            authorization == Some(format!("Bearer {token}").as_str())
        }
    }
}

fn origin_is_allowed(origin: Option<&str>, authority: &str) -> bool {
    origin.is_none_or(|origin| {
        origin == format!("http://{authority}") || origin == format!("https://{authority}")
    })
}

fn http_response(status: &str, body: &str) -> String {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
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
                "description": "List modeled workflows in the EMC event model.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            },
            {
                "name": "show_workflow",
                "description": "Show a modeled workflow document by workflow slug.",
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
            },
            {
                "name": "verify_project",
                "description": "Run Lean4 and Quint verification for generated model artifacts.",
                "inputSchema": {
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
                }
            },
            {
                "name": "validate_event_model",
                "description": "Validate event model workflow or slice files.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "target": {
                            "type": "string"
                        }
                    },
                    "required": ["target"],
                    "additionalProperties": false
                }
            },
            {
                "name": "review_gate",
                "description": "Evaluate whether a workflow has a current clean structured review.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow"],
                    "additionalProperties": false
                }
            },
            {
                "name": "add_workflow",
                "description": "Add a business workflow and regenerate synchronized model artifacts.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "slug": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "description": {
                            "type": "string"
                        }
                    },
                    "required": ["slug", "name", "description"],
                    "additionalProperties": false
                }
            },
            {
                "name": "add_slice",
                "description": "Add a business slice to a workflow composition.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        },
                        "slug": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "type": {
                            "type": "string"
                        },
                        "description": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow", "slug", "name", "type", "description"],
                    "additionalProperties": false
                }
            },
            {
                "name": "update_workflow",
                "description": "Update a business workflow and regenerate synchronized model artifacts.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "slug": {
                            "type": "string"
                        },
                        "description": {
                            "type": "string"
                        }
                    },
                    "required": ["slug", "description"],
                    "additionalProperties": false
                }
            },
            {
                "name": "connect_workflow",
                "description": "Connect workflow steps with a transition and regenerate synchronized model artifacts.",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        },
                        "from": {
                            "type": "string"
                        },
                        "to": {
                            "type": "string"
                        },
                        "via": {
                            "type": "string",
                            "enum": ["command", "event", "navigation"]
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow", "from", "to", "via", "name"],
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
        "verify_project" => Ok(Some(success_response(
            id,
            tool_result(verify_project_tool_text()?),
        ))),
        "validate_event_model" => Ok(Some(success_response(
            id,
            tool_result(validate_event_model_tool_text(request)?),
        ))),
        "review_gate" => Ok(Some(success_response(
            id,
            tool_result(review_gate_tool_text(request)?),
        ))),
        "add_workflow" => Ok(Some(success_response(
            id,
            tool_result(add_workflow_tool_text(request)?),
        ))),
        "add_slice" => Ok(Some(success_response(
            id,
            tool_result(add_slice_tool_text(request)?),
        ))),
        "update_workflow" => Ok(Some(success_response(
            id,
            tool_result(update_workflow_tool_text(request)?),
        ))),
        "connect_workflow" => Ok(Some(success_response(
            id,
            tool_result(connect_workflow_tool_text(request)?),
        ))),
        _ => Ok(Some(error_response(
            id,
            -32602,
            format!("unknown EMC MCP tool {name}"),
        ))),
    }
}

fn list_workflows_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(EffectPlan::new(vec![Effect::ListWorkflowsFromIndex]))
        .map(|reports| reports.join("\n"))
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
    interpret_collect_reports(EffectPlan::new(vec![Effect::ShowWorkflowFromWorkflow(
        slug,
    )]))
    .map(|reports| reports.join("\n"))
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
    interpret_collect_reports(EffectPlan::new(vec![Effect::GenerateSiteFromManifest(
        output,
    )]))
    .map(|reports| reports.join("\n"))
}

fn verify_project_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(EffectPlan::new(vec![Effect::VerifyProjectFromIndex]))
        .map(|reports| reports.join("\n"))
}

fn validate_event_model_tool_text(request: &Value) -> Result<String, ShellError> {
    let target = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .and_then(|arguments| arguments.get("target"))
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("validate_event_model requires target"))
        .and_then(|target| {
            ProjectPath::try_new(target.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(EffectPlan::new(vec![Effect::ValidateEventModelTarget(
        target,
    )]))
    .map(|reports| reports.join("\n"))
}

fn review_gate_tool_text(request: &Value) -> Result<String, ShellError> {
    let workflow_slug = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .and_then(|arguments| arguments.get("workflow"))
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("review_gate requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(review_gate(workflow_slug)).map(|reports| reports.join("\n"))
}

fn add_workflow_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_workflow requires arguments"))?;
    let slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow requires slug"))
        .and_then(|raw_slug| {
            parse_workflow_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow requires name"))
        .and_then(|raw_name| {
            parse_model_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let description = arguments
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow requires description"))
        .and_then(|raw_description| {
            parse_model_description(raw_description)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(EffectPlan::new(vec![Effect::AddWorkflowFromIndex(
        NewWorkflow::new(name, description, slug),
    )]))
    .map(|reports| reports.join("\n"))
}

fn add_slice_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_slice requires arguments"))?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let slice_slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice requires slug"))
        .and_then(|raw_slug| {
            parse_slice_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let slice_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice requires name"))
        .and_then(|raw_name| {
            parse_model_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let slice_kind = arguments
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice requires type"))
        .and_then(|raw_type| {
            parse_slice_kind(raw_type).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let slice_description = arguments
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice requires description"))
        .and_then(|raw_description| {
            parse_model_description(raw_description)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(EffectPlan::new(vec![Effect::AddSliceFromWorkflow(
        NewSlice::new(
            workflow_slug,
            slice_slug,
            slice_name,
            slice_description,
            slice_kind,
        ),
    )]))
    .map(|reports| reports.join("\n"))
}

fn update_workflow_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_workflow requires arguments"))?;
    let slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_workflow requires slug"))
        .and_then(|raw_slug| {
            parse_workflow_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let description = arguments
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_workflow requires description"))
        .and_then(|raw_description| {
            parse_model_description(raw_description)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(EffectPlan::new(vec![
        Effect::UpdateWorkflowDescriptionFromIndexAndWorkflow(slug, description),
    ]))
    .map(|reports| reports.join("\n"))
}

fn connect_workflow_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("connect_workflow requires arguments"))?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("connect_workflow requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_slug = arguments
        .get("from")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("connect_workflow requires from"))
        .and_then(|raw_source| {
            parse_slice_slug(raw_source).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let target_slug = arguments
        .get("to")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("connect_workflow requires to"))
        .and_then(|raw_target| {
            parse_slice_slug(raw_target).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let connection_kind = arguments
        .get("via")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("connect_workflow requires via"))
        .and_then(|raw_via| {
            parse_connection_kind(raw_via).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let trigger = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("connect_workflow requires name"))
        .and_then(|raw_name| {
            parse_transition_trigger_name(raw_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(EffectPlan::new(vec![Effect::ConnectWorkflowFromWorkflow(
        WorkflowConnection::new(
            workflow_slug,
            source_slug,
            target_slug,
            connection_kind,
            trigger,
        ),
    )]))
    .map(|reports| reports.join("\n"))
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
