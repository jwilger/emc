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
    fn event_model_validation_does_not_perform_filesystem_io_directly() -> Result<(), Box<dyn Error>>
    {
        let source = read_workspace_file("src/event_model_validation.rs")?;
        let violations = filesystem_io_markers()
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/event_model_validation.rs contains `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "validation must receive file contents through shell-interpreted effects"
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
    fn shell_check_transition_markers_use_semantic_workflow_documents() -> Result<(), Box<dyn Error>>
    {
        let source = read_workspace_file("src/shell.rs")?;
        let violations = ["fn transition_label(", "fn workflow_exit_transition_label("]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/shell.rs duplicates transition semantics via `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "shell check markers must derive workflow transitions from WorkflowDocument"
        );

        Ok(())
    }

    #[test]
    fn shell_check_slice_markers_use_semantic_workflow_documents() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/shell.rs")?;
        let violations = [
            "get(\"slice\")",
            "get(\"name\")",
            "get(\"type\")",
            "get(\"description\")",
        ]
        .iter()
        .filter(|marker| source.contains(**marker))
        .map(|marker| format!("src/shell.rs duplicates slice-detail semantics via `{marker}`"))
        .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "shell check markers must derive workflow slice details from WorkflowDocument"
        );

        Ok(())
    }

    #[test]
    fn shell_slice_file_references_use_semantic_workflow_documents() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/shell.rs")?;
        let violations = ["get(\"slice_files\")"]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/shell.rs duplicates slice-file semantics via `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "shell slice-file traversal must derive from WorkflowDocument"
        );

        Ok(())
    }

    #[test]
    fn validation_slice_file_references_use_semantic_workflow_documents()
    -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/event_model_validation.rs")?;
        let violations = ["get(\"slice_files\")"]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| {
                format!(
                    "src/event_model_validation.rs duplicates slice-file semantics via `{marker}`"
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "validation slice-file traversal must derive from WorkflowDocument"
        );

        Ok(())
    }

    #[test]
    fn shell_browser_index_paths_use_boundary_parser() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/shell.rs")?;
        let violations = ["get(\"workflows\")", "workflow.get(\"path\")"]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| format!("src/shell.rs duplicates browser-index semantics via `{marker}`"))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "shell browser-index path checks must derive from the boundary parser"
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
    fn browser_main_path_uses_semantic_workflow_documents() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/core/browser.rs")?;
        let violations = ["fn workflow_main_path_names("]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| {
                format!(
                    "src/core/browser.rs duplicates workflow main-path semantics via `{marker}`"
                )
            })
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "browser composition must derive main-path workflow steps from WorkflowDocument"
        );

        Ok(())
    }

    #[test]
    fn browser_branch_cards_use_semantic_workflow_documents() -> Result<(), Box<dyn Error>> {
        let source = read_workspace_file("src/core/browser.rs")?;
        let violations = ["fn workflow_branch_cards(", "fn workflow_branch_label("]
            .iter()
            .filter(|marker| source.contains(**marker))
            .map(|marker| {
                format!("src/core/browser.rs duplicates workflow branch semantics via `{marker}`")
            })
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "browser composition must derive branch cards from WorkflowDocument"
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
