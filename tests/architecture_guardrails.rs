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

    fn forbidden_io_markers() -> &'static [&'static str] {
        &[
            "std::fs",
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
