use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use rmcp::model::{
    CallToolResult, Content, Implementation, InitializeResult, JsonObject, ListToolsResult,
    ServerCapabilities, Tool, ToolsCapability,
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::command;
use crate::core::connection::WorkflowConnection;
use crate::core::slice::NewSlice;
use crate::core::workflow::NewWorkflow;
use crate::io::dto::{
    parse_connection_kind, parse_model_description, parse_model_name, parse_project_name,
    parse_project_path, parse_slice_kind, parse_slice_slug, parse_transition_trigger_name,
    parse_workflow_slug,
};
use crate::shell::{ShellError, interpret_collect_reports};

pub fn serve_stdio() -> Result<(), ShellError> {
    let stdin = io::stdin();
    stdin.lock().lines().try_for_each(|line| {
        line.map_err(|error| ShellError::message(error.to_string()))
            .and_then(|line| handle_input_line(&line))
            .and_then(|response| response.map_or(Ok(()), write_response))
    })
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
    host: Option<String>,
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
    let mut host = None;
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
            } else if normalized_name == "host" {
                host = Some(normalized_value);
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
        host,
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
    if request.path != "/mcp" {
        return Ok(http_response("404 Not Found", "{\"error\":\"not found\"}"));
    }
    let request_authority = request.host.as_deref().unwrap_or(authority);
    if !origin_is_allowed(request.origin.as_deref(), request_authority) {
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
    if request.method == "GET" {
        return Ok(http_response(
            "405 Method Not Allowed",
            "{\"error\":\"server-sent event streaming is not available\"}",
        ));
    }
    if request.method != "POST" {
        return Ok(http_response(
            "405 Method Not Allowed",
            "{\"error\":\"method not allowed\"}",
        ));
    }

    let mcp_request = match serde_json::from_str::<Value>(&request.body) {
        Ok(request) => request,
        Err(error) => {
            return Ok(http_response(
                "400 Bad Request",
                &json!({
                    "error": format!("invalid MCP HTTP JSON-RPC body: {error}")
                })
                .to_string(),
            ));
        }
    };
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
        Some("initialize") => Ok(Some(success_response(id, initialize_result()?))),
        Some("tools/list") => Ok(Some(success_response(id, tools_list_result()?))),
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

fn initialize_result() -> Result<Value, ShellError> {
    let mut capabilities = ServerCapabilities::default();
    capabilities.tools = Some(ToolsCapability::default());
    mcp_model_value(
        InitializeResult::new(capabilities)
            .with_server_info(Implementation::new("emc", env!("CARGO_PKG_VERSION"))),
    )
}

fn tools_list_result() -> Result<Value, ShellError> {
    mcp_model_value(ListToolsResult::with_all_items(vec![
        Tool::new(
            "init_project",
            "Initialize an EMC project layout and root formal artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string"
                        }
                    },
                    "required": ["name"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "list_workflows",
            "List modeled workflows in the EMC event model.",
            schema_object(json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "list_slices",
            "List modeled slices in the EMC event model.",
            schema_object(json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "list_transitions",
            "List modeled workflow transitions in the EMC event model.",
            schema_object(json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "show_workflow",
            "Show a modeled workflow document by workflow slug.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slug": {
                            "type": "string"
                        }
                    },
                    "required": ["slug"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "show_slice",
            "Show a modeled slice document by slice slug.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slug": {
                            "type": "string"
                        }
                    },
                    "required": ["slug"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "generate_site",
            "Generate the human-browsable event model site.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "output": {
                            "type": "string"
                        }
                    },
                    "required": ["output"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "check_project",
            "Check required project artifacts and generated model synchronization.",
            schema_object(json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "verify_project",
            "Run Lean4 and Quint verification for generated model artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "validate_event_model",
            "Validate event model workflow or slice files.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "target": {
                            "type": "string"
                        }
                    },
                    "required": ["target"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "review_gate",
            "Evaluate whether a workflow has a current clean structured review.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_workflow",
            "Add a business workflow and regenerate synchronized model artifacts.",
            schema_object(json!({
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
            })),
        ),
        Tool::new(
            "add_slice",
            "Add a business slice to a workflow composition.",
            schema_object(json!({
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
            })),
        ),
        Tool::new(
            "update_workflow",
            "Update a business workflow and regenerate synchronized model artifacts.",
            schema_object(json!({
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
            })),
        ),
        Tool::new(
            "update_workflow_name",
            "Update a business workflow name and regenerate synchronized model artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slug": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "required": ["slug", "name"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "update_slice",
            "Update a business slice and regenerate synchronized model artifacts.",
            schema_object(json!({
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
            })),
        ),
        Tool::new(
            "update_slice_kind",
            "Update a business slice kind and regenerate synchronized model artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slug": {
                            "type": "string"
                        },
                        "type": {
                            "type": "string",
                            "enum": ["state_view", "state_change", "translation", "automation"]
                        }
                    },
                    "required": ["slug", "type"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "update_slice_name",
            "Update a business slice name and regenerate synchronized model artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slug": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        }
                    },
                    "required": ["slug", "name"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "connect_workflow",
            "Connect workflow steps with a transition and regenerate synchronized model artifacts.",
            schema_object(json!({
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
                        "to_workflow": {
                            "type": "string"
                        },
                        "via": {
                            "type": "string",
                            "enum": ["command", "event", "navigation", "external_trigger", "outcome"]
                        },
                        "name": {
                            "type": "string"
                        },
                        "reason": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow", "from", "via", "name"],
                    "oneOf": [
                        {"required": ["to"]},
                        {"required": ["to_workflow", "reason"]}
                    ],
                    "additionalProperties": false
            })),
        ),
    ]))
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
        "init_project" => Ok(Some(tool_call_result_response(
            id,
            init_project_tool_text(request),
        ))),
        "list_workflows" => Ok(Some(tool_call_result_response(
            id,
            list_workflows_tool_text(),
        ))),
        "list_slices" => Ok(Some(tool_call_result_response(id, list_slices_tool_text()))),
        "list_transitions" => Ok(Some(tool_call_result_response(
            id,
            list_transitions_tool_text(),
        ))),
        "show_workflow" => Ok(Some(tool_call_result_response(
            id,
            show_workflow_tool_text(request),
        ))),
        "show_slice" => Ok(Some(tool_call_result_response(
            id,
            show_slice_tool_text(request),
        ))),
        "generate_site" => Ok(Some(tool_call_result_response(
            id,
            generate_site_tool_text(request),
        ))),
        "check_project" => Ok(Some(tool_call_result_response(
            id,
            check_project_tool_text(),
        ))),
        "verify_project" => Ok(Some(tool_call_result_response(
            id,
            verify_project_tool_text(),
        ))),
        "validate_event_model" => Ok(Some(tool_call_result_response(
            id,
            validate_event_model_tool_text(request),
        ))),
        "review_gate" => Ok(Some(tool_call_result_response(
            id,
            review_gate_tool_text(request),
        ))),
        "add_workflow" => Ok(Some(tool_call_result_response(
            id,
            add_workflow_tool_text(request),
        ))),
        "add_slice" => Ok(Some(tool_call_result_response(
            id,
            add_slice_tool_text(request),
        ))),
        "update_workflow" => Ok(Some(tool_call_result_response(
            id,
            update_workflow_tool_text(request),
        ))),
        "update_workflow_name" => Ok(Some(tool_call_result_response(
            id,
            update_workflow_name_tool_text(request),
        ))),
        "update_slice" => Ok(Some(tool_call_result_response(
            id,
            update_slice_tool_text(request),
        ))),
        "update_slice_kind" => Ok(Some(tool_call_result_response(
            id,
            update_slice_kind_tool_text(request),
        ))),
        "update_slice_name" => Ok(Some(tool_call_result_response(
            id,
            update_slice_name_tool_text(request),
        ))),
        "connect_workflow" => Ok(Some(tool_call_result_response(
            id,
            connect_workflow_tool_text(request),
        ))),
        _ => Ok(Some(error_response(
            id,
            -32602,
            format!("unknown EMC MCP tool {name}"),
        ))),
    }
}

fn tool_call_result_response(id: &Value, result: Result<String, ShellError>) -> Value {
    match result {
        Ok(text) => success_response(id, tool_result(text)),
        Err(error) => error_response(id, -32000, error.to_string()),
    }
}

fn init_project_tool_text(request: &Value) -> Result<String, ShellError> {
    let raw_name = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .and_then(|arguments| arguments.get("name"))
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("init_project requires name"))?;
    let name =
        parse_project_name(raw_name).map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(command::init(name)).map(|reports| reports.join("\n"))
}

fn list_workflows_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(command::list_workflows()).map(|reports| reports.join("\n"))
}

fn list_slices_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(command::list_slices()).map(|reports| reports.join("\n"))
}

fn list_transitions_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(command::list_transitions()).map(|reports| reports.join("\n"))
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
    interpret_collect_reports(command::show_workflow(slug)).map(|reports| reports.join("\n"))
}

fn show_slice_tool_text(request: &Value) -> Result<String, ShellError> {
    let raw_slug = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .and_then(|arguments| arguments.get("slug"))
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("show_slice requires slug"))?;
    let slug =
        parse_slice_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))?;
    interpret_collect_reports(command::show_slice(slug)).map(|reports| reports.join("\n"))
}

fn generate_site_tool_text(request: &Value) -> Result<String, ShellError> {
    let output = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .and_then(|arguments| arguments.get("output"))
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("generate_site requires output"))
        .and_then(|output| {
            parse_project_path(output).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::generate_site(output)).map(|reports| reports.join("\n"))
}

fn check_project_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(command::check_project()).map(|reports| reports.join("\n"))
}

fn verify_project_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(command::verify()).map(|reports| reports.join("\n"))
}

fn validate_event_model_tool_text(request: &Value) -> Result<String, ShellError> {
    let target = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .and_then(|arguments| arguments.get("target"))
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("validate_event_model requires target"))
        .and_then(|target| {
            parse_project_path(target).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::validate(target)).map(|reports| reports.join("\n"))
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
    interpret_collect_reports(command::review_gate_for_workflow(workflow_slug))
        .map(|reports| reports.join("\n"))
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
    interpret_collect_reports(command::add_workflow(NewWorkflow::new(
        name,
        description,
        slug,
    )))
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
    interpret_collect_reports(command::add_slice(NewSlice::new(
        workflow_slug,
        slice_slug,
        slice_name,
        slice_description,
        slice_kind,
    )))
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
    interpret_collect_reports(command::update_workflow_description(slug, description))
        .map(|reports| reports.join("\n"))
}

fn update_workflow_name_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_workflow_name requires arguments"))?;
    let slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_workflow_name requires slug"))
        .and_then(|raw_slug| {
            parse_workflow_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_workflow_name requires name"))
        .and_then(|raw_name| {
            parse_model_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::update_workflow_name(slug, name))
        .map(|reports| reports.join("\n"))
}

fn update_slice_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_slice requires arguments"))?;
    let slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_slice requires slug"))
        .and_then(|raw_slug| {
            parse_slice_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let description = arguments
        .get("description")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_slice requires description"))
        .and_then(|raw_description| {
            parse_model_description(raw_description)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::update_slice_description(slug, description))
        .map(|reports| reports.join("\n"))
}

fn update_slice_kind_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_slice_kind requires arguments"))?;
    let slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_slice_kind requires slug"))
        .and_then(|raw_slug| {
            parse_slice_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let kind = arguments
        .get("type")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_slice_kind requires type"))
        .and_then(|raw_type| {
            parse_slice_kind(raw_type).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::update_slice_kind(slug, kind))
        .map(|reports| reports.join("\n"))
}

fn update_slice_name_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_slice_name requires arguments"))?;
    let slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_slice_name requires slug"))
        .and_then(|raw_slug| {
            parse_slice_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("update_slice_name requires name"))
        .and_then(|raw_name| {
            parse_model_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::update_slice_name(slug, name))
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
    let connection = if let Some(raw_target) = arguments.get("to").and_then(Value::as_str) {
        let target_slug =
            parse_slice_slug(raw_target).map_err(|error| ShellError::message(error.to_string()))?;
        WorkflowConnection::new(
            workflow_slug,
            source_slug,
            target_slug,
            connection_kind,
            trigger,
        )
    } else {
        let target_workflow = arguments
            .get("to_workflow")
            .and_then(Value::as_str)
            .ok_or_else(|| ShellError::message("connect_workflow requires to or to_workflow"))
            .and_then(|raw_target| {
                parse_workflow_slug(raw_target)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
        let reason = arguments
            .get("reason")
            .and_then(Value::as_str)
            .ok_or_else(|| ShellError::message("connect_workflow requires reason"))
            .and_then(|raw_reason| {
                parse_model_description(raw_reason)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
        WorkflowConnection::new_workflow_exit(
            workflow_slug,
            source_slug,
            target_workflow,
            connection_kind,
            trigger,
            reason,
        )
    };
    interpret_collect_reports(command::connect_workflow(connection))
        .map(|reports| reports.join("\n"))
}

fn tool_result(text: String) -> Value {
    mcp_model_value(CallToolResult::success(vec![Content::text(text)])).unwrap_or_else(|error| {
        unreachable!("EMC MCP tool result must serialize through the rmcp model: {error}");
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
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    serde_json::to_writer(&mut lock, &response)
        .map_err(|error| ShellError::message(error.to_string()))?;
    writeln!(lock).map_err(|error| ShellError::message(error.to_string()))?;
    lock.flush()
        .map_err(|error| ShellError::message(error.to_string()))?;
    Ok(())
}

fn schema_object(value: Value) -> JsonObject {
    match value {
        Value::Object(object) => object,
        _ => unreachable!("EMC MCP tool schemas must be JSON objects"),
    }
}

fn mcp_model_value(model: impl Serialize) -> Result<Value, ShellError> {
    serde_json::to_value(model).map_err(|error| ShellError::message(error.to_string()))
}
