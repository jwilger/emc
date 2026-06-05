// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::process::Command;

    use assert_cmd::assert::Assert;
    use predicates::prelude::predicate;
    use tempfile::TempDir;

    fn repo_root() -> &'static str {
        env!("CARGO_MANIFEST_DIR")
    }

    fn header_tool() -> Command {
        let mut command = Command::new(format!("{}/scripts/copyright-headers.sh", repo_root()));
        command.current_dir(repo_root());
        command
    }

    #[test]
    fn copyright_header_tool_reports_missing_headers() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let source_file = temp_dir.path().join("missing.rs");
        fs::write(&source_file, "fn main() {}\n")?;

        Assert::new(header_tool().arg("--check").arg(&source_file).output()?)
            .failure()
            .stderr(predicate::str::contains("missing copyright header"))
            .stderr(predicate::str::contains(source_file.display().to_string()));

        Ok(())
    }

    #[test]
    fn copyright_header_tool_adds_language_appropriate_headers() -> Result<(), Box<dyn Error>> {
        let temp_dir = TempDir::new()?;
        let rust_file = temp_dir.path().join("source.rs");
        let markdown_file = temp_dir.path().join("notes.md");

        fs::write(&rust_file, "fn main() {}\n")?;
        fs::write(&markdown_file, "# Notes\n")?;

        Assert::new(
            header_tool()
                .arg("--fix")
                .arg(&rust_file)
                .arg(&markdown_file)
                .output()?,
        )
        .success();

        assert_eq!(
            fs::read_to_string(&rust_file)?,
            "// Copyright 2026 John Wilger\n\nfn main() {}\n"
        );
        assert_eq!(
            fs::read_to_string(&markdown_file)?,
            "<!-- Copyright 2026 John Wilger -->\n\n# Notes\n"
        );

        Ok(())
    }
}
