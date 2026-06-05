#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn cargo_manifest_pins_latest_nutype_for_semantic_types() -> Result<(), Box<dyn Error>> {
        let manifest = read_workspace_file("Cargo.toml")?;

        assert!(
            manifest.contains("nutype = { version = \"0.7.0\""),
            "Cargo.toml must pin nutype 0.7.0 for semantic data types"
        );

        Ok(())
    }

    #[test]
    fn core_modules_do_not_perform_io_directly() -> Result<(), Box<dyn Error>> {
        let violations = rust_files_under("src/core")?
            .into_iter()
            .map(|path| {
                fs::read_to_string(&path).map(|source| {
                    forbidden_io_markers()
                        .iter()
                        .filter(move |marker| source.contains(**marker))
                        .map(move |marker| format!("{} contains `{marker}`", path.display()))
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "core modules must describe I/O as effects and leave execution to shell interpreters"
        );

        Ok(())
    }

    #[test]
    fn cli_entrypoint_does_not_perform_filesystem_io_directly() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/main.rs")?;
        let violations = filesystem_io_markers()
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/main.rs contains `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "CLI commands must describe file reads as effects and leave execution to shell interpreters"
        );

        Ok(())
    }

    #[test]
    fn cli_entrypoint_uses_boundary_parsers_for_project_names() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/main.rs")?;
        let violations = ["ProjectName::try_new"]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/main.rs bypasses DTO project-name parsing via `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "CLI boundary parsing must convert raw project names through DTO parsers before command execution"
        );

        Ok(())
    }

    #[test]
    fn entrypoints_use_boundary_parsers_for_project_paths() -> Result<(), Box<dyn Error>> {
        let violations = ["src/main.rs", "src/mcp.rs"]
            .iter()
            .map(|relative_path| {
                read_workspace_file(relative_path).map(|source| {
                    source
                        .lines()
                        .enumerate()
                        .filter(|(_, line)| line.contains("ProjectPath::try_new"))
                        .map(|(index, line)| {
                            format!(
                                "{relative_path}:{} bypasses DTO project-path parsing: {line}",
                                index + 1
                            )
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "CLI and MCP boundary parsing must convert raw project paths through DTO parsers before command execution"
        );

        Ok(())
    }

    #[test]
    fn mcp_tool_handlers_do_not_perform_filesystem_io_directly() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/mcp.rs")?;
        let violations = filesystem_io_markers()
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/mcp.rs contains `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "MCP tools must route project file access through shell-interpreted effects"
        );

        Ok(())
    }

    #[test]
    fn mcp_tool_handlers_route_through_shared_command_plans() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/mcp.rs")?;
        let violations = ["EffectPlan", "Effect::"]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/mcp.rs constructs command effects directly via `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "MCP tools must route through the same semantic command-planning layer as CLI commands"
        );

        Ok(())
    }

    #[test]
    fn mcp_protocol_payloads_use_the_pinned_sdk_model_types() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/mcp.rs")?;
        let required_markers = [
            "rmcp::model",
            "InitializeResult",
            "ServerCapabilities",
            "Implementation",
            "Tool",
            "CallToolResult",
            "Content",
        ];
        let violations = required_markers
            .iter()
            .filter(|marker| !source.contains(**marker))
            .map(|marker| format!("src/mcp.rs does not use `{marker}` for MCP payloads"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "MCP protocol payloads must be built from the pinned rmcp SDK model types instead of ad hoc JSON shapes"
        );

        Ok(())
    }

    #[test]
    fn core_public_apis_do_not_expose_primitive_types() -> Result<(), Box<dyn Error>> {
        let violations = rust_files_under("src/core")?
            .into_iter()
            .map(|path| {
                fs::read_to_string(&path).map(|source| {
                    source
                        .lines()
                        .enumerate()
                        .filter(|(_, line)| line.trim_start().starts_with("pub "))
                        .filter(|(_, line)| {
                            forbidden_primitive_markers()
                                .iter()
                                .any(|marker| line.contains(marker))
                        })
                        .map(move |(index, line)| {
                            format!(
                                "{}:{} exposes primitive data: {line}",
                                path.display(),
                                index + 1
                            )
                        })
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "public core APIs must expose semantic types, not primitives"
        );

        Ok(())
    }

    #[test]
    fn layout_public_apis_use_semantic_collections() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/core/layout.rs")?;
        let mut violations = Vec::new();
        let mut signature_start = None;
        let mut signature = String::new();

        for (index, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("pub fn ") {
                signature_start = Some(index + 1);
                signature.clear();
            }

            if signature_start.is_some() {
                signature.push_str(line.trim());
                signature.push(' ');
                if line.contains('{') {
                    if signature.contains("Vec<") {
                        let Some(start) = signature_start else {
                            return Err("signature parser lost its start line".into());
                        };
                        violations.push(format!(
                            "src/core/layout.rs:{} exposes a structural collection in public API: {}",
                            start,
                            signature.trim()
                        ));
                    }
                    signature_start = None;
                    signature.clear();
                }
            }
        }

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "layout command-planning APIs must accept semantic collection types, not raw Vec<T>"
        );

        Ok(())
    }

    #[test]
    fn effect_public_apis_use_semantic_collections() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/core/effect.rs")?;
        let mut violations = Vec::new();
        let mut signature_start = None;
        let mut signature = String::new();

        for (index, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("pub fn ") {
                signature_start = Some(index + 1);
                signature.clear();
            }

            if signature_start.is_some() {
                signature.push_str(line.trim());
                signature.push(' ');
                if line.contains('{') {
                    if ["Vec<", "-> &["]
                        .iter()
                        .any(|marker| signature.contains(marker))
                    {
                        let Some(start) = signature_start else {
                            return Err("signature parser lost its start line".into());
                        };
                        violations.push(format!(
                            "src/core/effect.rs:{} exposes a structural collection in public API: {}",
                            start,
                            signature.trim()
                        ));
                    }
                    signature_start = None;
                    signature.clear();
                }
            }
        }

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "effect APIs must expose semantic collection types, not raw Vec<T> or slices"
        );

        Ok(())
    }

    #[test]
    fn core_public_apis_use_semantic_collections() -> Result<(), Box<dyn Error>> {
        let workspace = workspace_root();
        let violations = rust_files_under("src/core")?
            .into_iter()
            .map(|path| {
                let relative_path = path.strip_prefix(&workspace)?;
                public_collection_signature_violations(relative_path)
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "core public APIs must expose semantic collection types, not raw Vec<T> or slices"
        );

        Ok(())
    }

    #[test]
    fn workflow_mutation_core_uses_semantic_json_document_types() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/core/workflow.rs")?;
        let violations = [
            "serde_json::Value",
            "Value::",
            "<Value>",
            "&Value",
            " Value",
        ]
        .iter()
        .filter(|marker| source.contains(**marker))
        .map(|marker| format!("src/core/workflow.rs manipulates raw JSON `{marker}`"))
        .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "workflow mutation logic must use semantic document types instead of raw JSON values"
        );

        Ok(())
    }

    #[test]
    fn slice_mutation_core_uses_semantic_json_document_types() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/core/slice.rs")?;
        let violations = [
            "serde_json::Value",
            "Value::",
            "<Value>",
            "&Value",
            " Value",
        ]
        .iter()
        .filter(|marker| source.contains(**marker))
        .map(|marker| format!("src/core/slice.rs manipulates raw JSON `{marker}`"))
        .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "slice mutation logic must use semantic document types instead of raw JSON values"
        );

        Ok(())
    }

    #[test]
    fn connection_mutation_core_uses_semantic_json_document_types() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/core/connection.rs")?;
        let violations = [
            "serde_json::Value",
            "Value::",
            "<Value>",
            "&Value",
            " Value",
        ]
        .iter()
        .filter(|marker| source.contains(**marker))
        .map(|marker| format!("src/core/connection.rs manipulates raw JSON `{marker}`"))
        .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "connection mutation logic must use semantic document types instead of raw JSON values"
        );

        Ok(())
    }

    #[test]
    fn artifact_digest_core_uses_semantic_json_document_types() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/core/digest.rs")?;
        let violations = raw_json_markers()
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/core/digest.rs manipulates raw JSON `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "artifact digests must derive from semantic workflow documents instead of raw JSON values"
        );

        Ok(())
    }

    #[test]
    fn shell_review_records_use_semantic_document_parser() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/shell.rs")?;
        let violations = [
            "get(\"workflow_slug\")",
            "get(\"status\")",
            "get(\"model_content_digest\")",
            "get(\"category_results\")",
            "get(\"mandatory_findings\")",
        ]
        .iter()
        .filter(|marker| source.contains(**marker))
        .map(|marker| format!("src/shell.rs duplicates review-record semantics via `{marker}`"))
        .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "shell review gate checks must derive from a semantic review-record document parser"
        );

        Ok(())
    }

    #[test]
    fn shell_json_object_checks_use_semantic_document_parser() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/shell.rs")?;
        let violations = ["serde_json::Value", "serde_json::from_str::<Value>"]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/shell.rs parses raw JSON values via `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "shell JSON object checks must use a semantic document parser"
        );

        Ok(())
    }

    #[test]
    fn formal_transition_emission_uses_semantic_transition_records() -> Result<(), Box<dyn Error>> {
        let violations = [
            ("src/core/emit/lean.rs", "split_once(\"->\")"),
            ("src/core/emit/lean.rs", "fn transition_parts("),
            ("src/core/emit/quint.rs", "split_once(\"->\")"),
            ("src/core/emit/quint.rs", "fn transition_parts("),
            ("src/shell.rs", "fn transition_record_parts("),
        ]
        .into_iter()
        .filter_map(|(path, marker)| {
            read_workspace_file(path).ok().and_then(|source| {
                source
                    .contains(marker)
                    .then(|| format!("{path} reparses transition labels via `{marker}`"))
            })
        })
        .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "formal transition emission and check markers must use semantic transition records"
        );

        Ok(())
    }

    #[test]
    fn check_project_uses_normalized_formal_graph_readers() -> Result<(), Box<dyn Error>> {
        let layout = read_workspace_file("src/core/layout.rs")?;
        let shell = read_workspace_file("src/shell.rs")?;
        let required_markers = [
            (&layout, "formal_workflows: FormalWorkflowGraphs"),
            (&layout, "formal_workflows.into_inner()"),
            (&layout, "formal_workflow_effects"),
            (&shell, "Effect::CheckCurrentProject =>"),
            (&shell, "read_synchronized_formal_workflow_graphs()?"),
            (&shell, "check_project(project_name, formal_workflows)"),
            (&shell, "parse_lean_workflow_graph"),
            (&shell, "parse_quint_workflow_graph"),
        ];
        let violations = required_markers
            .iter()
            .filter(|(source, marker)| !source.contains(*marker))
            .map(|(_, marker)| format!("formal graph check is missing `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "check_project must read Lean and Quint artifacts back into normalized semantic workflow graphs"
        );

        Ok(())
    }

    fn forbidden_io_markers() -> &'static [&'static str] {
        &[
            "std::fs",
            "fs::read_to_string",
            "tokio::fs",
            "std::process",
            "Command::new",
            "std::env",
            "std::net",
            "TcpListener",
            "TcpStream",
            "SystemTime",
            "Instant::now",
            "println!",
            "eprintln!",
        ]
    }

    fn filesystem_io_markers() -> &'static [&'static str] {
        &["std::fs", "fs::read_to_string"]
    }

    fn forbidden_primitive_markers() -> &'static [&'static str] {
        &[
            ": String",
            ": &str",
            ": str",
            ": PathBuf",
            ": &Path",
            ": Vec<String>",
            "-> String",
            "-> PathBuf",
            "-> Vec<String>",
        ]
    }

    fn raw_json_markers() -> &'static [&'static str] {
        &[
            "serde_json::Value",
            "Value::",
            "<Value>",
            "&Value",
            " Value",
            "serde_json::Map",
            "serde_json::from_str",
        ]
    }

    fn public_collection_signature_violations(
        relative_path: &Path,
    ) -> Result<Vec<String>, Box<dyn Error>> {
        let source = fs::read_to_string(workspace_root().join(relative_path))?;
        let mut violations = Vec::new();
        let mut signature_start = None;
        let mut signature = String::new();
        let display_path = relative_path.display();

        for (index, line) in source.lines().enumerate() {
            if line.trim_start().starts_with("pub fn ") {
                signature_start = Some(index + 1);
                signature.clear();
            }

            if signature_start.is_some() {
                signature.push_str(line.trim());
                signature.push(' ');
                if line.contains('{') {
                    if ["Vec<", "-> &["]
                        .iter()
                        .any(|marker| signature.contains(marker))
                    {
                        let Some(start) = signature_start else {
                            return Err("signature parser lost its start line".into());
                        };
                        violations.push(format!(
                            "{display_path}:{start} exposes a structural collection in public API: {}",
                            signature.trim()
                        ));
                    }
                    signature_start = None;
                    signature.clear();
                }
            }
        }

        Ok(violations)
    }

    fn rust_files_under(relative_root: &str) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        let root = workspace_root().join(relative_root);
        if !root.exists() {
            return Ok(Vec::new());
        }

        collect_rust_files(&root)
    }

    fn collect_rust_files(path: &Path) -> Result<Vec<PathBuf>, Box<dyn Error>> {
        if path.is_file() {
            return Ok(path
                .extension()
                .is_some_and(|extension| extension == "rs")
                .then(|| path.to_path_buf())
                .into_iter()
                .collect());
        }

        fs::read_dir(path)?
            .map(|entry| entry.map(|directory_entry| directory_entry.path()))
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|entry_path| collect_rust_files(&entry_path))
            .collect::<Result<Vec<_>, _>>()
            .map(|nested| nested.into_iter().flatten().collect())
    }

    fn read_workspace_file(relative_path: &str) -> Result<String, Box<dyn Error>> {
        fs::read_to_string(workspace_root().join(relative_path)).map_err(Into::into)
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
