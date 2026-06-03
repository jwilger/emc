#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn github_ci_runs_the_same_strict_local_gate() -> Result<(), Box<dyn Error>> {
        let workflow = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))?;

        assert!(
            workflow.contains("RUSTFLAGS: -Dwarnings"),
            "hosted CI must treat Rust warnings as errors"
        );
        assert!(
            workflow.contains("just ci"),
            "hosted CI must run the same strict gate as local development"
        );

        Ok(())
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
