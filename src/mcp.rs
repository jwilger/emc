use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use rmcp::model::{
    CallToolResult, Content, Implementation, InitializeResult, JsonObject, ListToolsResult,
    ServerCapabilities, Tool, ToolsCapability,
};
use serde::Serialize;
use serde_json::{Value, json};

use crate::command;
use crate::core::connection::{WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::formal_slice_facts::{
    CommandErrorDefinitions, CommandErrorNames, CommandInputProvenanceChain,
    CommandObservedStreams, EmittedEventNames, NewAutomationDefinition, NewBitLevelDataFlow,
    NewBoardConnection, NewBoardElement, NewCommandDefinition, NewCommandErrorDefinition,
    NewCommandInput, NewControlDefinition, NewControlInputProvision, NewEventAttribute,
    NewEventDefinition, NewExternalPayloadDefinition, NewNavigationTarget, NewOutcomeDefinition,
    NewReadModelDefinition, NewReadModelField, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition, NewViewField, OutcomeEventNames, ReadModelDerivationSourceFields,
    ReadModelRelationshipFields, ScenarioKind, ScenarioStreamNames, ViewControls, ViewFilters,
    ViewLocalStates,
};
use crate::core::slice::NewSlice;
use crate::core::types::{
    SingletonRepeatBehavior, WorkflowCommandErrorRecord, WorkflowEntryLifecycleStateRecord,
    WorkflowOutcomeRecord, WorkflowOwnedDefinitionRecord, WorkflowTransitionEndpoint,
    WorkflowTransitionEvidenceRecord,
};
use crate::core::workflow::NewWorkflow;
use crate::io::dto::{
    parse_automation_name, parse_automation_reaction_description, parse_automation_trigger_name,
    parse_bit_encoding_semantics, parse_board_connection_endpoint,
    parse_board_connection_endpoint_kind, parse_board_element_declared_name,
    parse_board_element_kind, parse_board_element_name, parse_board_lane_id,
    parse_command_error_name, parse_command_error_names, parse_command_error_recovery_kind,
    parse_command_input_source_description, parse_command_input_source_kind, parse_command_name,
    parse_connection_kind, parse_contract_kind_name, parse_control_name,
    parse_control_recovery_behavior, parse_covered_definition_name, parse_data_flow_source,
    parse_data_flow_target, parse_datum_name, parse_datum_names, parse_event_attribute_name,
    parse_event_attribute_source_field, parse_event_attribute_source_kind,
    parse_event_attribute_source_name, parse_event_name, parse_event_names,
    parse_model_description, parse_model_name, parse_navigation_target_name,
    parse_navigation_target_names, parse_navigation_target_type, parse_outcome_label_name,
    parse_payload_contract_name, parse_project_name, parse_provenance_description,
    parse_read_model_derivation_rule, parse_read_model_field_source_kind, parse_read_model_name,
    parse_read_model_transitive_rule, parse_review_timestamp, parse_reviewer_id,
    parse_scenario_name, parse_scenario_step_text, parse_singleton_repeat_behavior,
    parse_sketch_token, parse_slice_kind, parse_slice_slug, parse_source_chain_hops,
    parse_stream_name, parse_stream_names, parse_transformation_semantics,
    parse_transition_trigger_name, parse_translation_external_event_name, parse_translation_name,
    parse_view_field_name, parse_view_field_source_kind, parse_view_name,
    parse_workflow_entry_lifecycle_evidence_text, parse_workflow_entry_lifecycle_state_name,
    parse_workflow_owned_definition_kind, parse_workflow_owned_definition_name,
    parse_workflow_slug, parse_workflow_transition_endpoint,
    parse_workflow_transition_evidence_text, parse_workflow_transition_kind,
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
            "Show modeled Lean4 and Quint workflow artifacts by workflow slug.",
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
            "Show modeled Lean4 and Quint slice artifacts by slice slug.",
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
            "record_clean_review",
            "Record a clean structured review for the current workflow digest.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        },
                        "reviewer": {
                            "type": "string"
                        },
                        "reviewed_at": {
                            "type": "string",
                            "format": "date-time",
                            "pattern": "^\\d{4}-\\d{2}-\\d{2}T\\d{2}:\\d{2}:\\d{2}\\.\\d{3}Z$",
                            "description": "Deterministic UTC millisecond timestamp, for example 2026-06-03T00:00:00.000Z."
                        }
                    },
                    "required": ["workflow", "reviewer", "reviewed_at"],
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
            "add_workflow_outcome",
            "Add a workflow composition outcome fact directly to Lean4 and Quint workflow artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        },
                        "source_slice": {
                            "type": "string"
                        },
                        "label": {
                            "type": "string"
                        },
                        "externally_relevant": {
                            "type": "boolean"
                        }
                    },
                    "required": ["workflow", "source_slice", "label", "externally_relevant"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_workflow_command_error",
            "Add a workflow composition command-local error fact directly to Lean4 and Quint workflow artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        },
                        "source_slice": {
                            "type": "string"
                        },
                        "command": {
                            "type": "string"
                        },
                        "error": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow", "source_slice", "command", "error"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_workflow_owned_definition",
            "Add a workflow composition ownership fact directly to Lean4 and Quint workflow artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        },
                        "source_slice": {
                            "type": "string"
                        },
                        "definition_kind": {
                            "type": "string"
                        },
                        "definition_name": {
                            "type": "string"
                        },
                        "definition_stream": {
                            "type": "string"
                        },
                        "source_provenance": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow", "source_slice", "definition_kind", "definition_name"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_workflow_transition_evidence",
            "Add workflow transition legality evidence directly to Lean4 and Quint workflow artifacts.",
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
                        "via": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "source_evidence": {
                            "type": "string"
                        },
                        "target_evidence": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow", "from", "to", "via", "name", "source_evidence", "target_evidence"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "require_workflow_entry_lifecycle_coverage",
            "Mark a workflow as requiring formal application-entry lifecycle coverage in Lean4 and Quint.",
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
            "add_workflow_entry_lifecycle_state",
            "Add formal application-entry lifecycle state coverage evidence directly to Lean4 and Quint workflow artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "workflow": {
                            "type": "string"
                        },
                        "state": {
                            "type": "string"
                        },
                        "step": {
                            "type": "string"
                        },
                        "evidence": {
                            "type": "string"
                        }
                    },
                    "required": ["workflow", "state", "step", "evidence"],
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
            "add_slice_scenario",
            "Add an acceptance or contract GWT scenario directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "kind": {
                            "type": "string",
                            "enum": ["acceptance", "contract"]
                        },
                        "name": {
                            "type": "string"
                        },
                        "given": {
                            "type": "string"
                        },
                        "when": {
                            "type": "string"
                        },
                        "then": {
                            "type": "string"
                        },
                        "contract_kind": {
                            "type": "string"
                        },
                        "covered_definition": {
                            "type": "string"
                        },
                        "read_streams": {
                            "type": "string"
                        },
                        "written_streams": {
                            "type": "string"
                        },
                        "error_references": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "kind", "name", "given", "when", "then"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_bit_level_data_flow",
            "Add source, transformation, target, and bit-encoding semantics directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "datum": {
                            "type": "string"
                        },
                        "source": {
                            "type": "string"
                        },
                        "transformation": {
                            "type": "string"
                        },
                        "target": {
                            "type": "string"
                        },
                        "bit_encoding": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "datum", "source", "transformation", "target", "bit_encoding"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_board_element",
            "Add a board element causal-shape fact directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "kind": {
                            "type": "string"
                        },
                        "lane": {
                            "type": "string"
                        },
                        "declared_name": {
                            "type": "string"
                        },
                        "main_path": {
                            "type": "boolean"
                        }
                    },
                    "required": ["slice", "name", "kind", "lane", "declared_name", "main_path"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_board_connection",
            "Add a board connection causal-shape fact directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "source": {
                            "type": "string"
                        },
                        "source_kind": {
                            "type": "string"
                        },
                        "target": {
                            "type": "string"
                        },
                        "target_kind": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "source", "source_kind", "target", "target_kind"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_command_definition",
            "Add a command, input provenance, and emitted events directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "input": {
                            "type": "string"
                        },
                        "input_source": {
                            "type": "string"
                        },
                        "input_description": {
                            "type": "string"
                        },
                        "input_provenance": {
                            "type": "string"
                        },
                        "emits": {
                            "type": "string"
                        },
                        "observes": {
                            "type": "string"
                        },
                        "source_event": {
                            "type": "string"
                        },
                        "source_attribute": {
                            "type": "string"
                        },
                        "source_payload": {
                            "type": "string"
                        },
                        "source_field": {
                            "type": "string"
                        },
                        "singleton": {
                            "type": "boolean"
                        },
                        "repeat_behavior": {
                            "type": "string"
                        },
                        "error": {
                            "type": "string"
                        },
                        "error_scenario": {
                            "type": "string"
                        },
                        "error_recovery": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "name", "input", "input_source", "input_description", "input_provenance", "emits"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_automation_definition",
            "Add an automation trigger, issued command, handled errors, and reaction semantics directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "trigger": {
                            "type": "string"
                        },
                        "command": {
                            "type": "string"
                        },
                        "handled_errors": {
                            "type": "string"
                        },
                        "reaction": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "name", "trigger", "command", "handled_errors", "reaction"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_translation_definition",
            "Add a translation external event, payload contract, and target command directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "external_event": {
                            "type": "string"
                        },
                        "payload_contract": {
                            "type": "string"
                        },
                        "command": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "name", "external_event", "payload_contract", "command"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_external_payload_definition",
            "Add an external payload field contract directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "field": {
                            "type": "string"
                        },
                        "field_provenance": {
                            "type": "string"
                        },
                        "bit_encoding": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "name", "field", "field_provenance", "bit_encoding"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_event_definition",
            "Add an event, stream, attribute source, and provenance directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "stream": {
                            "type": "string"
                        },
                        "attribute": {
                            "type": "string"
                        },
                        "attribute_source": {
                            "type": "string"
                        },
                        "attribute_source_name": {
                            "type": "string"
                        },
                        "attribute_source_field": {
                            "type": "string"
                        },
                        "attribute_provenance": {
                            "type": "string"
                        },
                        "observed": {
                            "type": "boolean"
                        },
                        "shared": {
                            "type": "boolean"
                        }
                    },
                    "required": ["slice", "name", "stream", "attribute", "attribute_source", "attribute_source_name", "attribute_source_field", "attribute_provenance"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_outcome_definition",
            "Add an outcome label and backing event set directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "label": {
                            "type": "string"
                        },
                        "events": {
                            "type": "string"
                        },
                        "externally_relevant": {
                            "type": "boolean"
                        }
                    },
                    "required": ["slice", "label", "events", "externally_relevant"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_read_model_definition",
            "Add a read model, field source, and field provenance directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "field": {
                            "type": "string"
                        },
                        "field_source": {
                            "type": "string"
                        },
                        "source_event": {
                            "type": "string"
                        },
                        "source_attribute": {
                            "type": "string"
                        },
                        "derivation_rule": {
                            "type": "string"
                        },
                        "derivation_source_fields": {
                            "type": "string"
                        },
                        "derivation_scenario": {
                            "type": "string"
                        },
                        "absence_event": {
                            "type": "string"
                        },
                        "absence_scenario": {
                            "type": "string"
                        },
                        "field_provenance": {
                            "type": "string"
                        },
                        "transitive": {
                            "type": "boolean"
                        },
                        "relationship_fields": {
                            "type": "string"
                        },
                        "transitive_rule": {
                            "type": "string"
                        },
                        "example_scenario": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "name", "field", "field_source", "field_provenance"],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "add_view_definition",
            "Add a view field plus command control, input provenance, error handling, and navigation directly to Lean4 and Quint slice artifacts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "slice": {
                            "type": "string"
                        },
                        "name": {
                            "type": "string"
                        },
                        "read_model": {
                            "type": "string"
                        },
                        "field": {
                            "type": "string"
                        },
                        "source_field": {
                            "type": "string"
                        },
                        "sketch_token": {
                            "type": "string"
                        },
                        "field_provenance": {
                            "type": "string"
                        },
                        "bit_encoding": {
                            "type": "string"
                        },
                        "control": {
                            "type": "string"
                        },
                        "control_command": {
                            "type": "string"
                        },
                        "control_input": {
                            "type": "string"
                        },
                        "control_input_source": {
                            "type": "string"
                        },
                        "control_input_description": {
                            "type": "string"
                        },
                        "control_input_sketch_token": {
                            "type": "string"
                        },
                        "control_input_visible": {
                            "type": "boolean"
                        },
                        "control_input_decision": {
                            "type": "boolean"
                        },
                        "handled_errors": {
                            "type": "string"
                        },
                        "recovery_behavior": {
                            "type": "string"
                        },
                        "control_sketch_token": {
                            "type": "string"
                        },
                        "navigation_type": {
                            "type": "string"
                        },
                        "navigation_target": {
                            "type": "string"
                        },
                        "local_states": {
                            "type": "string"
                        },
                        "filters": {
                            "type": "string"
                        },
                        "external_workflow": {
                            "type": "string"
                        },
                        "external_system": {
                            "type": "string"
                        },
                        "handoff_contract": {
                            "type": "string"
                        }
                    },
                    "required": ["slice", "name", "read_model", "field", "source_field", "sketch_token", "field_provenance", "bit_encoding", "control", "control_command", "control_input", "control_input_source", "control_input_description", "control_input_sketch_token", "control_input_visible", "control_input_decision", "handled_errors", "recovery_behavior", "control_sketch_token", "navigation_type", "navigation_target"],
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
            "remove_slice",
            "Remove a business slice and regenerate synchronized model artifacts.",
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
            "remove_workflow",
            "Remove a business workflow and its owned slices, then regenerate synchronized model artifacts.",
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
                        },
                        "payload_contract": {
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
        Tool::new(
            "remove_transition",
            "Remove a workflow transition and regenerate synchronized model artifacts.",
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
                        }
                    },
                    "required": ["workflow", "from", "via", "name"],
                    "oneOf": [
                        {"required": ["to"]},
                        {"required": ["to_workflow"]}
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
        "check_project" => Ok(Some(tool_call_result_response(
            id,
            check_project_tool_text(),
        ))),
        "verify_project" => Ok(Some(tool_call_result_response(
            id,
            verify_project_tool_text(),
        ))),
        "review_gate" => Ok(Some(tool_call_result_response(
            id,
            review_gate_tool_text(request),
        ))),
        "record_clean_review" => Ok(Some(tool_call_result_response(
            id,
            record_clean_review_tool_text(request),
        ))),
        "add_workflow" => Ok(Some(tool_call_result_response(
            id,
            add_workflow_tool_text(request),
        ))),
        "add_workflow_outcome" => Ok(Some(tool_call_result_response(
            id,
            add_workflow_outcome_tool_text(request),
        ))),
        "add_workflow_command_error" => Ok(Some(tool_call_result_response(
            id,
            add_workflow_command_error_tool_text(request),
        ))),
        "add_workflow_owned_definition" => Ok(Some(tool_call_result_response(
            id,
            add_workflow_owned_definition_tool_text(request),
        ))),
        "add_workflow_transition_evidence" => Ok(Some(tool_call_result_response(
            id,
            add_workflow_transition_evidence_tool_text(request),
        ))),
        "require_workflow_entry_lifecycle_coverage" => Ok(Some(tool_call_result_response(
            id,
            require_workflow_entry_lifecycle_coverage_tool_text(request),
        ))),
        "add_workflow_entry_lifecycle_state" => Ok(Some(tool_call_result_response(
            id,
            add_workflow_entry_lifecycle_state_tool_text(request),
        ))),
        "add_slice" => Ok(Some(tool_call_result_response(
            id,
            add_slice_tool_text(request),
        ))),
        "add_slice_scenario" => Ok(Some(tool_call_result_response(
            id,
            add_slice_scenario_tool_text(request),
        ))),
        "add_bit_level_data_flow" => Ok(Some(tool_call_result_response(
            id,
            add_bit_level_data_flow_tool_text(request),
        ))),
        "add_board_element" => Ok(Some(tool_call_result_response(
            id,
            add_board_element_tool_text(request),
        ))),
        "add_board_connection" => Ok(Some(tool_call_result_response(
            id,
            add_board_connection_tool_text(request),
        ))),
        "add_command_definition" => Ok(Some(tool_call_result_response(
            id,
            add_command_definition_tool_text(request),
        ))),
        "add_automation_definition" => Ok(Some(tool_call_result_response(
            id,
            add_automation_definition_tool_text(request),
        ))),
        "add_translation_definition" => Ok(Some(tool_call_result_response(
            id,
            add_translation_definition_tool_text(request),
        ))),
        "add_event_definition" => Ok(Some(tool_call_result_response(
            id,
            add_event_definition_tool_text(request),
        ))),
        "add_outcome_definition" => Ok(Some(tool_call_result_response(
            id,
            add_outcome_definition_tool_text(request),
        ))),
        "add_external_payload_definition" => Ok(Some(tool_call_result_response(
            id,
            add_external_payload_definition_tool_text(request),
        ))),
        "add_read_model_definition" => Ok(Some(tool_call_result_response(
            id,
            add_read_model_definition_tool_text(request),
        ))),
        "add_view_definition" => Ok(Some(tool_call_result_response(
            id,
            add_view_definition_tool_text(request),
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
        "remove_slice" => Ok(Some(tool_call_result_response(
            id,
            remove_slice_tool_text(request),
        ))),
        "remove_workflow" => Ok(Some(tool_call_result_response(
            id,
            remove_workflow_tool_text(request),
        ))),
        "connect_workflow" => Ok(Some(tool_call_result_response(
            id,
            connect_workflow_tool_text(request),
        ))),
        "remove_transition" => Ok(Some(tool_call_result_response(
            id,
            remove_transition_tool_text(request),
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

fn check_project_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(command::check_project()).map(|reports| reports.join("\n"))
}

fn verify_project_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(command::verify()).map(|reports| reports.join("\n"))
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

fn record_clean_review_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("record_clean_review requires arguments"))?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("record_clean_review requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let reviewer = arguments
        .get("reviewer")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("record_clean_review requires reviewer"))
        .and_then(|raw_reviewer| {
            parse_reviewer_id(raw_reviewer).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let reviewed_at = arguments
        .get("reviewed_at")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("record_clean_review requires reviewed_at"))
        .and_then(|raw_reviewed_at| {
            parse_review_timestamp(raw_reviewed_at)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::record_clean_review(
        workflow_slug,
        reviewer,
        reviewed_at,
    ))
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

fn add_workflow_outcome_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_workflow_outcome requires arguments"))?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_outcome requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_slice = arguments
        .get("source_slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_outcome requires source_slice"))
        .and_then(|raw_source_slice| {
            WorkflowTransitionEndpoint::try_new(raw_source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let label = arguments
        .get("label")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_outcome requires label"))
        .and_then(|raw_label| {
            parse_outcome_label_name(raw_label)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let externally_relevant = arguments
        .get("externally_relevant")
        .and_then(Value::as_bool)
        .ok_or_else(|| ShellError::message("add_workflow_outcome requires externally_relevant"))?;

    interpret_collect_reports(command::add_workflow_outcome(
        workflow_slug,
        WorkflowOutcomeRecord::new(source_slice, label, externally_relevant),
    ))
    .map(|reports| reports.join("\n"))
}

fn add_workflow_command_error_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_workflow_command_error requires arguments"))?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_command_error requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_slice = arguments
        .get("source_slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_command_error requires source_slice"))
        .and_then(|raw_source_slice| {
            WorkflowTransitionEndpoint::try_new(raw_source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_command_error requires command"))
        .and_then(|raw_command| {
            parse_command_name(raw_command).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let error_name = arguments
        .get("error")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_command_error requires error"))
        .and_then(|raw_error| {
            parse_command_error_name(raw_error)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::add_workflow_command_error(
        workflow_slug,
        WorkflowCommandErrorRecord::new(source_slice, command_name, error_name),
    ))
    .map(|reports| reports.join("\n"))
}

fn add_workflow_owned_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_workflow_owned_definition requires arguments"))?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_owned_definition requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_slice = arguments
        .get("source_slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_owned_definition requires source_slice"))
        .and_then(|raw_source_slice| {
            WorkflowTransitionEndpoint::try_new(raw_source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let definition_kind = arguments
        .get("definition_kind")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_workflow_owned_definition requires definition_kind")
        })
        .and_then(|raw_definition_kind| {
            parse_workflow_owned_definition_kind(raw_definition_kind)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let definition_name = arguments
        .get("definition_name")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_workflow_owned_definition requires definition_name")
        })
        .and_then(|raw_definition_name| {
            parse_workflow_owned_definition_name(raw_definition_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let definition_stream = arguments
        .get("definition_stream")
        .and_then(Value::as_str)
        .map(|raw_definition_stream| {
            parse_stream_name(raw_definition_stream)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()?;
    let source_provenance = arguments
        .get("source_provenance")
        .and_then(Value::as_str)
        .map(|raw_source_provenance| {
            parse_model_description(raw_source_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()?;
    let definition = match (definition_stream, source_provenance) {
        (Some(definition_stream), Some(source_provenance)) => {
            WorkflowOwnedDefinitionRecord::new_with_event_identity(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
            )
        }
        _ => WorkflowOwnedDefinitionRecord::new(source_slice, definition_kind, definition_name),
    };

    interpret_collect_reports(command::add_workflow_owned_definition(
        workflow_slug,
        definition,
    ))
    .map(|reports| reports.join("\n"))
}

fn add_workflow_transition_evidence_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| {
            ShellError::message("add_workflow_transition_evidence requires arguments")
        })?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_transition_evidence requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source = arguments
        .get("from")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_transition_evidence requires from"))
        .and_then(|raw_source| {
            parse_workflow_transition_endpoint(raw_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let target = arguments
        .get("to")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_transition_evidence requires to"))
        .and_then(|raw_target| {
            parse_workflow_transition_endpoint(raw_target)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let kind = arguments
        .get("via")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_transition_evidence requires via"))
        .and_then(|raw_kind| {
            parse_workflow_transition_kind(raw_kind)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let trigger = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_transition_evidence requires name"))
        .and_then(|raw_trigger| {
            parse_transition_trigger_name(raw_trigger)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_evidence = arguments
        .get("source_evidence")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_workflow_transition_evidence requires source_evidence")
        })
        .and_then(|raw_source_evidence| {
            parse_workflow_transition_evidence_text(raw_source_evidence)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let target_evidence = arguments
        .get("target_evidence")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_workflow_transition_evidence requires target_evidence")
        })
        .and_then(|raw_target_evidence| {
            parse_workflow_transition_evidence_text(raw_target_evidence)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::add_workflow_transition_evidence(
        workflow_slug,
        WorkflowTransitionEvidenceRecord::new(
            source,
            target,
            kind,
            trigger,
            source_evidence,
            target_evidence,
        ),
    ))
    .map(|reports| reports.join("\n"))
}

fn require_workflow_entry_lifecycle_coverage_tool_text(
    request: &Value,
) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| {
            ShellError::message("require_workflow_entry_lifecycle_coverage requires arguments")
        })?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("require_workflow_entry_lifecycle_coverage requires workflow")
        })
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::require_workflow_entry_lifecycle_coverage(
        workflow_slug,
    ))
    .map(|reports| reports.join("\n"))
}

fn add_workflow_entry_lifecycle_state_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| {
            ShellError::message("add_workflow_entry_lifecycle_state requires arguments")
        })?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_entry_lifecycle_state requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let state = arguments
        .get("state")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_entry_lifecycle_state requires state"))
        .and_then(|raw_state| {
            parse_workflow_entry_lifecycle_state_name(raw_state)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let step = arguments
        .get("step")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_entry_lifecycle_state requires step"))
        .and_then(|raw_step| {
            parse_workflow_transition_endpoint(raw_step)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let evidence = arguments
        .get("evidence")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_workflow_entry_lifecycle_state requires evidence"))
        .and_then(|raw_evidence| {
            parse_workflow_entry_lifecycle_evidence_text(raw_evidence)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::add_workflow_entry_lifecycle_state(
        workflow_slug,
        WorkflowEntryLifecycleStateRecord::new(state, step, evidence),
    ))
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

fn add_slice_scenario_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_slice_scenario requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice_scenario requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let scenario_kind = arguments
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice_scenario requires kind"))?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice_scenario requires name"))
        .and_then(|raw_name| {
            parse_scenario_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let given = arguments
        .get("given")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice_scenario requires given"))
        .and_then(|raw_given| {
            parse_scenario_step_text(raw_given)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let when = arguments
        .get("when")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice_scenario requires when"))
        .and_then(|raw_when| {
            parse_scenario_step_text(raw_when)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let then = arguments
        .get("then")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_slice_scenario requires then"))
        .and_then(|raw_then| {
            parse_scenario_step_text(raw_then)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    let scenario = match scenario_kind {
        "acceptance" => NewSliceScenario::new(
            slice_slug,
            ScenarioKind::acceptance(),
            name,
            given,
            when,
            then,
        ),
        "contract" => {
            let contract_kind = arguments
                .get("contract_kind")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message("add_slice_scenario requires contract_kind"))
                .and_then(|raw_contract_kind| {
                    parse_contract_kind_name(raw_contract_kind)
                        .map_err(|error| ShellError::message(error.to_string()))
                })?;
            let covered_definition = arguments
                .get("covered_definition")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ShellError::message("add_slice_scenario requires covered_definition")
                })
                .and_then(|raw_covered_definition| {
                    parse_covered_definition_name(raw_covered_definition)
                        .map_err(|error| ShellError::message(error.to_string()))
                })?;
            NewSliceScenario::new_contract(
                slice_slug,
                name,
                given,
                when,
                then,
                contract_kind,
                covered_definition,
            )
        }
        _ => {
            return Err(ShellError::message(format!(
                "invalid scenario kind: {scenario_kind}"
            )));
        }
    };
    let scenario = apply_optional_scenario_streams(arguments, scenario)?;
    let scenario = apply_optional_scenario_error_references(arguments, scenario)?;

    interpret_collect_reports(command::add_slice_scenario(scenario))
        .map(|reports| reports.join("\n"))
}

fn apply_optional_scenario_streams(
    arguments: &Value,
    scenario: NewSliceScenario,
) -> Result<NewSliceScenario, ShellError> {
    match (
        arguments.get("read_streams").and_then(Value::as_str),
        arguments.get("written_streams").and_then(Value::as_str),
    ) {
        (Some(raw_read_streams), Some(raw_written_streams)) => {
            let read_streams = parse_stream_names(raw_read_streams)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let written_streams = parse_stream_names(raw_written_streams)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(scenario.with_streams(
                ScenarioStreamNames::from_streams(read_streams),
                ScenarioStreamNames::from_streams(written_streams),
            ))
        }
        (None, None) => Ok(scenario),
        _ => Err(ShellError::message(
            "add_slice_scenario requires read_streams and written_streams together",
        )),
    }
}

fn apply_optional_scenario_error_references(
    arguments: &Value,
    scenario: NewSliceScenario,
) -> Result<NewSliceScenario, ShellError> {
    if let Some(raw_error_references) = arguments.get("error_references").and_then(Value::as_str) {
        let error_references = parse_command_error_names(raw_error_references)
            .map_err(|error| ShellError::message(error.to_string()))?;
        Ok(scenario.with_error_references(CommandErrorNames::from_names(error_references)))
    } else {
        Ok(scenario)
    }
}

fn add_bit_level_data_flow_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_bit_level_data_flow requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_bit_level_data_flow requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let datum = arguments
        .get("datum")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_bit_level_data_flow requires datum"))
        .and_then(|raw_datum| {
            parse_datum_name(raw_datum).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source = arguments
        .get("source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_bit_level_data_flow requires source"))
        .and_then(|raw_source| {
            parse_data_flow_source(raw_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let transformation = arguments
        .get("transformation")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_bit_level_data_flow requires transformation"))
        .and_then(|raw_transformation| {
            parse_transformation_semantics(raw_transformation)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let target = arguments
        .get("target")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_bit_level_data_flow requires target"))
        .and_then(|raw_target| {
            parse_data_flow_target(raw_target)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let bit_encoding = arguments
        .get("bit_encoding")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_bit_level_data_flow requires bit_encoding"))
        .and_then(|raw_bit_encoding| {
            parse_bit_encoding_semantics(raw_bit_encoding)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::add_bit_level_data_flow(NewBitLevelDataFlow::new(
        slice_slug,
        datum,
        source,
        transformation,
        target,
        bit_encoding,
    )))
    .map(|reports| reports.join("\n"))
}

fn add_board_element_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_board_element requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_element requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_element requires name"))
        .and_then(|raw_name| {
            parse_board_element_name(raw_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let kind = arguments
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_element requires kind"))
        .and_then(|raw_kind| {
            parse_board_element_kind(raw_kind)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let lane = arguments
        .get("lane")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_element requires lane"))
        .and_then(|raw_lane| {
            parse_board_lane_id(raw_lane).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let declared_name = arguments
        .get("declared_name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_element requires declared_name"))
        .and_then(|raw_declared_name| {
            parse_board_element_declared_name(raw_declared_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let main_path = arguments
        .get("main_path")
        .and_then(Value::as_bool)
        .ok_or_else(|| ShellError::message("add_board_element requires main_path"))?;

    interpret_collect_reports(command::add_board_element(NewBoardElement::new(
        slice_slug,
        name,
        kind,
        lane,
        declared_name,
        main_path,
    )))
    .map(|reports| reports.join("\n"))
}

fn add_board_connection_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_board_connection requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_connection requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source = arguments
        .get("source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_connection requires source"))
        .and_then(|raw_source| {
            parse_board_connection_endpoint(raw_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_kind = arguments
        .get("source_kind")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_connection requires source_kind"))
        .and_then(|raw_source_kind| {
            parse_board_connection_endpoint_kind(raw_source_kind)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let target = arguments
        .get("target")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_connection requires target"))
        .and_then(|raw_target| {
            parse_board_connection_endpoint(raw_target)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let target_kind = arguments
        .get("target_kind")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_board_connection requires target_kind"))
        .and_then(|raw_target_kind| {
            parse_board_connection_endpoint_kind(raw_target_kind)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::add_board_connection(NewBoardConnection::new(
        slice_slug,
        source,
        source_kind,
        target,
        target_kind,
    )))
    .map(|reports| reports.join("\n"))
}

fn add_command_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_command_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_command_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_command_definition requires name"))
        .and_then(|raw_name| {
            parse_command_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_name = arguments
        .get("input")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_command_definition requires input"))
        .and_then(|raw_input| {
            parse_datum_name(raw_input).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_source = arguments
        .get("input_source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_command_definition requires input_source"))
        .and_then(|raw_source| {
            parse_command_input_source_kind(raw_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_description = arguments
        .get("input_description")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_command_definition requires input_description"))
        .and_then(|raw_description| {
            parse_command_input_source_description(raw_description)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let provenance_chain = arguments
        .get("input_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_command_definition requires input_provenance"))
        .and_then(|raw_provenance| {
            parse_source_chain_hops(raw_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let emitted_events = arguments
        .get("emits")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_command_definition requires emits"))
        .and_then(|raw_emits| {
            parse_event_names(raw_emits).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let observed_streams = arguments
        .get("observes")
        .and_then(Value::as_str)
        .map(parse_stream_names)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_event = arguments
        .get("source_event")
        .and_then(Value::as_str)
        .map(parse_event_name)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_attribute = arguments
        .get("source_attribute")
        .and_then(Value::as_str)
        .map(parse_event_attribute_name)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_payload = arguments
        .get("source_payload")
        .and_then(Value::as_str)
        .map(parse_event_attribute_source_name)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_field = arguments
        .get("source_field")
        .and_then(Value::as_str)
        .map(parse_event_attribute_source_field)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let command_errors = parse_optional_command_errors(arguments)?;

    let singleton_repeat_behavior = parse_optional_singleton_repeat_behavior(arguments)?;
    let command_input = NewCommandInput::new(
        input_name,
        input_source,
        input_description,
        CommandInputProvenanceChain::from_hops(provenance_chain),
    );
    let command_input = match (source_event, source_attribute, source_payload, source_field) {
        (Some(event), Some(attribute), None, None) => {
            command_input.with_event_stream_source(event, attribute)
        }
        (None, None, Some(payload), Some(field)) => {
            command_input.with_external_payload_source(payload, field)
        }
        (None, None, None, None) => command_input,
        (Some(_), None, None, None) => {
            return Err(ShellError::message(
                "add_command_definition requires source_attribute when source_event is provided",
            ));
        }
        (None, Some(_), None, None) => {
            return Err(ShellError::message(
                "add_command_definition requires source_event when source_attribute is provided",
            ));
        }
        (None, None, Some(_), None) => {
            return Err(ShellError::message(
                "add_command_definition requires source_field when source_payload is provided",
            ));
        }
        (None, None, None, Some(_)) => {
            return Err(ShellError::message(
                "add_command_definition requires source_payload when source_field is provided",
            ));
        }
        _ => {
            return Err(ShellError::message(
                "add_command_definition accepts one command input source reference at a time",
            ));
        }
    };
    let command_definition = NewCommandDefinition::new(
        slice_slug,
        command_name,
        command_input,
        EmittedEventNames::from_events(emitted_events),
    )
    .with_observed_streams(CommandObservedStreams::from_streams(
        observed_streams.unwrap_or_default(),
    ))
    .with_errors(command_errors);
    let command_definition = singleton_repeat_behavior
        .map_or(command_definition.clone(), |behavior| {
            command_definition.with_singleton_repeat_behavior(behavior)
        });

    interpret_collect_reports(command::add_command_definition(command_definition))
        .map(|reports| reports.join("\n"))
}

fn parse_optional_singleton_repeat_behavior(
    arguments: &Value,
) -> Result<Option<SingletonRepeatBehavior>, ShellError> {
    match (
        arguments.get("singleton").and_then(Value::as_bool),
        arguments.get("repeat_behavior").and_then(Value::as_str),
    ) {
        (Some(true), Some(repeat_behavior)) => parse_singleton_repeat_behavior(repeat_behavior)
            .map(Some)
            .map_err(|error| ShellError::message(error.to_string())),
        (Some(false), _) | (None, None) => Ok(None),
        (Some(true), None) => Err(ShellError::message(
            "add_command_definition requires repeat_behavior when singleton is true",
        )),
        (None, Some(_)) => Err(ShellError::message(
            "add_command_definition requires singleton when repeat_behavior is provided",
        )),
    }
}

fn parse_optional_command_errors(arguments: &Value) -> Result<CommandErrorDefinitions, ShellError> {
    let maybe_error = arguments.get("error").and_then(Value::as_str);
    let maybe_scenario = arguments.get("error_scenario").and_then(Value::as_str);
    let maybe_recovery = arguments.get("error_recovery").and_then(Value::as_str);

    match (maybe_error, maybe_scenario, maybe_recovery) {
        (None, None, None) => Ok(CommandErrorDefinitions::empty()),
        (Some(error), Some(scenario), Some(recovery)) => {
            let command_error = parse_command_error_name(error)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let command_error_scenario = parse_scenario_name(scenario)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let command_error_recovery = parse_command_error_recovery_kind(recovery)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(CommandErrorDefinitions::from_errors([
                NewCommandErrorDefinition::new(
                    command_error,
                    command_error_scenario,
                    command_error_recovery,
                ),
            ]))
        }
        _ => Err(ShellError::message(
            "add_command_definition requires error, error_scenario, and error_recovery together",
        )),
    }
}

fn add_automation_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_automation_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_automation_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let automation_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_automation_definition requires name"))
        .and_then(|raw_name| {
            parse_automation_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let trigger_name = arguments
        .get("trigger")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_automation_definition requires trigger"))
        .and_then(|raw_trigger| {
            parse_automation_trigger_name(raw_trigger)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_automation_definition requires command"))
        .and_then(|raw_command| {
            parse_command_name(raw_command).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let handled_errors = arguments
        .get("handled_errors")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_automation_definition requires handled_errors"))
        .and_then(|raw_errors| {
            parse_command_error_names(raw_errors)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let reaction_description = arguments
        .get("reaction")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_automation_definition requires reaction"))
        .and_then(|raw_reaction| {
            parse_automation_reaction_description(raw_reaction)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::add_automation_definition(
        NewAutomationDefinition::new(
            slice_slug,
            automation_name,
            trigger_name,
            command_name,
            CommandErrorNames::from_names(handled_errors),
            reaction_description,
        ),
    ))
    .map(|reports| reports.join("\n"))
}

fn add_translation_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_translation_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_translation_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let translation_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_translation_definition requires name"))
        .and_then(|raw_name| {
            parse_translation_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let external_event_name = arguments
        .get("external_event")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_translation_definition requires external_event"))
        .and_then(|raw_external_event| {
            parse_translation_external_event_name(raw_external_event)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let payload_contract_name = arguments
        .get("payload_contract")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_translation_definition requires payload_contract"))
        .and_then(|raw_payload_contract| {
            parse_payload_contract_name(raw_payload_contract)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_translation_definition requires command"))
        .and_then(|raw_command| {
            parse_command_name(raw_command).map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::add_translation_definition(
        NewTranslationDefinition::new(
            slice_slug,
            translation_name,
            external_event_name,
            payload_contract_name,
            command_name,
        ),
    ))
    .map(|reports| reports.join("\n"))
}

fn add_external_payload_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_external_payload_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_external_payload_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let payload_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_external_payload_definition requires name"))
        .and_then(|raw_name| {
            parse_event_attribute_source_name(raw_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let payload_field = arguments
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_external_payload_definition requires field"))
        .and_then(|raw_field| {
            parse_event_attribute_source_field(raw_field)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let field_provenance = arguments
        .get("field_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_external_payload_definition requires field_provenance")
        })
        .and_then(|raw_field_provenance| {
            parse_provenance_description(raw_field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let bit_encoding = arguments
        .get("bit_encoding")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_external_payload_definition requires bit_encoding"))
        .and_then(|raw_bit_encoding| {
            parse_bit_encoding_semantics(raw_bit_encoding)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(command::add_external_payload_definition(
        NewExternalPayloadDefinition::new(
            slice_slug,
            payload_name,
            payload_field,
            field_provenance,
            bit_encoding,
        ),
    ))
    .map(|reports| reports.join("\n"))
}

fn add_outcome_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_outcome_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_outcome_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let label = arguments
        .get("label")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_outcome_definition requires label"))
        .and_then(|raw_label| {
            parse_outcome_label_name(raw_label)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let events = arguments
        .get("events")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_outcome_definition requires events"))
        .and_then(|raw_events| {
            parse_event_names(raw_events).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let externally_relevant = arguments
        .get("externally_relevant")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            ShellError::message("add_outcome_definition requires externally_relevant")
        })?;

    interpret_collect_reports(command::add_outcome_definition(NewOutcomeDefinition::new(
        slice_slug,
        label,
        OutcomeEventNames::from_events(events),
        externally_relevant,
    )))
    .map(|reports| reports.join("\n"))
}

fn add_event_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_event_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_event_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let event_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_event_definition requires name"))
        .and_then(|raw_name| {
            parse_event_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let stream_name = arguments
        .get("stream")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_event_definition requires stream"))
        .and_then(|raw_stream| {
            parse_stream_name(raw_stream).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let attribute_name = arguments
        .get("attribute")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_event_definition requires attribute"))
        .and_then(|raw_attribute| {
            parse_event_attribute_name(raw_attribute)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let attribute_source_kind = arguments
        .get("attribute_source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_event_definition requires attribute_source"))
        .and_then(|raw_attribute_source| {
            parse_event_attribute_source_kind(raw_attribute_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let attribute_source_name = arguments
        .get("attribute_source_name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_event_definition requires attribute_source_name"))
        .and_then(|raw_attribute_source_name| {
            parse_event_attribute_source_name(raw_attribute_source_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let attribute_source_field = arguments
        .get("attribute_source_field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_event_definition requires attribute_source_field"))
        .and_then(|raw_attribute_source_field| {
            parse_event_attribute_source_field(raw_attribute_source_field)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let provenance_description = arguments
        .get("attribute_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_event_definition requires attribute_provenance"))
        .and_then(|raw_provenance| {
            parse_provenance_description(raw_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    let attribute = NewEventAttribute::new(
        attribute_name,
        attribute_source_kind,
        attribute_source_name,
        attribute_source_field,
        provenance_description,
    );
    let observed = arguments
        .get("observed")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let shared = arguments
        .get("shared")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let event = if shared {
        NewEventDefinition::new_shared(slice_slug, event_name, stream_name, attribute)
    } else if observed {
        NewEventDefinition::new_observed(slice_slug, event_name, stream_name, attribute)
    } else {
        NewEventDefinition::new(slice_slug, event_name, stream_name, attribute)
    };

    interpret_collect_reports(command::add_event_definition(event))
        .map(|reports| reports.join("\n"))
}

fn add_read_model_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_read_model_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_read_model_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let read_model_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_read_model_definition requires name"))
        .and_then(|raw_name| {
            parse_read_model_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let field_name = arguments
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_read_model_definition requires field"))
        .and_then(|raw_field| {
            parse_datum_name(raw_field).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let field_source_kind = arguments
        .get("field_source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_read_model_definition requires field_source"))
        .and_then(|raw_field_source| {
            parse_read_model_field_source_kind(raw_field_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let provenance_description = arguments
        .get("field_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_read_model_definition requires field_provenance"))
        .and_then(|raw_provenance| {
            parse_provenance_description(raw_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_event = arguments.get("source_event").and_then(Value::as_str);
    let source_attribute = arguments.get("source_attribute").and_then(Value::as_str);
    let derivation_rule = arguments.get("derivation_rule").and_then(Value::as_str);
    let derivation_source_fields = arguments
        .get("derivation_source_fields")
        .and_then(Value::as_str);
    let derivation_scenario = arguments.get("derivation_scenario").and_then(Value::as_str);
    let absence_event = arguments.get("absence_event").and_then(Value::as_str);
    let absence_scenario = arguments.get("absence_scenario").and_then(Value::as_str);
    let read_model_field = match (
        source_event,
        source_attribute,
        derivation_rule,
        derivation_source_fields,
        derivation_scenario,
        absence_event,
        absence_scenario,
    ) {
        (Some(raw_source_event), Some(raw_source_attribute), None, None, None, None, None) => {
            NewReadModelField::new(
                field_name,
                field_source_kind,
                parse_event_name(raw_source_event)
                    .map_err(|error| ShellError::message(error.to_string()))?,
                parse_event_attribute_name(raw_source_attribute)
                    .map_err(|error| ShellError::message(error.to_string()))?,
                provenance_description,
            )
        }
        (
            None,
            None,
            Some(raw_derivation_rule),
            Some(raw_derivation_source_fields),
            Some(raw_derivation_scenario),
            None,
            None,
        ) => NewReadModelField::new_derivation(
            field_name,
            field_source_kind,
            parse_read_model_derivation_rule(raw_derivation_rule)
                .map_err(|error| ShellError::message(error.to_string()))?,
            ReadModelDerivationSourceFields::from_fields(
                parse_datum_names(raw_derivation_source_fields)
                    .map_err(|error| ShellError::message(error.to_string()))?,
            ),
            parse_scenario_name(raw_derivation_scenario)
                .map_err(|error| ShellError::message(error.to_string()))?,
            provenance_description,
        ),
        (None, None, None, None, None, Some(raw_absence_event), Some(raw_absence_scenario)) => {
            NewReadModelField::new_absence_default(
                field_name,
                field_source_kind,
                parse_event_name(raw_absence_event)
                    .map_err(|error| ShellError::message(error.to_string()))?,
                parse_scenario_name(raw_absence_scenario)
                    .map_err(|error| ShellError::message(error.to_string()))?,
                provenance_description,
            )
        }
        (Some(_), None, _, _, _, _, _) | (None, Some(_), _, _, _, _, _) => {
            return Err(ShellError::message(
                "add_read_model_definition requires source_event and source_attribute together",
            ));
        }
        (_, _, Some(_), None, _, _, _)
        | (_, _, None, Some(_), _, _, _)
        | (_, _, Some(_), _, None, _, _)
        | (_, _, None, _, Some(_), _, _) => {
            return Err(ShellError::message(
                "add_read_model_definition requires derivation_rule, derivation_source_fields, and derivation_scenario together",
            ));
        }
        (_, _, _, _, _, Some(_), None) | (_, _, _, _, _, None, Some(_)) => {
            return Err(ShellError::message(
                "add_read_model_definition requires absence_event and absence_scenario together",
            ));
        }
        _ => {
            return Err(ShellError::message(
                "add_read_model_definition requires source_event/source_attribute, derivation_rule/derivation_source_fields/derivation_scenario, or absence_event/absence_scenario",
            ));
        }
    };

    let read_model = NewReadModelDefinition::new(slice_slug, read_model_name, read_model_field);
    let read_model = if arguments
        .get("transitive")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        let relationship_fields = arguments
            .get("relationship_fields")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ShellError::message(
                    "add_read_model_definition requires relationship_fields for transitive read models",
                )
            })
            .and_then(|raw_relationship_fields| {
                parse_datum_names(raw_relationship_fields)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
        let transitive_rule = arguments
            .get("transitive_rule")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ShellError::message(
                    "add_read_model_definition requires transitive_rule for transitive read models",
                )
            })
            .and_then(|raw_transitive_rule| {
                parse_read_model_transitive_rule(raw_transitive_rule)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
        let example_scenario = arguments
            .get("example_scenario")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                ShellError::message(
                    "add_read_model_definition requires example_scenario for transitive read models",
                )
            })
            .and_then(|raw_example_scenario| {
                parse_scenario_name(raw_example_scenario)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
        read_model.with_transitive_semantics(
            ReadModelRelationshipFields::from_fields(relationship_fields),
            transitive_rule,
            example_scenario,
        )
    } else {
        read_model
    };

    interpret_collect_reports(command::add_read_model_definition(read_model))
        .map(|reports| reports.join("\n"))
}

fn add_view_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_view_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let view_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires name"))
        .and_then(|raw_name| {
            parse_view_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let read_model_name = arguments
        .get("read_model")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires read_model"))
        .and_then(|raw_read_model| {
            parse_read_model_name(raw_read_model)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let field_name = arguments
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires field"))
        .and_then(|raw_field| {
            parse_view_field_name(raw_field).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_field = arguments
        .get("source_field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires source_field"))
        .and_then(|raw_source_field| {
            parse_view_field_name(raw_source_field)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let sketch_token = arguments
        .get("sketch_token")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires sketch_token"))
        .and_then(|raw_sketch_token| {
            parse_sketch_token(raw_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let provenance_description = arguments
        .get("field_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires field_provenance"))
        .and_then(|raw_provenance| {
            parse_provenance_description(raw_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let bit_encoding = arguments
        .get("bit_encoding")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires bit_encoding"))
        .and_then(|raw_bit_encoding| {
            parse_bit_encoding_semantics(raw_bit_encoding)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_name = arguments
        .get("control")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires control"))
        .and_then(|raw_control| {
            parse_control_name(raw_control).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_command = arguments
        .get("control_command")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires control_command"))
        .and_then(|raw_command| {
            parse_command_name(raw_command).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_input = arguments
        .get("control_input")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires control_input"))
        .and_then(|raw_input| {
            parse_datum_name(raw_input).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_input_source = arguments
        .get("control_input_source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires control_input_source"))
        .and_then(|raw_source| {
            parse_command_input_source_kind(raw_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_input_description = arguments
        .get("control_input_description")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_view_definition requires control_input_description")
        })
        .and_then(|raw_description| {
            parse_command_input_source_description(raw_description)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_input_sketch_token = arguments
        .get("control_input_sketch_token")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_view_definition requires control_input_sketch_token")
        })
        .and_then(|raw_sketch_token| {
            parse_sketch_token(raw_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_input_visible = arguments
        .get("control_input_visible")
        .and_then(Value::as_bool)
        .ok_or_else(|| ShellError::message("add_view_definition requires control_input_visible"))?;
    let control_input_decision = arguments
        .get("control_input_decision")
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            ShellError::message("add_view_definition requires control_input_decision")
        })?;
    let handled_errors = arguments
        .get("handled_errors")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires handled_errors"))
        .and_then(|raw_errors| {
            parse_command_error_names(raw_errors)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let recovery_behavior = arguments
        .get("recovery_behavior")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires recovery_behavior"))
        .and_then(|raw_recovery| {
            parse_control_recovery_behavior(raw_recovery)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_sketch_token = arguments
        .get("control_sketch_token")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires control_sketch_token"))
        .and_then(|raw_sketch_token| {
            parse_sketch_token(raw_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let navigation_type = arguments
        .get("navigation_type")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires navigation_type"))
        .and_then(|raw_navigation_type| {
            parse_navigation_target_type(raw_navigation_type)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let navigation_target = arguments
        .get("navigation_target")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("add_view_definition requires navigation_target"))
        .and_then(|raw_navigation_target| {
            parse_navigation_target_name(raw_navigation_target)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let external_workflow = arguments.get("external_workflow").and_then(Value::as_str);
    let external_system = arguments.get("external_system").and_then(Value::as_str);
    let handoff_contract = arguments.get("handoff_contract").and_then(Value::as_str);
    let navigation = match (external_workflow, external_system, handoff_contract) {
        (Some(raw_external_workflow), None, None) => {
            let external_workflow = parse_navigation_target_name(raw_external_workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            NewNavigationTarget::new(navigation_type, navigation_target)
                .with_external_workflow(external_workflow)
        }
        (None, Some(raw_external_system), Some(raw_handoff_contract)) => {
            let external_system = parse_navigation_target_name(raw_external_system)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let handoff_contract = parse_payload_contract_name(raw_handoff_contract)
                .map_err(|error| ShellError::message(error.to_string()))?;
            NewNavigationTarget::new(navigation_type, navigation_target)
                .with_external_system(external_system, handoff_contract)
        }
        (None, None, None) => NewNavigationTarget::new(navigation_type, navigation_target),
        _ => {
            return Err(ShellError::message(
                "add_view_definition requires either external_workflow alone or external_system and handoff_contract together",
            ));
        }
    };
    let local_states = arguments
        .get("local_states")
        .and_then(Value::as_str)
        .map(parse_navigation_target_names)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let filters = arguments
        .get("filters")
        .and_then(Value::as_str)
        .map(parse_navigation_target_names)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;

    interpret_collect_reports(command::add_view_definition(
        NewViewDefinition::new(
            slice_slug,
            view_name,
            NewViewField::new(
                field_name,
                parse_view_field_source_kind("read_model")
                    .map_err(|error| ShellError::message(error.to_string()))?,
                read_model_name,
                source_field,
                sketch_token,
                provenance_description,
                bit_encoding,
            ),
        )
        .with_local_states(ViewLocalStates::from_targets(
            local_states.unwrap_or_default(),
        ))
        .with_filters(ViewFilters::from_targets(filters.unwrap_or_default()))
        .with_controls(ViewControls::from_controls([NewControlDefinition::new(
            control_name,
            control_command,
            NewControlInputProvision::new(
                control_input,
                control_input_source,
                control_input_description,
                control_input_sketch_token,
                control_input_visible,
                control_input_decision,
            ),
            CommandErrorNames::from_names(handled_errors),
            recovery_behavior,
            control_sketch_token,
            navigation,
        )])),
    ))
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

fn remove_slice_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_slice requires arguments"))?;
    let slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_slice requires slug"))
        .and_then(|raw_slug| {
            parse_slice_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::remove_slice(slug)).map(|reports| reports.join("\n"))
}

fn remove_workflow_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_workflow requires arguments"))?;
    let slug = arguments
        .get("slug")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_workflow requires slug"))
        .and_then(|raw_slug| {
            parse_workflow_slug(raw_slug).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(command::remove_workflow(slug)).map(|reports| reports.join("\n"))
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
        if let Some(raw_payload_contract) =
            arguments.get("payload_contract").and_then(Value::as_str)
        {
            let payload_contract = parse_payload_contract_name(raw_payload_contract)
                .map_err(|error| ShellError::message(error.to_string()))?;
            WorkflowConnection::new_with_payload_contract(
                workflow_slug,
                source_slug,
                target_slug,
                connection_kind,
                trigger,
                payload_contract,
            )
        } else {
            WorkflowConnection::new(
                workflow_slug,
                source_slug,
                target_slug,
                connection_kind,
                trigger,
            )
        }
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

fn remove_transition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_transition requires arguments"))?;
    let workflow_slug = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_transition requires workflow"))
        .and_then(|raw_workflow| {
            parse_workflow_slug(raw_workflow)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let source_slug = arguments
        .get("from")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_transition requires from"))
        .and_then(|raw_source| {
            parse_slice_slug(raw_source).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let connection_kind = arguments
        .get("via")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_transition requires via"))
        .and_then(|raw_via| {
            parse_connection_kind(raw_via).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let trigger = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_transition requires name"))
        .and_then(|raw_name| {
            parse_transition_trigger_name(raw_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let removal = if let Some(raw_target) = arguments.get("to").and_then(Value::as_str) {
        let target_slug =
            parse_slice_slug(raw_target).map_err(|error| ShellError::message(error.to_string()))?;
        WorkflowTransitionRemoval::new(
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
            .ok_or_else(|| ShellError::message("remove_transition requires to or to_workflow"))
            .and_then(|raw_target| {
                parse_workflow_slug(raw_target)
                    .map_err(|error| ShellError::message(error.to_string()))
            })?;
        WorkflowTransitionRemoval::new_workflow_exit(
            workflow_slug,
            source_slug,
            target_workflow,
            connection_kind,
            trigger,
        )
    };
    interpret_collect_reports(command::remove_transition(removal)).map(|reports| reports.join("\n"))
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
