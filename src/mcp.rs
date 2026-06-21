// Copyright 2026 John Wilger

use std::fmt::Display;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{TcpListener, TcpStream};

use emc::modeling_process_guide;
use rmcp::model::{
    CallToolResult, Content, Implementation, InitializeResult, JsonObject, ListToolsResult,
    ServerCapabilities, Tool, ToolsCapability,
};
use serde::Serialize;
use serde_json::{Value, json};

const DEFAULT_MCP_PROTOCOL_VERSION: &str = "2025-11-25";
const SUPPORTED_MCP_PROTOCOL_VERSIONS: &[&str] = &["2025-11-25", "2025-06-18", "2024-11-05"];

use crate::command;
use crate::core::connection::{ConnectionKind, WorkflowConnection, WorkflowTransitionRemoval};
use crate::core::effect::{ArtifactDigest, ChosenEventId, EventConflictId};
use crate::core::formal_slice_facts::{
    CommandErrorDefinitions, CommandErrorNames, CommandInputProvenanceChain, CommandInputSource,
    CommandObservedStreams, EmittedEventNames, NewAutomationDefinition, NewBitLevelDataFlow,
    NewBoardConnection, NewBoardElement, NewCommandDefinition, NewCommandErrorDefinition,
    NewCommandInput, NewControlDefinition, NewControlInputProvision, NewEventAttribute,
    NewEventDefinition, NewExternalPayloadDefinition, NewNavigationTarget, NewOutcomeDefinition,
    NewReadModelDefinition, NewReadModelField, NewSliceScenario, NewTranslationDefinition,
    NewViewDefinition, NewViewField, OutcomeEventNames, ReadModelDerivationSourceFields,
    ReadModelFieldSource, ReadModelRelationshipFields, ScenarioKind, ScenarioStreamNames,
    ViewControls, ViewFilters, ViewLocalStates,
};
use crate::core::modeling_enums::MODELING_ENUMS;
use crate::core::slice::NewSlice;
use crate::core::types::{
    BoardConnectionEndpoint, BoardConnectionEndpointKind, CommandInputSourceDescription,
    CommandInputSourceKind, CommandName, DatumName, EventAttributeName, EventAttributeSourceField,
    EventAttributeSourceKind, EventAttributeSourceName, EventName,
    GeneratedEventAttributeSourceKind, ModelDescription, NavigationTargetName,
    ProvenanceDescription, ReadModelFieldSourceKind, SingletonRepeatBehavior, SliceSlug,
    SourceChainHop, StreamName, TransitionTriggerName, ViewName, WorkflowCommandErrorRecord,
    WorkflowEntryLifecycleStateRecord, WorkflowEventParticipation, WorkflowOutcomeRecord,
    WorkflowOwnedDefinitionKind, WorkflowOwnedDefinitionName, WorkflowOwnedDefinitionRecord,
    WorkflowSlug, WorkflowTransitionEndpoint, WorkflowTransitionEvidenceNavigationEndpoints,
    WorkflowTransitionEvidenceRecord, WorkflowTransitionKind, WorkflowTransitionSourceEvidenceText,
    WorkflowTransitionTargetEvidenceText, WorkflowViewRole,
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
    parse_data_flow_source_kind, parse_data_flow_target, parse_datum_name, parse_datum_names,
    parse_event_attribute_name, parse_event_attribute_source_field,
    parse_event_attribute_source_kind, parse_event_attribute_source_name, parse_event_name,
    parse_event_names, parse_generated_event_attribute_source_kind, parse_model_description,
    parse_model_name, parse_navigation_target_name, parse_navigation_target_names,
    parse_navigation_target_type, parse_outcome_label_name, parse_payload_contract_name,
    parse_project_name, parse_provenance_description, parse_read_model_derivation_rule,
    parse_read_model_field_source_kind, parse_read_model_name, parse_read_model_transitive_rule,
    parse_review_timestamp, parse_reviewer_id, parse_scenario_kind, parse_scenario_name,
    parse_scenario_step_text, parse_singleton_repeat_behavior, parse_sketch_token,
    parse_slice_kind, parse_slice_slug, parse_source_chain_hops, parse_stream_name,
    parse_stream_names, parse_transformation_semantics, parse_transition_trigger_name,
    parse_translation_external_event_name, parse_translation_name, parse_view_field_name,
    parse_view_field_source_kind, parse_view_name, parse_workflow_entry_lifecycle_evidence_text,
    parse_workflow_entry_lifecycle_state_name, parse_workflow_event_participation,
    parse_workflow_owned_definition_kind, parse_workflow_owned_definition_name,
    parse_workflow_slug, parse_workflow_transition_endpoint, parse_workflow_transition_kind,
    parse_workflow_transition_source_evidence_text, parse_workflow_transition_target_evidence_text,
    parse_workflow_view_role,
};
use crate::shell::{ShellError, interpret_collect_reports};

pub(crate) fn serve_stdio() -> Result<(), ShellError> {
    let stdin = io::stdin();
    stdin.lock().lines().try_for_each(|line| {
        let response = line
            .map_err(|error| ShellError::message(error.to_string()))
            .and_then(|line| handle_input_line(&line))?;
        response.map_or(Ok(()), |response| write_response(&response))
    })
}

pub(crate) fn serve_http(
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
            let stream = stream.map_err(|error| ShellError::message(error.to_string()))?;
            handle_http_stream(stream, &authority, &auth_policy)
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
    let response = http_response_for_request(&request, authority, auth_policy)?;
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
    request: &HttpRequest,
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
        Some("initialize") => Ok(Some(success_response(id, &initialize_result(request)?))),
        Some("tools/list") => Ok(Some(success_response(id, &tools_list_result()?))),
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

fn initialize_result(request: &Value) -> Result<Value, ShellError> {
    let mut capabilities = ServerCapabilities::default();
    capabilities.tools = Some(ToolsCapability::default());
    let mut result = mcp_model_value(
        InitializeResult::new(capabilities)
            .with_server_info(Implementation::new("emc", env!("CARGO_PKG_VERSION"))),
    )?;
    result
        .as_object_mut()
        .ok_or_else(|| ShellError::message("MCP initialize result is not a JSON object"))?
        .insert(
            "protocolVersion".to_owned(),
            Value::String(initialize_protocol_version(request).to_owned()),
        );
    Ok(result)
}

fn initialize_protocol_version(request: &Value) -> &str {
    request
        .get("params")
        .and_then(|params| params.get("protocolVersion"))
        .and_then(Value::as_str)
        .filter(|requested| SUPPORTED_MCP_PROTOCOL_VERSIONS.contains(requested))
        .unwrap_or(DEFAULT_MCP_PROTOCOL_VERSION)
}

fn tools_list_result() -> Result<Value, ShellError> {
    let mut tools = Vec::new();
    tools.extend(project_lifecycle_tools());
    tools.extend(project_query_tools());
    tools.extend(project_status_tools());
    tools.extend(workflow_composition_tools());
    tools.extend(workflow_evidence_tools());
    tools.extend(slice_structure_tools());
    tools.extend(slice_definition_tools());
    tools.extend(model_mutation_tools());
    mcp_model_value(ListToolsResult::with_all_items(tools))
}

fn project_lifecycle_tools() -> Vec<Tool> {
    vec![
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
    ]
}

fn project_query_tools() -> Vec<Tool> {
    vec![
        Tool::new(
            "list_conflicts",
            "List unresolved exported event conflicts.",
            schema_object(json!({
                    "type": "object",
                    "properties": {},
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "resolve_conflict",
            "Resolve an exported event conflict by choosing one event.",
            schema_object(json!({
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "string"
                        },
                        "choose_event": {
                            "type": "string"
                        }
                    },
                    "required": ["id", "choose_event"],
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
    ]
}

fn project_status_tools() -> Vec<Tool> {
    vec![
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
            "list_modeling_enums",
            "List accepted modeled enum strings for constrained EMC tool arguments.",
            schema_object(json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
            })),
        ),
        Tool::new(
            "get_modeling_guidance",
            "Return the full EMC modeling process guide.",
            schema_object(json!({
                    "type": "object",
                    "properties": {},
                    "required": [],
                    "additionalProperties": false
            })),
        ),
    ]
}

fn workflow_composition_tools() -> Vec<Tool> {
    vec![
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
    ]
}

fn workflow_evidence_tools() -> Vec<Tool> {
    vec![
        add_workflow_owned_definition_tool(),
        add_workflow_transition_evidence_tool(),
        require_workflow_entry_lifecycle_coverage_tool(),
        add_workflow_entry_lifecycle_state_tool(),
    ]
}

fn add_workflow_owned_definition_tool() -> Tool {
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
                    },
                    "event_participation": {
                        "type": "string"
                    },
                    "view_role": {
                        "type": "string"
                    }
                },
                "required": ["workflow", "source_slice", "definition_kind", "definition_name"],
                "additionalProperties": false
        })),
    )
}

fn add_workflow_transition_evidence_tool() -> Tool {
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
                    "source_control": {
                        "type": "string",
                        "description": "Required when via is navigation; source-slice-owned control that initiates navigation."
                    },
                    "target_view": {
                        "type": "string",
                        "description": "Required when via is navigation; target-slice-owned entry view reached by navigation."
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
    )
}

fn require_workflow_entry_lifecycle_coverage_tool() -> Tool {
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
    )
}

fn add_workflow_entry_lifecycle_state_tool() -> Tool {
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
    )
}

fn slice_structure_tools() -> Vec<Tool> {
    vec![
        add_slice_tool(),
        add_slice_scenario_tool(),
        add_bit_level_data_flow_tool(),
        add_board_element_tool(),
        add_board_connection_tool(),
    ]
}

fn add_slice_tool() -> Tool {
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
    )
}

fn add_slice_scenario_tool() -> Tool {
    Tool::new(
        "add_slice_scenario",
        "Add an acceptance or contract GWT scenario directly to Lean4 and Quint slice artifacts.",
        slice_scenario_schema(),
    )
}

fn slice_scenario_schema() -> JsonObject {
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
    }))
}

fn add_bit_level_data_flow_tool() -> Tool {
    Tool::new(
        "add_bit_level_data_flow",
        "Add source, transformation, target, and bit-encoding semantics directly to Lean4 and Quint slice artifacts.",
        bit_level_data_flow_schema(),
    )
}

fn update_bit_level_data_flow_tool() -> Tool {
    Tool::new(
        "update_bit_level_data_flow",
        "Update source, transformation, target, and bit-encoding semantics in Lean4 and Quint slice artifacts.",
        bit_level_data_flow_update_schema(),
    )
}

fn remove_bit_level_data_flow_tool() -> Tool {
    Tool::new(
        "remove_bit_level_data_flow",
        "Remove source, transformation, target, and bit-encoding semantics from Lean4 and Quint slice artifacts.",
        bit_level_data_flow_schema(),
    )
}

fn bit_level_data_flow_schema() -> JsonObject {
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
                "source_kind": {
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
            "required": ["slice", "datum", "source", "source_kind", "transformation", "target", "bit_encoding"],
            "additionalProperties": false
    }))
}

fn bit_level_data_flow_update_schema() -> JsonObject {
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
                "source_kind": {
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
                },
                "new_datum": {
                    "type": "string"
                },
                "new_source": {
                    "type": "string"
                },
                "new_source_kind": {
                    "type": "string"
                },
                "new_transformation": {
                    "type": "string"
                },
                "new_target": {
                    "type": "string"
                },
                "new_bit_encoding": {
                    "type": "string"
                }
            },
            "required": ["slice", "datum", "source", "source_kind", "transformation", "target", "bit_encoding", "new_datum", "new_source", "new_source_kind", "new_transformation", "new_target", "new_bit_encoding"],
            "additionalProperties": false
    }))
}

fn add_board_element_tool() -> Tool {
    Tool::new(
        "add_board_element",
        "Add a board element causal-shape fact directly to Lean4 and Quint slice artifacts.",
        board_element_schema(),
    )
}

fn update_board_element_tool() -> Tool {
    Tool::new(
        "update_board_element",
        "Update a board element causal-shape fact in Lean4 and Quint slice artifacts.",
        board_element_schema(),
    )
}

fn remove_board_element_tool() -> Tool {
    Tool::new(
        "remove_board_element",
        "Remove a board element causal-shape fact from Lean4 and Quint slice artifacts.",
        schema_object(json!({
                "type": "object",
                "properties": {
                    "slice": {
                        "type": "string"
                    },
                    "name": {
                        "type": "string"
                    }
                },
                "required": ["slice", "name"],
                "additionalProperties": false
        })),
    )
}

fn board_element_schema() -> JsonObject {
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
    }))
}

fn add_board_connection_tool() -> Tool {
    Tool::new(
        "add_board_connection",
        "Add a board connection causal-shape fact directly to Lean4 and Quint slice artifacts.",
        board_connection_schema(),
    )
}

fn update_board_connection_tool() -> Tool {
    Tool::new(
        "update_board_connection",
        "Update a board connection causal-shape fact in Lean4 and Quint slice artifacts.",
        board_connection_update_schema(),
    )
}

fn remove_board_connection_tool() -> Tool {
    Tool::new(
        "remove_board_connection",
        "Remove a board connection causal-shape fact from Lean4 and Quint slice artifacts.",
        board_connection_schema(),
    )
}

fn board_connection_schema() -> JsonObject {
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
    }))
}

fn board_connection_update_schema() -> JsonObject {
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
                },
                "new_source": {
                    "type": "string"
                },
                "new_source_kind": {
                    "type": "string"
                },
                "new_target": {
                    "type": "string"
                },
                "new_target_kind": {
                    "type": "string"
                }
            },
            "required": [
                "slice",
                "source",
                "source_kind",
                "target",
                "target_kind",
                "new_source",
                "new_source_kind",
                "new_target",
                "new_target_kind"
            ],
            "additionalProperties": false
    }))
}

fn slice_definition_tools() -> Vec<Tool> {
    vec![
        add_command_definition_tool(),
        add_automation_definition_tool(),
        add_translation_definition_tool(),
        add_external_payload_definition_tool(),
        add_event_definition_tool(),
        add_outcome_definition_tool(),
        add_read_model_definition_tool(),
        add_view_definition_tool(),
    ]
}

fn add_command_definition_tool() -> Tool {
    Tool::new(
        "add_command_definition",
        "Add a command, input provenance, and emitted events directly to Lean4 and Quint slice artifacts.",
        schema_object(add_command_definition_schema()),
    )
}

fn update_command_definition_tool() -> Tool {
    Tool::new(
        "update_command_definition",
        "Update a command, input provenance, and emitted events in a slice, then regenerate synchronized model artifacts.",
        schema_object(add_command_definition_schema()),
    )
}

fn remove_command_definition_tool() -> Tool {
    Tool::new(
        "remove_command_definition",
        "Remove a command definition from a slice, then regenerate synchronized model artifacts.",
        schema_object(json!({
            "type": "object",
            "properties": {
                "slice": {
                    "type": "string"
                },
                "name": {
                    "type": "string"
                }
            },
            "required": ["slice", "name"],
            "additionalProperties": false
        })),
    )
}

fn add_command_definition_schema() -> Value {
    json!({
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
            "source_name": {
                "type": "string"
            },
            "source_session": {
                "type": "string"
            },
            "source_argument": {
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
    })
}

fn add_automation_definition_tool() -> Tool {
    Tool::new(
        "add_automation_definition",
        "Add an automation trigger, issued command, handled errors, and reaction semantics directly to Lean4 and Quint slice artifacts.",
        schema_object(automation_definition_schema()),
    )
}

fn update_automation_definition_tool() -> Tool {
    Tool::new(
        "update_automation_definition",
        "Update an automation trigger, issued command, handled errors, and reaction semantics in a slice, then regenerate synchronized model artifacts.",
        schema_object(automation_definition_schema()),
    )
}

fn remove_automation_definition_tool() -> Tool {
    Tool::new(
        "remove_automation_definition",
        "Remove an automation definition from a slice, then regenerate synchronized model artifacts.",
        schema_object(slice_named_definition_schema()),
    )
}

fn automation_definition_schema() -> Value {
    json!({
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
    })
}

fn add_translation_definition_tool() -> Tool {
    Tool::new(
        "add_translation_definition",
        "Add a translation external event, payload contract, and target command directly to Lean4 and Quint slice artifacts.",
        schema_object(translation_definition_schema()),
    )
}

fn update_translation_definition_tool() -> Tool {
    Tool::new(
        "update_translation_definition",
        "Update a translation external event, payload contract, and target command in a slice, then regenerate synchronized model artifacts.",
        schema_object(translation_definition_schema()),
    )
}

fn remove_translation_definition_tool() -> Tool {
    Tool::new(
        "remove_translation_definition",
        "Remove a translation definition from a slice, then regenerate synchronized model artifacts.",
        schema_object(slice_named_definition_schema()),
    )
}

fn translation_definition_schema() -> Value {
    json!({
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
    })
}

fn add_external_payload_definition_tool() -> Tool {
    Tool::new(
        "add_external_payload_definition",
        "Add an external payload field contract directly to Lean4 and Quint slice artifacts.",
        schema_object(external_payload_definition_schema()),
    )
}

fn update_external_payload_definition_tool() -> Tool {
    Tool::new(
        "update_external_payload_definition",
        "Update an external payload field contract in a slice, then regenerate synchronized model artifacts.",
        schema_object(external_payload_definition_schema()),
    )
}

fn remove_external_payload_definition_tool() -> Tool {
    Tool::new(
        "remove_external_payload_definition",
        "Remove an external payload field contract from a slice, then regenerate synchronized model artifacts.",
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
                }
            },
            "required": ["slice", "name", "field"],
            "additionalProperties": false
        })),
    )
}

fn external_payload_definition_schema() -> Value {
    json!({
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
    })
}

fn add_event_definition_tool() -> Tool {
    Tool::new(
        "add_event_definition",
        "Add an event, stream, attribute source, and provenance directly to Lean4 and Quint slice artifacts.",
        schema_object(event_definition_schema()),
    )
}

fn update_event_definition_tool() -> Tool {
    Tool::new(
        "update_event_definition",
        "Update an event, stream, attribute source, and provenance in a slice, then regenerate synchronized model artifacts.",
        schema_object(event_definition_schema()),
    )
}

fn remove_event_definition_tool() -> Tool {
    Tool::new(
        "remove_event_definition",
        "Remove an event definition from a slice, then regenerate synchronized model artifacts.",
        schema_object(json!({
            "type": "object",
            "properties": {
                "slice": {
                    "type": "string"
                },
                "name": {
                    "type": "string"
                }
            },
            "required": ["slice", "name"],
            "additionalProperties": false
        })),
    )
}

fn event_definition_schema() -> Value {
    json!({
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
            "generated_source_kind": {
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
    })
}

fn add_outcome_definition_tool() -> Tool {
    Tool::new(
        "add_outcome_definition",
        "Add an outcome label and backing event set directly to Lean4 and Quint slice artifacts.",
        schema_object(outcome_definition_schema()),
    )
}

fn update_outcome_definition_tool() -> Tool {
    Tool::new(
        "update_outcome_definition",
        "Update an outcome label and backing event set in a slice, then regenerate synchronized model artifacts.",
        schema_object(outcome_definition_schema()),
    )
}

fn remove_outcome_definition_tool() -> Tool {
    Tool::new(
        "remove_outcome_definition",
        "Remove an outcome definition from a slice, then regenerate synchronized model artifacts.",
        schema_object(slice_outcome_label_schema()),
    )
}

fn outcome_definition_schema() -> Value {
    json!({
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
    })
}

fn slice_outcome_label_schema() -> Value {
    json!({
            "type": "object",
            "properties": {
                "slice": {
                    "type": "string"
                },
                "label": {
                    "type": "string"
                }
            },
            "required": ["slice", "label"],
            "additionalProperties": false
    })
}

fn add_read_model_definition_tool() -> Tool {
    Tool::new(
        "add_read_model_definition",
        "Add a read model, field source, and field provenance directly to Lean4 and Quint slice artifacts.",
        schema_object(read_model_definition_schema()),
    )
}

fn update_read_model_definition_tool() -> Tool {
    Tool::new(
        "update_read_model_definition",
        "Update a read model, field source, and field provenance in a slice, then regenerate synchronized model artifacts.",
        schema_object(read_model_definition_schema()),
    )
}

fn remove_read_model_definition_tool() -> Tool {
    Tool::new(
        "remove_read_model_definition",
        "Remove a read model definition from a slice, then regenerate synchronized model artifacts.",
        schema_object(slice_named_definition_schema()),
    )
}

fn read_model_definition_schema() -> Value {
    json!({
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
    })
}

fn slice_named_definition_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "slice": {
                "type": "string"
            },
            "name": {
                "type": "string"
            }
        },
        "required": ["slice", "name"],
        "additionalProperties": false
    })
}

fn add_view_definition_tool() -> Tool {
    Tool::new(
        "add_view_definition",
        "Add a view field plus command control, input provenance, error handling, and navigation directly to Lean4 and Quint slice artifacts.",
        schema_object(add_view_definition_schema()),
    )
}

fn update_view_definition_tool() -> Tool {
    Tool::new(
        "update_view_definition",
        "Update a view field plus command control, input provenance, error handling, and navigation in a slice, then regenerate synchronized model artifacts.",
        schema_object(add_view_definition_schema()),
    )
}

fn remove_view_definition_tool() -> Tool {
    Tool::new(
        "remove_view_definition",
        "Remove a view definition from a slice, then regenerate synchronized model artifacts.",
        schema_object(slice_named_definition_schema()),
    )
}

fn update_control_definition_tool() -> Tool {
    Tool::new(
        "update_control_definition",
        "Update a control on a slice view, then regenerate synchronized model artifacts.",
        schema_object(control_definition_schema()),
    )
}

fn remove_control_definition_tool() -> Tool {
    Tool::new(
        "remove_control_definition",
        "Remove a control from a slice view, then regenerate synchronized model artifacts.",
        schema_object(json!({
            "type": "object",
            "properties": {
                "slice": {
                    "type": "string"
                },
                "view": {
                    "type": "string"
                },
                "name": {
                    "type": "string"
                }
            },
            "required": ["slice", "view", "name"],
            "additionalProperties": false
        })),
    )
}

fn control_definition_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "slice": {
                "type": "string"
            },
            "view": {
                "type": "string"
            },
            "name": {
                "type": "string"
            },
            "command": {
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
            "input_sketch_token": {
                "type": "string"
            },
            "input_visible": {
                "type": "boolean"
            },
            "input_decision": {
                "type": "boolean"
            },
            "handled_errors": {
                "type": "string"
            },
            "recovery_behavior": {
                "type": "string"
            },
            "sketch_token": {
                "type": "string"
            },
            "navigation_type": {
                "type": "string"
            },
            "navigation_target": {
                "type": "string"
            }
        },
        "required": ["slice", "view", "name", "command", "input", "input_source", "input_description", "input_sketch_token", "input_visible", "input_decision", "handled_errors", "recovery_behavior", "sketch_token", "navigation_type", "navigation_target"],
        "additionalProperties": false
    })
}

fn add_view_definition_schema() -> Value {
    json!({
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
    })
}

fn model_mutation_tools() -> Vec<Tool> {
    vec![
        update_workflow_tool(),
        update_workflow_name_tool(),
        update_workflow_outcome_tool(),
        update_workflow_command_error_tool(),
        update_slice_tool(),
        update_slice_kind_tool(),
        update_slice_name_tool(),
        update_slice_scenario_tool(),
        update_automation_definition_tool(),
        update_translation_definition_tool(),
        update_external_payload_definition_tool(),
        update_bit_level_data_flow_tool(),
        update_board_element_tool(),
        update_board_connection_tool(),
        update_command_definition_tool(),
        update_event_definition_tool(),
        update_outcome_definition_tool(),
        update_read_model_definition_tool(),
        update_view_definition_tool(),
        update_control_definition_tool(),
        remove_slice_tool(),
        remove_slice_scenario_tool(),
        remove_automation_definition_tool(),
        remove_translation_definition_tool(),
        remove_external_payload_definition_tool(),
        remove_bit_level_data_flow_tool(),
        remove_board_element_tool(),
        remove_board_connection_tool(),
        remove_command_definition_tool(),
        remove_event_definition_tool(),
        remove_outcome_definition_tool(),
        remove_read_model_definition_tool(),
        remove_view_definition_tool(),
        remove_control_definition_tool(),
        remove_workflow_tool(),
        remove_workflow_outcome_tool(),
        remove_workflow_command_error_tool(),
        connect_workflow_tool(),
        remove_transition_tool(),
    ]
}

fn update_workflow_tool() -> Tool {
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
    )
}

fn update_workflow_name_tool() -> Tool {
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
    )
}

fn update_slice_tool() -> Tool {
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
    )
}

fn update_slice_kind_tool() -> Tool {
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
    )
}

fn update_slice_name_tool() -> Tool {
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
    )
}

fn remove_slice_tool() -> Tool {
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
    )
}

fn update_slice_scenario_tool() -> Tool {
    Tool::new(
        "update_slice_scenario",
        "Update an acceptance or contract GWT scenario and regenerate synchronized model artifacts.",
        slice_scenario_schema(),
    )
}

fn remove_slice_scenario_tool() -> Tool {
    Tool::new(
        "remove_slice_scenario",
        "Remove a GWT scenario and regenerate synchronized model artifacts.",
        schema_object(json!({
                "type": "object",
                "properties": {
                    "slice": {
                        "type": "string"
                    },
                    "name": {
                        "type": "string"
                    }
                },
                "required": ["slice", "name"],
                "additionalProperties": false
        })),
    )
}

fn update_workflow_outcome_tool() -> Tool {
    Tool::new(
        "update_workflow_outcome",
        "Update a workflow composition outcome fact in Lean4 and Quint workflow artifacts.",
        workflow_outcome_update_schema(),
    )
}

fn remove_workflow_outcome_tool() -> Tool {
    Tool::new(
        "remove_workflow_outcome",
        "Remove a workflow composition outcome fact from Lean4 and Quint workflow artifacts.",
        workflow_outcome_schema(),
    )
}

fn workflow_outcome_schema() -> JsonObject {
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
    }))
}

fn workflow_outcome_update_schema() -> JsonObject {
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
                },
                "new_source_slice": {
                    "type": "string"
                },
                "new_label": {
                    "type": "string"
                },
                "new_externally_relevant": {
                    "type": "boolean"
                }
            },
            "required": [
                "workflow",
                "source_slice",
                "label",
                "externally_relevant",
                "new_source_slice",
                "new_label",
                "new_externally_relevant"
            ],
            "additionalProperties": false
    }))
}

fn update_workflow_command_error_tool() -> Tool {
    Tool::new(
        "update_workflow_command_error",
        "Update a workflow composition command-local error fact in Lean4 and Quint workflow artifacts.",
        workflow_command_error_update_schema(),
    )
}

fn remove_workflow_command_error_tool() -> Tool {
    Tool::new(
        "remove_workflow_command_error",
        "Remove a workflow composition command-local error fact from Lean4 and Quint workflow artifacts.",
        workflow_command_error_schema(),
    )
}

fn workflow_command_error_schema() -> JsonObject {
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
    }))
}

fn workflow_command_error_update_schema() -> JsonObject {
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
                },
                "new_source_slice": {
                    "type": "string"
                },
                "new_command": {
                    "type": "string"
                },
                "new_error": {
                    "type": "string"
                }
            },
            "required": [
                "workflow",
                "source_slice",
                "command",
                "error",
                "new_source_slice",
                "new_command",
                "new_error"
            ],
            "additionalProperties": false
    }))
}

fn remove_workflow_tool() -> Tool {
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
    )
}

fn connect_workflow_tool() -> Tool {
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
                    "source_control": {
                        "type": "string",
                        "description": "Required when via is navigation; source-slice-owned control that initiates navigation."
                    },
                    "target_view": {
                        "type": "string",
                        "description": "Required when via is navigation; target-slice-owned entry view reached by navigation."
                    },
                    "reason": {
                        "type": "string"
                    },
                    "payload_contract": {
                        "type": "string"
                    }
                },
                "required": ["workflow", "from", "via", "name"],
                "additionalProperties": false
        })),
    )
}

fn remove_transition_tool() -> Tool {
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
                "additionalProperties": false
        })),
    )
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "return type mirrors the sibling arms of handle_request's match (which do return Err/None via ?), so the Result<Option<_>> shape is required for arm-type unification"
)]
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

    match dispatch_tool_text(name, request) {
        Some(result) => Ok(Some(tool_call_result_response(id, result))),
        None => Ok(Some(error_response(
            id,
            -32602,
            format!("unknown EMC MCP tool {name}"),
        ))),
    }
}

/// Routes a tool name to its text-producing handler, returning `None` for
/// unknown tools. Grouped by tool category so each sub-dispatcher stays small.
fn dispatch_tool_text(name: &str, request: &Value) -> Option<Result<String, ShellError>> {
    query_tool_text(name, request)
        .or_else(|| status_tool_text(name, request))
        .or_else(|| workflow_evidence_tool_text(name, request))
        .or_else(|| slice_structure_tool_text(name, request))
        .or_else(|| slice_definition_tool_text(name, request))
        .or_else(|| mutation_tool_text(name, request))
}

fn query_tool_text(name: &str, request: &Value) -> Option<Result<String, ShellError>> {
    match name {
        "init_project" => Some(init_project_tool_text(request)),
        "list_workflows" => Some(list_workflows_tool_text()),
        "list_slices" => Some(list_slices_tool_text()),
        "list_transitions" => Some(list_transitions_tool_text()),
        "list_conflicts" => Some(list_conflicts_tool_text()),
        "list_modeling_enums" => Some(list_modeling_enums_tool_text()),
        "get_modeling_guidance" => Some(Ok(get_modeling_guidance_tool_text())),
        "resolve_conflict" => Some(resolve_conflict_tool_text(request)),
        "show_workflow" => Some(show_workflow_tool_text(request)),
        "show_slice" => Some(show_slice_tool_text(request)),
        _ => None,
    }
}

fn status_tool_text(name: &str, request: &Value) -> Option<Result<String, ShellError>> {
    match name {
        "check_project" => Some(check_project_tool_text()),
        "verify_project" => Some(verify_project_tool_text()),
        "review_gate" => Some(review_gate_tool_text(request)),
        "record_clean_review" => Some(record_clean_review_tool_text(request)),
        _ => None,
    }
}

fn workflow_evidence_tool_text(name: &str, request: &Value) -> Option<Result<String, ShellError>> {
    match name {
        "add_workflow" => Some(add_workflow_tool_text(request)),
        "add_workflow_outcome" => Some(add_workflow_outcome_tool_text(request)),
        "add_workflow_command_error" => Some(add_workflow_command_error_tool_text(request)),
        "add_workflow_owned_definition" => Some(add_workflow_owned_definition_tool_text(request)),
        "add_workflow_transition_evidence" => {
            Some(add_workflow_transition_evidence_tool_text(request))
        }
        "require_workflow_entry_lifecycle_coverage" => {
            Some(require_workflow_entry_lifecycle_coverage_tool_text(request))
        }
        "add_workflow_entry_lifecycle_state" => {
            Some(add_workflow_entry_lifecycle_state_tool_text(request))
        }
        _ => None,
    }
}

fn slice_structure_tool_text(name: &str, request: &Value) -> Option<Result<String, ShellError>> {
    match name {
        "add_slice" => Some(add_slice_tool_text(request)),
        "add_slice_scenario" => Some(add_slice_scenario_tool_text(request)),
        "add_bit_level_data_flow" => Some(add_bit_level_data_flow_tool_text(request)),
        "add_board_element" => Some(add_board_element_tool_text(request)),
        "add_board_connection" => Some(add_board_connection_tool_text(request)),
        _ => None,
    }
}

fn slice_definition_tool_text(name: &str, request: &Value) -> Option<Result<String, ShellError>> {
    match name {
        "add_command_definition" => Some(add_command_definition_tool_text(request)),
        "add_automation_definition" => Some(add_automation_definition_tool_text(request)),
        "add_translation_definition" => Some(add_translation_definition_tool_text(request)),
        "add_event_definition" => Some(add_event_definition_tool_text(request)),
        "add_outcome_definition" => Some(add_outcome_definition_tool_text(request)),
        "add_external_payload_definition" => {
            Some(add_external_payload_definition_tool_text(request))
        }
        "add_read_model_definition" => Some(add_read_model_definition_tool_text(request)),
        "add_view_definition" => Some(add_view_definition_tool_text(request)),
        _ => None,
    }
}

fn mutation_tool_text(name: &str, request: &Value) -> Option<Result<String, ShellError>> {
    match name {
        "update_workflow" => Some(update_workflow_tool_text(request)),
        "update_workflow_name" => Some(update_workflow_name_tool_text(request)),
        "update_workflow_outcome" => Some(update_workflow_outcome_tool_text(request)),
        "update_workflow_command_error" => Some(update_workflow_command_error_tool_text(request)),
        "update_slice" => Some(update_slice_tool_text(request)),
        "update_slice_kind" => Some(update_slice_kind_tool_text(request)),
        "update_slice_name" => Some(update_slice_name_tool_text(request)),
        "update_slice_scenario" => Some(update_slice_scenario_tool_text(request)),
        "update_automation_definition" => Some(update_automation_definition_tool_text(request)),
        "update_translation_definition" => Some(update_translation_definition_tool_text(request)),
        "update_external_payload_definition" => {
            Some(update_external_payload_definition_tool_text(request))
        }
        "update_bit_level_data_flow" => Some(update_bit_level_data_flow_tool_text(request)),
        "update_board_element" => Some(update_board_element_tool_text(request)),
        "update_board_connection" => Some(update_board_connection_tool_text(request)),
        "update_command_definition" => Some(update_command_definition_tool_text(request)),
        "update_event_definition" => Some(update_event_definition_tool_text(request)),
        "update_outcome_definition" => Some(update_outcome_definition_tool_text(request)),
        "update_read_model_definition" => Some(update_read_model_definition_tool_text(request)),
        "update_view_definition" => Some(update_view_definition_tool_text(request)),
        "update_control_definition" => Some(update_control_definition_tool_text(request)),
        "remove_slice" => Some(remove_slice_tool_text(request)),
        "remove_slice_scenario" => Some(remove_slice_scenario_tool_text(request)),
        "remove_automation_definition" => Some(remove_automation_definition_tool_text(request)),
        "remove_translation_definition" => Some(remove_translation_definition_tool_text(request)),
        "remove_external_payload_definition" => {
            Some(remove_external_payload_definition_tool_text(request))
        }
        "remove_bit_level_data_flow" => Some(remove_bit_level_data_flow_tool_text(request)),
        "remove_board_element" => Some(remove_board_element_tool_text(request)),
        "remove_board_connection" => Some(remove_board_connection_tool_text(request)),
        "remove_command_definition" => Some(remove_command_definition_tool_text(request)),
        "remove_event_definition" => Some(remove_event_definition_tool_text(request)),
        "remove_outcome_definition" => Some(remove_outcome_definition_tool_text(request)),
        "remove_read_model_definition" => Some(remove_read_model_definition_tool_text(request)),
        "remove_view_definition" => Some(remove_view_definition_tool_text(request)),
        "remove_control_definition" => Some(remove_control_definition_tool_text(request)),
        "remove_workflow" => Some(remove_workflow_tool_text(request)),
        "remove_workflow_outcome" => Some(remove_workflow_outcome_tool_text(request)),
        "remove_workflow_command_error" => Some(remove_workflow_command_error_tool_text(request)),
        "connect_workflow" => Some(connect_workflow_tool_text(request)),
        "remove_transition" => Some(remove_transition_tool_text(request)),
        _ => None,
    }
}

fn tool_call_result_response(id: &Value, result: Result<String, ShellError>) -> Value {
    match result {
        Ok(text) => success_response(id, &tool_result(text)),
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
    interpret_collect_reports(&command::init(&name)).map(|reports| reports.join("\n"))
}

fn list_workflows_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(&command::list_workflows()).map(|reports| reports.join("\n"))
}

fn list_slices_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(&command::list_slices()).map(|reports| reports.join("\n"))
}

fn list_transitions_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(&command::list_transitions()).map(|reports| reports.join("\n"))
}

fn list_conflicts_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(&command::list_conflicts()).map(|reports| reports.join("\n"))
}

fn list_modeling_enums_tool_text() -> Result<String, ShellError> {
    serde_json::to_string_pretty(
        &MODELING_ENUMS
            .iter()
            .map(|modeled_enum| {
                json!({
                    "name": modeled_enum.name(),
                    "values": modeled_enum.values(),
                })
            })
            .collect::<Vec<_>>(),
    )
    .map_err(|error| ShellError::message(error.to_string()))
}

fn get_modeling_guidance_tool_text() -> String {
    modeling_process_guide().to_owned()
}

fn resolve_conflict_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("resolve_conflict requires arguments"))?;
    let conflict_id = arguments
        .get("id")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("resolve_conflict requires id"))?;
    let chosen_event_id = arguments
        .get("choose_event")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("resolve_conflict requires choose_event"))?;
    interpret_collect_reports(&command::resolve_conflict(
        parse_event_conflict_id(conflict_id)?,
        parse_chosen_event_id(chosen_event_id)?,
    ))
    .map(|reports| reports.join("\n"))
}

fn parse_event_conflict_id(value: &str) -> Result<EventConflictId, ShellError> {
    parse_artifact_digest("event conflict id", value).map(EventConflictId::new)
}

fn parse_chosen_event_id(value: &str) -> Result<ChosenEventId, ShellError> {
    parse_artifact_digest("chosen event id", value).map(ChosenEventId::new)
}

fn parse_artifact_digest(label: &str, value: &str) -> Result<ArtifactDigest, ShellError> {
    ArtifactDigest::try_new(value.to_owned())
        .map_err(|error| ShellError::message(format!("invalid {label}: {error}")))
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
    interpret_collect_reports(&command::show_workflow(slug)).map(|reports| reports.join("\n"))
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
    interpret_collect_reports(&command::show_slice(slug)).map(|reports| reports.join("\n"))
}

fn check_project_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(&command::check_project()).map(|reports| reports.join("\n"))
}

fn verify_project_tool_text() -> Result<String, ShellError> {
    interpret_collect_reports(&command::verify()).map(|reports| reports.join("\n"))
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
    interpret_collect_reports(&command::review_gate_for_workflow(workflow_slug))
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
    interpret_collect_reports(&command::record_clean_review(
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
    interpret_collect_reports(&command::add_workflow(NewWorkflow::new(
        name,
        description,
        slug,
    )))
    .map(|reports| reports.join("\n"))
}

fn add_workflow_outcome_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "add_workflow_outcome")?;
    let workflow_slug = workflow_slug_from_arguments(arguments, "add_workflow_outcome")?;

    interpret_collect_reports(&command::add_workflow_outcome(
        workflow_slug,
        workflow_outcome_from_arguments(arguments, "add_workflow_outcome", "")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn update_workflow_outcome_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "update_workflow_outcome")?;
    let workflow_slug = workflow_slug_from_arguments(arguments, "update_workflow_outcome")?;

    interpret_collect_reports(&command::update_workflow_outcome(
        workflow_slug,
        workflow_outcome_from_arguments(arguments, "update_workflow_outcome", "")?,
        workflow_outcome_from_arguments(arguments, "update_workflow_outcome", "new_")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn remove_workflow_outcome_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "remove_workflow_outcome")?;
    let workflow_slug = workflow_slug_from_arguments(arguments, "remove_workflow_outcome")?;

    interpret_collect_reports(&command::remove_workflow_outcome(
        workflow_slug,
        workflow_outcome_from_arguments(arguments, "remove_workflow_outcome", "")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn workflow_slug_from_arguments(
    arguments: &Value,
    tool_name: &str,
) -> Result<WorkflowSlug, ShellError> {
    let raw_workflow = arguments
        .get("workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires workflow")))?;
    parse_workflow_slug(raw_workflow).map_err(|error| ShellError::message(error.to_string()))
}

fn workflow_outcome_from_arguments(
    arguments: &Value,
    tool_name: &str,
    prefix: &str,
) -> Result<WorkflowOutcomeRecord, ShellError> {
    let source_slice_field = format!("{prefix}source_slice");
    let label_field = format!("{prefix}label");
    let externally_relevant_field = format!("{prefix}externally_relevant");
    let source_slice = arguments
        .get(&source_slice_field)
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires {source_slice_field}")))
        .and_then(|raw_source_slice| {
            WorkflowTransitionEndpoint::try_new(raw_source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let label = arguments
        .get(&label_field)
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires {label_field}")))
        .and_then(|raw_label| {
            parse_outcome_label_name(raw_label)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let externally_relevant = arguments
        .get(&externally_relevant_field)
        .and_then(Value::as_bool)
        .ok_or_else(|| {
            ShellError::message(format!("{tool_name} requires {externally_relevant_field}"))
        })?;
    Ok(WorkflowOutcomeRecord::new(
        source_slice,
        label,
        externally_relevant,
    ))
}

fn add_workflow_command_error_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "add_workflow_command_error")?;
    let workflow_slug = workflow_slug_from_arguments(arguments, "add_workflow_command_error")?;

    interpret_collect_reports(&command::add_workflow_command_error(
        workflow_slug,
        workflow_command_error_from_arguments(arguments, "add_workflow_command_error", "")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn update_workflow_command_error_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "update_workflow_command_error")?;
    let workflow_slug = workflow_slug_from_arguments(arguments, "update_workflow_command_error")?;

    interpret_collect_reports(&command::update_workflow_command_error(
        workflow_slug,
        workflow_command_error_from_arguments(arguments, "update_workflow_command_error", "")?,
        workflow_command_error_from_arguments(arguments, "update_workflow_command_error", "new_")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn remove_workflow_command_error_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "remove_workflow_command_error")?;
    let workflow_slug = workflow_slug_from_arguments(arguments, "remove_workflow_command_error")?;

    interpret_collect_reports(&command::remove_workflow_command_error(
        workflow_slug,
        workflow_command_error_from_arguments(arguments, "remove_workflow_command_error", "")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn workflow_command_error_from_arguments(
    arguments: &Value,
    tool_name: &str,
    prefix: &str,
) -> Result<WorkflowCommandErrorRecord, ShellError> {
    let source_slice_field = format!("{prefix}source_slice");
    let command_field = format!("{prefix}command");
    let error_field = format!("{prefix}error");
    let source_slice = arguments
        .get(&source_slice_field)
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires {source_slice_field}")))
        .and_then(|raw_source_slice| {
            WorkflowTransitionEndpoint::try_new(raw_source_slice.to_owned())
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get(&command_field)
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires {command_field}")))
        .and_then(|raw_command| {
            parse_command_name(raw_command).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let error_name = arguments
        .get(&error_field)
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires {error_field}")))
        .and_then(|raw_error| {
            parse_command_error_name(raw_error)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    Ok(WorkflowCommandErrorRecord::new(
        source_slice,
        command_name,
        error_name,
    ))
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
    let definition_stream = parse_optional_owned_definition_stream(arguments)?;
    let source_provenance = parse_optional_owned_definition_provenance(arguments)?;
    let event_participation = parse_optional_owned_definition_event_participation(arguments)?;
    let view_role = parse_optional_owned_definition_view_role(arguments)?;
    let definition = build_workflow_owned_definition(
        source_slice,
        definition_kind,
        definition_name,
        definition_stream,
        source_provenance,
        event_participation,
        view_role,
    )?;

    interpret_collect_reports(&command::add_workflow_owned_definition(
        workflow_slug,
        definition,
    ))
    .map(|reports| reports.join("\n"))
}

fn parse_optional_owned_definition_stream(
    arguments: &Value,
) -> Result<Option<StreamName>, ShellError> {
    arguments
        .get("definition_stream")
        .and_then(Value::as_str)
        .map(|raw_definition_stream| {
            parse_stream_name(raw_definition_stream)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()
}

fn parse_optional_owned_definition_provenance(
    arguments: &Value,
) -> Result<Option<ModelDescription>, ShellError> {
    arguments
        .get("source_provenance")
        .and_then(Value::as_str)
        .map(|raw_source_provenance| {
            parse_model_description(raw_source_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()
}

fn parse_optional_owned_definition_event_participation(
    arguments: &Value,
) -> Result<Option<WorkflowEventParticipation>, ShellError> {
    arguments
        .get("event_participation")
        .and_then(Value::as_str)
        .map(|raw_event_participation| {
            parse_workflow_event_participation(raw_event_participation)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()
}

fn parse_optional_owned_definition_view_role(
    arguments: &Value,
) -> Result<Option<WorkflowViewRole>, ShellError> {
    arguments
        .get("view_role")
        .and_then(Value::as_str)
        .map(|raw_view_role| {
            parse_workflow_view_role(raw_view_role)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()
}

fn build_workflow_owned_definition(
    source_slice: WorkflowTransitionEndpoint,
    definition_kind: WorkflowOwnedDefinitionKind,
    definition_name: WorkflowOwnedDefinitionName,
    definition_stream: Option<StreamName>,
    source_provenance: Option<ModelDescription>,
    event_participation: Option<WorkflowEventParticipation>,
    view_role: Option<WorkflowViewRole>,
) -> Result<WorkflowOwnedDefinitionRecord, ShellError> {
    match (
        definition_stream,
        source_provenance,
        event_participation,
        view_role,
    ) {
        (None, None, None, Some(view_role)) => WorkflowOwnedDefinitionRecord::new_with_view_role(
            source_slice,
            definition_kind,
            definition_name,
            view_role,
        )
        .ok_or_else(|| ShellError::message("view_role requires definition_kind view")),
        (Some(definition_stream), Some(source_provenance), Some(event_participation), None) => Ok(
            WorkflowOwnedDefinitionRecord::new_with_event_identity_and_participation(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
                event_participation,
            ),
        ),
        (Some(definition_stream), Some(source_provenance), None, None) => {
            Ok(WorkflowOwnedDefinitionRecord::new_with_event_identity(
                source_slice,
                definition_kind,
                definition_name,
                definition_stream,
                source_provenance,
            ))
        }
        (None, None, None, None) => Ok(WorkflowOwnedDefinitionRecord::new(
            source_slice,
            definition_kind,
            definition_name,
        )),
        (_, _, Some(_), _) => Err(ShellError::message(
            "event_participation requires definition_stream and source_provenance",
        )),
        (_, _, _, Some(_)) => Err(ShellError::message(
            "view_role cannot be combined with event identity fields",
        )),
        _ => Err(ShellError::message(
            "definition_stream and source_provenance must be provided together",
        )),
    }
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
    let source_control = parse_optional_transition_source_control(arguments)?;
    let target_view = parse_optional_transition_target_view(arguments)?;
    let source_evidence = arguments
        .get("source_evidence")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_workflow_transition_evidence requires source_evidence")
        })
        .and_then(|raw_source_evidence| {
            parse_workflow_transition_source_evidence_text(raw_source_evidence)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let target_evidence = arguments
        .get("target_evidence")
        .and_then(Value::as_str)
        .ok_or_else(|| {
            ShellError::message("add_workflow_transition_evidence requires target_evidence")
        })
        .and_then(|raw_target_evidence| {
            parse_workflow_transition_target_evidence_text(raw_target_evidence)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    let navigation_endpoints =
        resolve_transition_navigation_endpoints(kind, source_control, target_view)?;
    let evidence = build_workflow_transition_evidence(
        source,
        target,
        kind,
        trigger,
        navigation_endpoints,
        source_evidence,
        target_evidence,
    );

    interpret_collect_reports(&command::add_workflow_transition_evidence(
        workflow_slug,
        evidence,
    ))
    .map(|reports| reports.join("\n"))
}

fn resolve_transition_navigation_endpoints(
    kind: WorkflowTransitionKind,
    source_control: Option<TransitionTriggerName>,
    target_view: Option<WorkflowOwnedDefinitionName>,
) -> Result<Option<WorkflowTransitionEvidenceNavigationEndpoints>, ShellError> {
    if kind == WorkflowTransitionKind::Navigation {
        Ok(Some(WorkflowTransitionEvidenceNavigationEndpoints::new(
            source_control.ok_or_else(|| {
                ShellError::message(
                    "navigation transition evidence requires source_control owned by source slice",
                )
            })?,
            target_view.ok_or_else(|| {
                ShellError::message(
                    "navigation transition evidence requires target_view owned by target slice",
                )
            })?,
        )))
    } else {
        Ok(None)
    }
}

fn parse_optional_transition_source_control(
    arguments: &Value,
) -> Result<Option<TransitionTriggerName>, ShellError> {
    arguments
        .get("source_control")
        .and_then(Value::as_str)
        .map(|raw_source_control| {
            parse_transition_trigger_name(raw_source_control)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()
}

fn parse_optional_transition_target_view(
    arguments: &Value,
) -> Result<Option<WorkflowOwnedDefinitionName>, ShellError> {
    arguments
        .get("target_view")
        .and_then(Value::as_str)
        .map(|raw_target_view| {
            parse_workflow_owned_definition_name(raw_target_view)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()
}

fn build_workflow_transition_evidence(
    source: WorkflowTransitionEndpoint,
    target: WorkflowTransitionEndpoint,
    kind: WorkflowTransitionKind,
    trigger: TransitionTriggerName,
    navigation_endpoints: Option<WorkflowTransitionEvidenceNavigationEndpoints>,
    source_evidence: WorkflowTransitionSourceEvidenceText,
    target_evidence: WorkflowTransitionTargetEvidenceText,
) -> WorkflowTransitionEvidenceRecord {
    match navigation_endpoints {
        Some(navigation_endpoints) => {
            WorkflowTransitionEvidenceRecord::new_with_navigation_endpoints(
                source,
                target,
                kind,
                trigger,
                navigation_endpoints,
                source_evidence,
                target_evidence,
            )
        }
        None => WorkflowTransitionEvidenceRecord::new(
            source,
            target,
            kind,
            trigger,
            source_evidence,
            target_evidence,
        ),
    }
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

    interpret_collect_reports(&command::require_workflow_entry_lifecycle_coverage(
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

    interpret_collect_reports(&command::add_workflow_entry_lifecycle_state(
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
    interpret_collect_reports(&command::add_slice(NewSlice::new(
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
    let scenario = parse_slice_scenario_tool_arguments(arguments, "add_slice_scenario")?;

    interpret_collect_reports(&command::add_slice_scenario(scenario))
        .map(|reports| reports.join("\n"))
}

fn parse_slice_scenario_tool_arguments(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewSliceScenario, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let scenario_kind = arguments
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires kind")))
        .and_then(|raw_kind| {
            parse_scenario_kind(raw_kind).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_scenario_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let given = arguments
        .get("given")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires given")))
        .and_then(|raw_given| {
            parse_scenario_step_text(raw_given)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let when = arguments
        .get("when")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires when")))
        .and_then(|raw_when| {
            parse_scenario_step_text(raw_when)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let then = arguments
        .get("then")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires then")))
        .and_then(|raw_then| {
            parse_scenario_step_text(raw_then)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    let scenario = match scenario_kind {
        ScenarioKind::Acceptance => NewSliceScenario::new(
            slice_slug,
            ScenarioKind::acceptance(),
            name,
            given,
            when,
            then,
        ),
        ScenarioKind::Contract => {
            let contract_kind = arguments
                .get("contract_kind")
                .and_then(Value::as_str)
                .ok_or_else(|| ShellError::message(format!("{tool_name} requires contract_kind")))
                .and_then(|raw_contract_kind| {
                    parse_contract_kind_name(raw_contract_kind)
                        .map_err(|error| ShellError::message(error.to_string()))
                })?;
            let covered_definition = arguments
                .get("covered_definition")
                .and_then(Value::as_str)
                .ok_or_else(|| {
                    ShellError::message(format!("{tool_name} requires covered_definition"))
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
    };
    let scenario = apply_optional_scenario_streams(arguments, scenario)?;
    apply_optional_scenario_error_references(arguments, scenario)
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
    let arguments = required_tool_arguments(request, "add_bit_level_data_flow")?;
    interpret_collect_reports(&command::add_bit_level_data_flow(
        bit_level_data_flow_from_arguments(arguments, "add_bit_level_data_flow", "")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn update_bit_level_data_flow_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "update_bit_level_data_flow")?;
    interpret_collect_reports(&command::update_bit_level_data_flow(
        bit_level_data_flow_from_arguments(arguments, "update_bit_level_data_flow", "")?,
        bit_level_data_flow_from_arguments(arguments, "update_bit_level_data_flow", "new_")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn remove_bit_level_data_flow_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "remove_bit_level_data_flow")?;
    interpret_collect_reports(&command::remove_bit_level_data_flow(
        bit_level_data_flow_from_arguments(arguments, "remove_bit_level_data_flow", "")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn bit_level_data_flow_from_arguments(
    arguments: &Value,
    tool_name: &str,
    prefix: &str,
) -> Result<NewBitLevelDataFlow, ShellError> {
    Ok(NewBitLevelDataFlow::new(
        bit_level_data_flow_slice(arguments, tool_name)?,
        data_flow_arg(
            arguments,
            tool_name,
            &format!("{prefix}datum"),
            parse_datum_name,
        )?,
        data_flow_arg(
            arguments,
            tool_name,
            &format!("{prefix}source_kind"),
            parse_data_flow_source_kind,
        )?,
        data_flow_arg(
            arguments,
            tool_name,
            &format!("{prefix}source"),
            parse_data_flow_source,
        )?,
        data_flow_arg(
            arguments,
            tool_name,
            &format!("{prefix}transformation"),
            parse_transformation_semantics,
        )?,
        data_flow_arg(
            arguments,
            tool_name,
            &format!("{prefix}target"),
            parse_data_flow_target,
        )?,
        data_flow_arg(
            arguments,
            tool_name,
            &format!("{prefix}bit_encoding"),
            parse_bit_encoding_semantics,
        )?,
    ))
}

fn bit_level_data_flow_slice(arguments: &Value, tool_name: &str) -> Result<SliceSlug, ShellError> {
    let raw_slice = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))?;
    parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
}

fn data_flow_arg<T, E>(
    arguments: &Value,
    tool_name: &str,
    field: &str,
    parse: impl FnOnce(&str) -> Result<T, E>,
) -> Result<T, ShellError>
where
    E: Display,
{
    let raw_value = arguments
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires {field}")))?;
    parse(raw_value).map_err(|error| ShellError::message(error.to_string()))
}

fn add_board_element_tool_text(request: &Value) -> Result<String, ShellError> {
    interpret_collect_reports(&command::add_board_element(board_element_from_request(
        request,
        "add_board_element",
    )?))
    .map(|reports| reports.join("\n"))
}

fn update_board_element_tool_text(request: &Value) -> Result<String, ShellError> {
    interpret_collect_reports(&command::update_board_element(board_element_from_request(
        request,
        "update_board_element",
    )?))
    .map(|reports| reports.join("\n"))
}

fn remove_board_element_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "remove_board_element")?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_board_element requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_board_element requires name"))
        .and_then(|raw_name| {
            parse_board_element_name(raw_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(&command::remove_board_element(slice_slug, name))
        .map(|reports| reports.join("\n"))
}

fn board_element_from_request(
    request: &Value,
    tool_name: &str,
) -> Result<NewBoardElement, ShellError> {
    let arguments = required_tool_arguments(request, tool_name)?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_board_element_name(raw_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let kind = arguments
        .get("kind")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires kind")))
        .and_then(|raw_kind| {
            parse_board_element_kind(raw_kind)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let lane = arguments
        .get("lane")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires lane")))
        .and_then(|raw_lane| {
            parse_board_lane_id(raw_lane).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let declared_name = arguments
        .get("declared_name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires declared_name")))
        .and_then(|raw_declared_name| {
            parse_board_element_declared_name(raw_declared_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let main_path = arguments
        .get("main_path")
        .and_then(Value::as_bool)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires main_path")))?;

    Ok(NewBoardElement::new(
        slice_slug,
        name,
        kind,
        lane,
        declared_name,
        main_path,
    ))
}

fn required_tool_arguments<'request>(
    request: &'request Value,
    tool_name: &str,
) -> Result<&'request Value, ShellError> {
    request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires arguments")))
}

fn add_board_connection_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "add_board_connection")?;
    interpret_collect_reports(&command::add_board_connection(
        board_connection_from_arguments(arguments, "add_board_connection", "")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn update_board_connection_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "update_board_connection")?;
    interpret_collect_reports(&command::update_board_connection(
        board_connection_from_arguments(arguments, "update_board_connection", "")?,
        board_connection_from_arguments(arguments, "update_board_connection", "new_")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn remove_board_connection_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = required_tool_arguments(request, "remove_board_connection")?;
    interpret_collect_reports(&command::remove_board_connection(
        board_connection_from_arguments(arguments, "remove_board_connection", "")?,
    ))
    .map(|reports| reports.join("\n"))
}

fn board_connection_from_arguments(
    arguments: &Value,
    tool_name: &str,
    prefix: &str,
) -> Result<NewBoardConnection, ShellError> {
    Ok(NewBoardConnection::new(
        board_connection_slice(arguments, tool_name)?,
        board_connection_endpoint_arg(arguments, tool_name, &format!("{prefix}source"))?,
        board_connection_endpoint_kind_arg(arguments, tool_name, &format!("{prefix}source_kind"))?,
        board_connection_endpoint_arg(arguments, tool_name, &format!("{prefix}target"))?,
        board_connection_endpoint_kind_arg(arguments, tool_name, &format!("{prefix}target_kind"))?,
    ))
}

fn board_connection_slice(arguments: &Value, tool_name: &str) -> Result<SliceSlug, ShellError> {
    let raw_slice = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))?;
    parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
}

fn board_connection_endpoint_arg(
    arguments: &Value,
    tool_name: &str,
    field: &str,
) -> Result<BoardConnectionEndpoint, ShellError> {
    let raw_value = arguments
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires {field}")))?;
    parse_board_connection_endpoint(raw_value)
        .map_err(|error| ShellError::message(error.to_string()))
}

fn board_connection_endpoint_kind_arg(
    arguments: &Value,
    tool_name: &str,
    field: &str,
) -> Result<BoardConnectionEndpointKind, ShellError> {
    let raw_value = arguments
        .get(field)
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires {field}")))?;
    parse_board_connection_endpoint_kind(raw_value)
        .map_err(|error| ShellError::message(error.to_string()))
}

/// Required scalar fields for a command definition, parsed from MCP arguments.
struct CommandDefinitionInputs {
    slice_slug: SliceSlug,
    command_name: CommandName,
    input_name: DatumName,
    input_source: CommandInputSourceKind,
    input_description: CommandInputSourceDescription,
    provenance_chain: Vec<SourceChainHop>,
    emitted_events: Vec<EventName>,
}

/// Optional source-reference fields that disambiguate a command's input source.
struct CommandInputSourceRefs {
    event: Option<EventName>,
    attribute: Option<EventAttributeName>,
    payload: Option<EventAttributeSourceName>,
    name: Option<EventAttributeSourceName>,
    session: Option<EventAttributeSourceName>,
    argument: Option<EventAttributeSourceName>,
    field: Option<EventAttributeSourceField>,
}

fn add_command_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_command_definition requires arguments"))?;
    let command_definition =
        build_command_definition_from_arguments(arguments, "add_command_definition")?;

    interpret_collect_reports(&command::add_command_definition(command_definition))
        .map(|reports| reports.join("\n"))
}

fn update_command_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_command_definition requires arguments"))?;
    let command_definition =
        build_command_definition_from_arguments(arguments, "update_command_definition")?;

    interpret_collect_reports(&command::update_command_definition(command_definition))
        .map(|reports| reports.join("\n"))
}

fn remove_command_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_command_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_command_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_command_definition requires name"))
        .and_then(|raw_name| {
            parse_command_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(&command::remove_command_definition(
        slice_slug,
        command_name,
    ))
    .map(|reports| reports.join("\n"))
}

fn build_command_definition_from_arguments(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewCommandDefinition, ShellError> {
    let inputs = parse_command_definition_inputs(arguments, tool_name)?;
    let observed_streams = arguments
        .get("observes")
        .and_then(Value::as_str)
        .map(parse_stream_names)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let refs = parse_command_input_source_refs(arguments)?;
    let command_errors = parse_optional_command_errors(arguments, tool_name)?;
    let singleton_repeat_behavior = parse_optional_singleton_repeat_behavior(arguments, tool_name)?;

    let command_input_source = resolve_command_input_source(inputs.input_source, refs, tool_name)?;
    let command_input = NewCommandInput::new(
        inputs.input_name,
        command_input_source,
        inputs.input_description,
        CommandInputProvenanceChain::from_hops(inputs.provenance_chain),
    );
    let command_definition = NewCommandDefinition::new(
        inputs.slice_slug,
        inputs.command_name,
        command_input,
        EmittedEventNames::from_events(inputs.emitted_events),
    )
    .with_observed_streams(CommandObservedStreams::from_streams(
        observed_streams.unwrap_or_default(),
    ))
    .with_errors(command_errors);
    let command_definition = singleton_repeat_behavior
        .map_or(command_definition.clone(), |behavior| {
            command_definition.with_singleton_repeat_behavior(behavior)
        });
    Ok(command_definition)
}

fn parse_command_definition_inputs(
    arguments: &Value,
    tool_name: &str,
) -> Result<CommandDefinitionInputs, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_command_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_name = arguments
        .get("input")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input")))
        .and_then(|raw_input| {
            parse_datum_name(raw_input).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_source = arguments
        .get("input_source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input_source")))
        .and_then(|raw_source| {
            parse_command_input_source_kind(raw_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_description = arguments
        .get("input_description")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input_description")))
        .and_then(|raw_description| {
            parse_command_input_source_description(raw_description)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let provenance_chain = arguments
        .get("input_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input_provenance")))
        .and_then(|raw_provenance| {
            parse_source_chain_hops(raw_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let emitted_events = arguments
        .get("emits")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires emits")))
        .and_then(|raw_emits| {
            parse_event_names(raw_emits).map_err(|error| ShellError::message(error.to_string()))
        })?;
    Ok(CommandDefinitionInputs {
        slice_slug,
        command_name,
        input_name,
        input_source,
        input_description,
        provenance_chain,
        emitted_events,
    })
}

fn parse_command_input_source_refs(
    arguments: &Value,
) -> Result<CommandInputSourceRefs, ShellError> {
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
    let source_name = arguments
        .get("source_name")
        .and_then(Value::as_str)
        .map(parse_event_attribute_source_name)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_session = arguments
        .get("source_session")
        .and_then(Value::as_str)
        .map(parse_event_attribute_source_name)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let source_argument = arguments
        .get("source_argument")
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
    Ok(CommandInputSourceRefs {
        event: source_event,
        attribute: source_attribute,
        payload: source_payload,
        name: source_name,
        session: source_session,
        argument: source_argument,
        field: source_field,
    })
}

fn resolve_command_input_source(
    input_source: CommandInputSourceKind,
    refs: CommandInputSourceRefs,
    tool_name: &str,
) -> Result<CommandInputSource, ShellError> {
    match (
        input_source,
        refs.event,
        refs.attribute,
        refs.payload,
        refs.name,
        refs.session,
        refs.argument,
        refs.field,
    ) {
        (
            CommandInputSourceKind::EventStreamState,
            Some(event),
            Some(attribute),
            None,
            None,
            None,
            None,
            None,
        ) => Ok(CommandInputSource::event_stream_state(event, attribute)),
        (
            CommandInputSourceKind::ExternalPayload,
            None,
            None,
            Some(payload),
            None,
            None,
            None,
            Some(field),
        ) => Ok(CommandInputSource::external_payload(payload, field)),
        (
            CommandInputSourceKind::Generated,
            None,
            None,
            None,
            Some(source),
            None,
            None,
            Some(field),
        ) => Ok(CommandInputSource::generated(source, field)),
        (
            CommandInputSourceKind::Session,
            None,
            None,
            None,
            None,
            Some(session),
            None,
            Some(field),
        ) => Ok(CommandInputSource::session(session, field)),
        (
            CommandInputSourceKind::InvocationArgument,
            None,
            None,
            None,
            None,
            None,
            Some(argument),
            Some(field),
        ) => Ok(CommandInputSource::invocation_argument(argument, field)),
        (CommandInputSourceKind::Actor, None, None, None, None, None, None, None) => {
            Ok(CommandInputSource::actor())
        }
        (_, Some(_), None, None, None, None, None, None) => Err(ShellError::message(format!(
            "{tool_name} requires source_attribute when source_event is provided"
        ))),
        (_, None, Some(_), None, None, None, None, None) => Err(ShellError::message(format!(
            "{tool_name} requires source_event when source_attribute is provided"
        ))),
        (_, None, None, Some(_), None, None, None, None) => Err(ShellError::message(format!(
            "{tool_name} requires source_field when source_payload is provided"
        ))),
        (_, None, None, None, Some(_), None, None, None) => Err(ShellError::message(format!(
            "{tool_name} requires source_field when source_name is provided"
        ))),
        (_, None, None, None, None, Some(_), None, None) => Err(ShellError::message(format!(
            "{tool_name} requires source_field when source_session is provided"
        ))),
        (_, None, None, None, None, None, Some(_), None) => Err(ShellError::message(format!(
            "{tool_name} requires source_field when source_argument is provided"
        ))),
        (_, None, None, None, None, None, None, Some(_)) => Err(ShellError::message(format!(
            "{tool_name} requires source_payload, source_name, source_session, or source_argument when source_field is provided"
        ))),
        _ => Err(ShellError::message(format!(
            "{tool_name} requires input_source and source reference fields to describe the same command input source"
        ))),
    }
}

fn parse_optional_singleton_repeat_behavior(
    arguments: &Value,
    tool_name: &str,
) -> Result<Option<SingletonRepeatBehavior>, ShellError> {
    match (
        arguments.get("singleton").and_then(Value::as_bool),
        arguments.get("repeat_behavior").and_then(Value::as_str),
    ) {
        (Some(true), Some(repeat_behavior)) => parse_singleton_repeat_behavior(repeat_behavior)
            .map(Some)
            .map_err(|error| ShellError::message(error.to_string())),
        (Some(false), _) | (None, None) => Ok(None),
        (Some(true), None) => Err(ShellError::message(format!(
            "{tool_name} requires repeat_behavior when singleton is true"
        ))),
        (None, Some(_)) => Err(ShellError::message(format!(
            "{tool_name} requires singleton when repeat_behavior is provided"
        ))),
    }
}

fn parse_optional_command_errors(
    arguments: &Value,
    tool_name: &str,
) -> Result<CommandErrorDefinitions, ShellError> {
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
        _ => Err(ShellError::message(format!(
            "{tool_name} requires error, error_scenario, and error_recovery together"
        ))),
    }
}

fn add_automation_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "add_automation_definition")?;
    let automation = parse_automation_definition(arguments, "add_automation_definition")?;
    interpret_collect_reports(&command::add_automation_definition(automation))
        .map(|reports| reports.join("\n"))
}

fn update_automation_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "update_automation_definition")?;
    let automation = parse_automation_definition(arguments, "update_automation_definition")?;
    interpret_collect_reports(&command::update_automation_definition(automation))
        .map(|reports| reports.join("\n"))
}

fn remove_automation_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "remove_automation_definition")?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_automation_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let automation_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_automation_definition requires name"))
        .and_then(|raw_name| {
            parse_automation_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(&command::remove_automation_definition(
        slice_slug,
        automation_name,
    ))
    .map(|reports| reports.join("\n"))
}

fn parse_automation_definition(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewAutomationDefinition, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let automation_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_automation_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let trigger_name = arguments
        .get("trigger")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires trigger")))
        .and_then(|raw_trigger| {
            parse_automation_trigger_name(raw_trigger)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires command")))
        .and_then(|raw_command| {
            parse_command_name(raw_command).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let handled_errors = arguments
        .get("handled_errors")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires handled_errors")))
        .and_then(|raw_errors| {
            parse_command_error_names(raw_errors)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let reaction_description = arguments
        .get("reaction")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires reaction")))
        .and_then(|raw_reaction| {
            parse_automation_reaction_description(raw_reaction)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    Ok(NewAutomationDefinition::new(
        slice_slug,
        automation_name,
        trigger_name,
        command_name,
        CommandErrorNames::from_names(handled_errors),
        reaction_description,
    ))
}

fn add_translation_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "add_translation_definition")?;
    let translation = parse_translation_definition(arguments, "add_translation_definition")?;
    interpret_collect_reports(&command::add_translation_definition(translation))
        .map(|reports| reports.join("\n"))
}

fn update_translation_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "update_translation_definition")?;
    let translation = parse_translation_definition(arguments, "update_translation_definition")?;
    interpret_collect_reports(&command::update_translation_definition(translation))
        .map(|reports| reports.join("\n"))
}

fn remove_translation_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "remove_translation_definition")?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_translation_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let translation_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_translation_definition requires name"))
        .and_then(|raw_name| {
            parse_translation_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(&command::remove_translation_definition(
        slice_slug,
        translation_name,
    ))
    .map(|reports| reports.join("\n"))
}

fn parse_translation_definition(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewTranslationDefinition, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let translation_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_translation_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let external_event_name = arguments
        .get("external_event")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires external_event")))
        .and_then(|raw_external_event| {
            parse_translation_external_event_name(raw_external_event)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let payload_contract_name = arguments
        .get("payload_contract")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires payload_contract")))
        .and_then(|raw_payload_contract| {
            parse_payload_contract_name(raw_payload_contract)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let command_name = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires command")))
        .and_then(|raw_command| {
            parse_command_name(raw_command).map_err(|error| ShellError::message(error.to_string()))
        })?;
    Ok(NewTranslationDefinition::new(
        slice_slug,
        translation_name,
        external_event_name,
        payload_contract_name,
        command_name,
    ))
}

fn add_external_payload_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "add_external_payload_definition")?;
    let external_payload =
        parse_external_payload_definition(arguments, "add_external_payload_definition")?;
    interpret_collect_reports(&command::add_external_payload_definition(external_payload))
        .map(|reports| reports.join("\n"))
}

fn update_external_payload_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "update_external_payload_definition")?;
    let external_payload =
        parse_external_payload_definition(arguments, "update_external_payload_definition")?;
    interpret_collect_reports(&command::update_external_payload_definition(
        external_payload,
    ))
    .map(|reports| reports.join("\n"))
}

fn remove_external_payload_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "remove_external_payload_definition")?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_external_payload_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let payload_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_external_payload_definition requires name"))
        .and_then(|raw_name| {
            parse_event_attribute_source_name(raw_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let payload_field = arguments
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_external_payload_definition requires field"))
        .and_then(|raw_field| {
            parse_event_attribute_source_field(raw_field)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(&command::remove_external_payload_definition(
        slice_slug,
        payload_name,
        payload_field,
    ))
    .map(|reports| reports.join("\n"))
}

fn parse_external_payload_definition(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewExternalPayloadDefinition, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let payload_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_event_attribute_source_name(raw_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let payload_field = arguments
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires field")))
        .and_then(|raw_field| {
            parse_event_attribute_source_field(raw_field)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let field_provenance = arguments
        .get("field_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires field_provenance")))
        .and_then(|raw_field_provenance| {
            parse_provenance_description(raw_field_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let bit_encoding = arguments
        .get("bit_encoding")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires bit_encoding")))
        .and_then(|raw_bit_encoding| {
            parse_bit_encoding_semantics(raw_bit_encoding)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    Ok(NewExternalPayloadDefinition::new(
        slice_slug,
        payload_name,
        payload_field,
        field_provenance,
        bit_encoding,
    ))
}

fn add_outcome_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "add_outcome_definition")?;
    let outcome = parse_outcome_definition(arguments, "add_outcome_definition")?;
    interpret_collect_reports(&command::add_outcome_definition(outcome))
        .map(|reports| reports.join("\n"))
}

fn update_outcome_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "update_outcome_definition")?;
    let outcome = parse_outcome_definition(arguments, "update_outcome_definition")?;
    interpret_collect_reports(&command::update_outcome_definition(outcome))
        .map(|reports| reports.join("\n"))
}

fn remove_outcome_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = tool_arguments(request, "remove_outcome_definition")?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_outcome_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let label = arguments
        .get("label")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_outcome_definition requires label"))
        .and_then(|raw_label| {
            parse_outcome_label_name(raw_label)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(&command::remove_outcome_definition(slice_slug, label))
        .map(|reports| reports.join("\n"))
}

fn tool_arguments<'a>(request: &'a Value, tool_name: &str) -> Result<&'a Value, ShellError> {
    request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires arguments")))
}

fn parse_outcome_definition(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewOutcomeDefinition, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let label = arguments
        .get("label")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires label")))
        .and_then(|raw_label| {
            parse_outcome_label_name(raw_label)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let events = arguments
        .get("events")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires events")))
        .and_then(|raw_events| {
            parse_event_names(raw_events).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let externally_relevant = arguments
        .get("externally_relevant")
        .and_then(Value::as_bool)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires externally_relevant")))?;
    Ok(NewOutcomeDefinition::new(
        slice_slug,
        label,
        OutcomeEventNames::from_events(events),
        externally_relevant,
    ))
}

fn add_event_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_event_definition requires arguments"))?;
    let event = build_event_definition_from_arguments(arguments, "add_event_definition")?;

    interpret_collect_reports(&command::add_event_definition(event))
        .map(|reports| reports.join("\n"))
}

fn update_event_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_event_definition requires arguments"))?;
    let event = build_event_definition_from_arguments(arguments, "update_event_definition")?;

    interpret_collect_reports(&command::update_event_definition(event))
        .map(|reports| reports.join("\n"))
}

fn remove_event_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_event_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_event_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let event_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_event_definition requires name"))
        .and_then(|raw_name| {
            parse_event_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(&command::remove_event_definition(slice_slug, event_name))
        .map(|reports| reports.join("\n"))
}

fn build_event_definition_from_arguments(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewEventDefinition, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let event_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_event_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let stream_name = arguments
        .get("stream")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires stream")))
        .and_then(|raw_stream| {
            parse_stream_name(raw_stream).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let attribute_name = arguments
        .get("attribute")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires attribute")))
        .and_then(|raw_attribute| {
            parse_event_attribute_name(raw_attribute)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let attribute_source_kind = arguments
        .get("attribute_source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires attribute_source")))
        .and_then(|raw_attribute_source| {
            parse_event_attribute_source_kind(raw_attribute_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let attribute_source_name = arguments
        .get("attribute_source_name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires attribute_source_name")))
        .and_then(|raw_attribute_source_name| {
            parse_event_attribute_source_name(raw_attribute_source_name)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let attribute_source_field = arguments
        .get("attribute_source_field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires attribute_source_field")))
        .and_then(|raw_attribute_source_field| {
            parse_event_attribute_source_field(raw_attribute_source_field)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let generated_source_kind = arguments
        .get("generated_source_kind")
        .and_then(Value::as_str)
        .map(parse_generated_event_attribute_source_kind)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))?;
    let provenance_description = arguments
        .get("attribute_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires attribute_provenance")))
        .and_then(|raw_provenance| {
            parse_provenance_description(raw_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;

    let attribute = build_event_attribute(
        attribute_name,
        attribute_source_kind,
        attribute_source_name,
        attribute_source_field,
        generated_source_kind,
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
    Ok(build_event_definition(
        slice_slug,
        event_name,
        stream_name,
        attribute,
        observed,
        shared,
    ))
}

fn build_event_attribute(
    attribute_name: EventAttributeName,
    attribute_source_kind: EventAttributeSourceKind,
    attribute_source_name: EventAttributeSourceName,
    attribute_source_field: EventAttributeSourceField,
    generated_source_kind: Option<GeneratedEventAttributeSourceKind>,
    provenance_description: ProvenanceDescription,
) -> NewEventAttribute {
    match generated_source_kind {
        Some(generated_source_kind) => NewEventAttribute::new_with_generated_source_kind(
            attribute_name,
            attribute_source_kind,
            attribute_source_name,
            attribute_source_field,
            generated_source_kind,
            provenance_description,
        ),
        None => NewEventAttribute::new(
            attribute_name,
            attribute_source_kind,
            attribute_source_name,
            attribute_source_field,
            provenance_description,
        ),
    }
}

fn build_event_definition(
    slice_slug: SliceSlug,
    event_name: EventName,
    stream_name: StreamName,
    attribute: NewEventAttribute,
    observed: bool,
    shared: bool,
) -> NewEventDefinition {
    if shared {
        NewEventDefinition::new_shared(slice_slug, event_name, stream_name, attribute)
    } else if observed {
        NewEventDefinition::new_observed(slice_slug, event_name, stream_name, attribute)
    } else {
        NewEventDefinition::new(slice_slug, event_name, stream_name, attribute)
    }
}

fn add_read_model_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_read_model_definition requires arguments"))?;
    let read_model =
        build_read_model_definition_from_arguments(arguments, "add_read_model_definition")?;

    interpret_collect_reports(&command::add_read_model_definition(read_model))
        .map(|reports| reports.join("\n"))
}

fn update_read_model_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_read_model_definition requires arguments"))?;
    let read_model =
        build_read_model_definition_from_arguments(arguments, "update_read_model_definition")?;

    interpret_collect_reports(&command::update_read_model_definition(read_model))
        .map(|reports| reports.join("\n"))
}

fn remove_read_model_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_read_model_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_read_model_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let read_model_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_read_model_definition requires name"))
        .and_then(|raw_name| {
            parse_read_model_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(&command::remove_read_model_definition(
        slice_slug,
        read_model_name,
    ))
    .map(|reports| reports.join("\n"))
}

fn build_read_model_definition_from_arguments(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewReadModelDefinition, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let read_model_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_read_model_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let field_name = arguments
        .get("field")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires field")))
        .and_then(|raw_field| {
            parse_datum_name(raw_field).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let field_source_kind = arguments
        .get("field_source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires field_source")))
        .and_then(|raw_field_source| {
            parse_read_model_field_source_kind(raw_field_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let provenance_description = arguments
        .get("field_provenance")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires field_provenance")))
        .and_then(|raw_provenance| {
            parse_provenance_description(raw_provenance)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let read_model_source = resolve_read_model_field_source(field_source_kind, arguments)?;
    let read_model_field =
        NewReadModelField::new(field_name, read_model_source, provenance_description);
    let read_model = NewReadModelDefinition::new(slice_slug, read_model_name, read_model_field);
    let read_model = apply_optional_transitive_semantics(read_model, arguments)?;

    Ok(read_model)
}

fn resolve_read_model_field_source(
    field_source_kind: ReadModelFieldSourceKind,
    arguments: &Value,
) -> Result<ReadModelFieldSource, ShellError> {
    let source_event = arguments.get("source_event").and_then(Value::as_str);
    let source_attribute = arguments.get("source_attribute").and_then(Value::as_str);
    let derivation_rule = arguments.get("derivation_rule").and_then(Value::as_str);
    let derivation_source_fields = arguments
        .get("derivation_source_fields")
        .and_then(Value::as_str);
    let derivation_scenario = arguments.get("derivation_scenario").and_then(Value::as_str);
    let absence_event = arguments.get("absence_event").and_then(Value::as_str);
    let absence_scenario = arguments.get("absence_scenario").and_then(Value::as_str);
    match (
        field_source_kind,
        source_event,
        source_attribute,
        derivation_rule,
        derivation_source_fields,
        derivation_scenario,
        absence_event,
        absence_scenario,
    ) {
        (
            ReadModelFieldSourceKind::EventAttribute,
            Some(raw_source_event),
            Some(raw_source_attribute),
            None,
            None,
            None,
            None,
            None,
        ) => Ok(ReadModelFieldSource::event_attribute(
            parse_event_name(raw_source_event)
                .map_err(|error| ShellError::message(error.to_string()))?,
            parse_event_attribute_name(raw_source_attribute)
                .map_err(|error| ShellError::message(error.to_string()))?,
        )),
        (
            ReadModelFieldSourceKind::Derivation,
            None,
            None,
            Some(raw_derivation_rule),
            Some(raw_derivation_source_fields),
            Some(raw_derivation_scenario),
            None,
            None,
        ) => Ok(ReadModelFieldSource::derivation(
            parse_read_model_derivation_rule(raw_derivation_rule)
                .map_err(|error| ShellError::message(error.to_string()))?,
            ReadModelDerivationSourceFields::from_fields(
                parse_datum_names(raw_derivation_source_fields)
                    .map_err(|error| ShellError::message(error.to_string()))?,
            ),
            parse_scenario_name(raw_derivation_scenario)
                .map_err(|error| ShellError::message(error.to_string()))?,
        )),
        (
            ReadModelFieldSourceKind::AbsenceDefault,
            None,
            None,
            None,
            None,
            None,
            Some(raw_absence_event),
            Some(raw_absence_scenario),
        ) => Ok(ReadModelFieldSource::absence_default(
            parse_event_name(raw_absence_event)
                .map_err(|error| ShellError::message(error.to_string()))?,
            parse_scenario_name(raw_absence_scenario)
                .map_err(|error| ShellError::message(error.to_string()))?,
        )),
        (_, Some(_), None, _, _, _, _, _) | (_, None, Some(_), _, _, _, _, _) => {
            Err(ShellError::message(
                "add_read_model_definition requires source_event and source_attribute together",
            ))
        }
        (_, _, _, Some(_), None, _, _, _)
        | (_, _, _, None, Some(_), _, _, _)
        | (_, _, _, Some(_), _, None, _, _)
        | (_, _, _, None, _, Some(_), _, _) => Err(ShellError::message(
            "add_read_model_definition requires derivation_rule, derivation_source_fields, and derivation_scenario together",
        )),
        (_, _, _, _, _, _, Some(_), None) | (_, _, _, _, _, _, None, Some(_)) => {
            Err(ShellError::message(
                "add_read_model_definition requires absence_event and absence_scenario together",
            ))
        }
        _ => Err(ShellError::message(
            "add_read_model_definition requires field_source and source fields to describe the same read model field source",
        )),
    }
}

fn apply_optional_transitive_semantics(
    read_model: NewReadModelDefinition,
    arguments: &Value,
) -> Result<NewReadModelDefinition, ShellError> {
    if !arguments
        .get("transitive")
        .and_then(Value::as_bool)
        .unwrap_or(false)
    {
        return Ok(read_model);
    }
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
    Ok(read_model.with_transitive_semantics(
        ReadModelRelationshipFields::from_fields(relationship_fields),
        transitive_rule,
        example_scenario,
    ))
}

fn add_view_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("add_view_definition requires arguments"))?;
    let view_definition = build_view_definition_from_arguments(arguments, "add_view_definition")?;

    interpret_collect_reports(&command::add_view_definition(view_definition))
        .map(|reports| reports.join("\n"))
}

fn update_view_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_view_definition requires arguments"))?;
    let view_definition =
        build_view_definition_from_arguments(arguments, "update_view_definition")?;

    interpret_collect_reports(&command::update_view_definition(view_definition))
        .map(|reports| reports.join("\n"))
}

fn remove_view_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_view_definition requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_view_definition requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let view_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_view_definition requires name"))
        .and_then(|raw_name| {
            parse_view_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(&command::remove_view_definition(slice_slug, view_name))
        .map(|reports| reports.join("\n"))
}

fn update_control_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_control_definition requires arguments"))?;
    let (slice_slug, view_name) = parse_control_location(arguments, "update_control_definition")?;
    let control = parse_control_definition(arguments, "update_control_definition")?;

    interpret_collect_reports(&command::update_control_definition(
        slice_slug, view_name, control,
    ))
    .map(|reports| reports.join("\n"))
}

fn remove_control_definition_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_control_definition requires arguments"))?;
    let (slice_slug, view_name) = parse_control_location(arguments, "remove_control_definition")?;
    let control_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_control_definition requires name"))
        .and_then(|raw_name| {
            parse_control_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;

    interpret_collect_reports(&command::remove_control_definition(
        slice_slug,
        view_name,
        control_name,
    ))
    .map(|reports| reports.join("\n"))
}

fn parse_control_location(
    arguments: &Value,
    tool_name: &str,
) -> Result<(SliceSlug, ViewName), ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let view_name = arguments
        .get("view")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires view")))
        .and_then(|raw_view| {
            parse_view_name(raw_view).map_err(|error| ShellError::message(error.to_string()))
        })?;
    Ok((slice_slug, view_name))
}

fn parse_control_definition(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewControlDefinition, ShellError> {
    let control_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_control_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let control_command = arguments
        .get("command")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires command")))
        .and_then(|raw_command| {
            parse_command_name(raw_command).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input = parse_control_definition_input(arguments, tool_name)?;
    let handled_errors = arguments
        .get("handled_errors")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires handled_errors")))
        .and_then(|raw_errors| {
            parse_command_error_names(raw_errors)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let recovery_behavior = arguments
        .get("recovery_behavior")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires recovery_behavior")))
        .and_then(|raw_recovery| {
            parse_control_recovery_behavior(raw_recovery)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let sketch_token = arguments
        .get("sketch_token")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires sketch_token")))
        .and_then(|raw_sketch_token| {
            parse_sketch_token(raw_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let navigation = parse_control_definition_navigation(arguments, tool_name)?;

    Ok(NewControlDefinition::new(
        control_name,
        control_command,
        input,
        CommandErrorNames::from_names(handled_errors),
        recovery_behavior,
        sketch_token,
        navigation,
    ))
}

fn parse_control_definition_input(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewControlInputProvision, ShellError> {
    let input = arguments
        .get("input")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input")))
        .and_then(|raw_input| {
            parse_datum_name(raw_input).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_source = arguments
        .get("input_source")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input_source")))
        .and_then(|raw_source| {
            parse_command_input_source_kind(raw_source)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_description = arguments
        .get("input_description")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input_description")))
        .and_then(|raw_description| {
            parse_command_input_source_description(raw_description)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_sketch_token = arguments
        .get("input_sketch_token")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input_sketch_token")))
        .and_then(|raw_sketch_token| {
            parse_sketch_token(raw_sketch_token)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let input_visible = arguments
        .get("input_visible")
        .and_then(Value::as_bool)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input_visible")))?;
    let input_decision = arguments
        .get("input_decision")
        .and_then(Value::as_bool)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires input_decision")))?;

    Ok(NewControlInputProvision::new(
        input,
        input_source,
        input_description,
        input_sketch_token,
        input_visible,
        input_decision,
    ))
}

fn parse_control_definition_navigation(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewNavigationTarget, ShellError> {
    let navigation_type = arguments
        .get("navigation_type")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires navigation_type")))
        .and_then(|raw_navigation_type| {
            parse_navigation_target_type(raw_navigation_type)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    let navigation_target = arguments
        .get("navigation_target")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires navigation_target")))
        .and_then(|raw_navigation_target| {
            parse_navigation_target_name(raw_navigation_target)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    Ok(NewNavigationTarget::new(navigation_type, navigation_target))
}

fn build_view_definition_from_arguments(
    arguments: &Value,
    tool_name: &str,
) -> Result<NewViewDefinition, ShellError> {
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires slice")))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let view_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message(format!("{tool_name} requires name")))
        .and_then(|raw_name| {
            parse_view_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let view_field = parse_view_definition_field(arguments)?;
    let navigation = parse_view_definition_navigation(arguments)?;
    let control = parse_view_definition_control(arguments, navigation)?;
    let local_states = parse_view_definition_navigation_targets(arguments, "local_states")?;
    let filters = parse_view_definition_navigation_targets(arguments, "filters")?;

    let view_definition = NewViewDefinition::new(slice_slug, view_name, view_field)
        .with_local_states(ViewLocalStates::from_targets(
            local_states.unwrap_or_default(),
        ))
        .with_filters(ViewFilters::from_targets(filters.unwrap_or_default()))
        .with_controls(ViewControls::from_controls([control]));

    Ok(view_definition)
}

fn parse_view_definition_field(arguments: &Value) -> Result<NewViewField, ShellError> {
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
    Ok(NewViewField::new(
        field_name,
        parse_view_field_source_kind("read_model")
            .map_err(|error| ShellError::message(error.to_string()))?,
        read_model_name,
        source_field,
        sketch_token,
        provenance_description,
        bit_encoding,
    ))
}

fn parse_view_definition_control_input(
    arguments: &Value,
) -> Result<NewControlInputProvision, ShellError> {
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
    Ok(NewControlInputProvision::new(
        control_input,
        control_input_source,
        control_input_description,
        control_input_sketch_token,
        control_input_visible,
        control_input_decision,
    ))
}

fn parse_view_definition_control(
    arguments: &Value,
    navigation: NewNavigationTarget,
) -> Result<NewControlDefinition, ShellError> {
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
    let control_input = parse_view_definition_control_input(arguments)?;
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
    Ok(NewControlDefinition::new(
        control_name,
        control_command,
        control_input,
        CommandErrorNames::from_names(handled_errors),
        recovery_behavior,
        control_sketch_token,
        navigation,
    ))
}

fn parse_view_definition_navigation(arguments: &Value) -> Result<NewNavigationTarget, ShellError> {
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
    match (external_workflow, external_system, handoff_contract) {
        (Some(raw_external_workflow), None, None) => {
            let external_workflow = parse_navigation_target_name(raw_external_workflow)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(NewNavigationTarget::new(navigation_type, navigation_target)
                .with_external_workflow(external_workflow))
        }
        (None, Some(raw_external_system), Some(raw_handoff_contract)) => {
            let external_system = parse_navigation_target_name(raw_external_system)
                .map_err(|error| ShellError::message(error.to_string()))?;
            let handoff_contract = parse_payload_contract_name(raw_handoff_contract)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(NewNavigationTarget::new(navigation_type, navigation_target)
                .with_external_system(external_system, handoff_contract))
        }
        (None, None, None) => Ok(NewNavigationTarget::new(navigation_type, navigation_target)),
        _ => Err(ShellError::message(
            "add_view_definition requires either external_workflow alone or external_system and handoff_contract together",
        )),
    }
}

fn parse_view_definition_navigation_targets(
    arguments: &Value,
    key: &str,
) -> Result<Option<Vec<NavigationTargetName>>, ShellError> {
    arguments
        .get(key)
        .and_then(Value::as_str)
        .map(parse_navigation_target_names)
        .transpose()
        .map_err(|error| ShellError::message(error.to_string()))
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
    interpret_collect_reports(&command::update_workflow_description(slug, description))
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
    interpret_collect_reports(&command::update_workflow_name(slug, name))
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
    interpret_collect_reports(&command::update_slice_description(slug, description))
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
    interpret_collect_reports(&command::update_slice_kind(slug, kind))
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
    interpret_collect_reports(&command::update_slice_name(slug, name))
        .map(|reports| reports.join("\n"))
}

fn update_slice_scenario_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("update_slice_scenario requires arguments"))?;
    let scenario = parse_slice_scenario_tool_arguments(arguments, "update_slice_scenario")?;
    interpret_collect_reports(&command::update_slice_scenario(scenario))
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
    interpret_collect_reports(&command::remove_slice(slug)).map(|reports| reports.join("\n"))
}

fn remove_slice_scenario_tool_text(request: &Value) -> Result<String, ShellError> {
    let arguments = request
        .get("params")
        .and_then(|params| params.get("arguments"))
        .ok_or_else(|| ShellError::message("remove_slice_scenario requires arguments"))?;
    let slice_slug = arguments
        .get("slice")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_slice_scenario requires slice"))
        .and_then(|raw_slice| {
            parse_slice_slug(raw_slice).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let scenario_name = arguments
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("remove_slice_scenario requires name"))
        .and_then(|raw_name| {
            parse_scenario_name(raw_name).map_err(|error| ShellError::message(error.to_string()))
        })?;
    interpret_collect_reports(&command::remove_slice_scenario(slice_slug, scenario_name))
        .map(|reports| reports.join("\n"))
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
    interpret_collect_reports(&command::remove_workflow(slug)).map(|reports| reports.join("\n"))
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
    let source_control = arguments
        .get("source_control")
        .and_then(Value::as_str)
        .map(|raw_source_control| {
            parse_transition_trigger_name(raw_source_control)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()?;
    let target_view = arguments
        .get("target_view")
        .and_then(Value::as_str)
        .map(|raw_target_view| {
            parse_workflow_owned_definition_name(raw_target_view)
                .map_err(|error| ShellError::message(error.to_string()))
        })
        .transpose()?;
    let transition = ConnectionTransition {
        workflow_slug,
        source_slug,
        connection_kind,
        trigger,
    };
    let navigation_endpoints = ConnectionNavigationEndpoints {
        source_control,
        target_view,
    };
    let connection = build_workflow_connection(arguments, transition, navigation_endpoints)?;
    interpret_collect_reports(&command::connect_workflow(connection))
        .map(|reports| reports.join("\n"))
}

/// Common, already-parsed fields shared by every workflow connection variant.
struct ConnectionTransition {
    workflow_slug: WorkflowSlug,
    source_slug: SliceSlug,
    connection_kind: ConnectionKind,
    trigger: TransitionTriggerName,
}

/// Optional navigation endpoints required only for navigation connections.
struct ConnectionNavigationEndpoints {
    source_control: Option<TransitionTriggerName>,
    target_view: Option<WorkflowOwnedDefinitionName>,
}

fn build_workflow_connection(
    arguments: &Value,
    transition: ConnectionTransition,
    navigation_endpoints: ConnectionNavigationEndpoints,
) -> Result<WorkflowConnection, ShellError> {
    match arguments.get("to").and_then(Value::as_str) {
        Some(raw_target) => {
            let target_slug = parse_slice_slug(raw_target)
                .map_err(|error| ShellError::message(error.to_string()))?;
            build_intra_workflow_connection(
                arguments,
                transition,
                target_slug,
                navigation_endpoints,
            )
        }
        None => build_workflow_exit_connection(arguments, transition),
    }
}

fn build_intra_workflow_connection(
    arguments: &Value,
    transition: ConnectionTransition,
    target_slug: SliceSlug,
    navigation_endpoints: ConnectionNavigationEndpoints,
) -> Result<WorkflowConnection, ShellError> {
    let ConnectionTransition {
        workflow_slug,
        source_slug,
        connection_kind,
        trigger,
    } = transition;
    if connection_kind == ConnectionKind::Navigation {
        return Ok(WorkflowConnection::new_with_navigation_endpoints(
            workflow_slug,
            source_slug,
            target_slug,
            connection_kind,
            trigger,
            navigation_endpoints.source_control.ok_or_else(|| {
                ShellError::message(
                    "navigation workflow transitions require source_control owned by source slice",
                )
            })?,
            navigation_endpoints.target_view.ok_or_else(|| {
                ShellError::message(
                    "navigation workflow transitions require target_view owned by target slice",
                )
            })?,
        ));
    }
    match arguments.get("payload_contract").and_then(Value::as_str) {
        Some(raw_payload_contract) => {
            let payload_contract = parse_payload_contract_name(raw_payload_contract)
                .map_err(|error| ShellError::message(error.to_string()))?;
            Ok(WorkflowConnection::new_with_payload_contract(
                workflow_slug,
                source_slug,
                target_slug,
                connection_kind,
                trigger,
                payload_contract,
            ))
        }
        None => Ok(WorkflowConnection::new(
            workflow_slug,
            source_slug,
            target_slug,
            connection_kind,
            trigger,
        )),
    }
}

fn build_workflow_exit_connection(
    arguments: &Value,
    transition: ConnectionTransition,
) -> Result<WorkflowConnection, ShellError> {
    let ConnectionTransition {
        workflow_slug,
        source_slug,
        connection_kind,
        trigger,
    } = transition;
    let target_workflow = arguments
        .get("to_workflow")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("connect_workflow requires to or to_workflow"))
        .and_then(|raw_target| {
            parse_workflow_slug(raw_target).map_err(|error| ShellError::message(error.to_string()))
        })?;
    let reason = arguments
        .get("reason")
        .and_then(Value::as_str)
        .ok_or_else(|| ShellError::message("connect_workflow requires reason"))
        .and_then(|raw_reason| {
            parse_model_description(raw_reason)
                .map_err(|error| ShellError::message(error.to_string()))
        })?;
    Ok(WorkflowConnection::new_workflow_exit(
        workflow_slug,
        source_slug,
        target_workflow,
        connection_kind,
        trigger,
        reason,
    ))
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
    interpret_collect_reports(&command::remove_transition(removal))
        .map(|reports| reports.join("\n"))
}

fn tool_result(text: String) -> Value {
    mcp_model_value(CallToolResult::success(vec![Content::text(text)])).unwrap_or_else(|error| {
        unreachable!("EMC MCP tool result must serialize through the rmcp model: {error}");
    })
}

fn success_response(id: &Value, result: &Value) -> Value {
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

fn write_response(response: &Value) -> Result<(), ShellError> {
    let stdout = io::stdout();
    let mut lock = stdout.lock();
    serde_json::to_writer(&mut lock, response)
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
