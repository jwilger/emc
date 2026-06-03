#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn justfile_exposes_manual_mutation_testing_without_adding_it_to_ci()
    -> Result<(), Box<dyn Error>> {
        let justfile = fs::read_to_string(workspace_root().join("justfile"))?;

        assert!(
            justfile.contains("mutants-diff:"),
            "justfile must expose a diff-scoped mutation testing gate"
        );
        assert!(
            justfile.contains("git diff HEAD"),
            "diff-scoped mutation testing must build a diff from touched code"
        );
        assert!(
            justfile.contains("cargo mutants --in-diff"),
            "diff-scoped mutation testing must target touched code"
        );
        assert!(
            justfile.contains("mutants-core:"),
            "justfile must expose a focused core mutation testing gate"
        );
        assert!(
            justfile.contains("--file src/core/workflow.rs"),
            "core mutation testing must include workflow mutations"
        );
        assert!(
            justfile.contains("--file src/core/slice.rs"),
            "core mutation testing must include slice mutations"
        );
        assert!(
            justfile.contains("--file src/core/connection.rs"),
            "core mutation testing must include connection mutations"
        );
        assert!(
            justfile.contains("mutants-full:"),
            "justfile must expose an explicit full mutation testing gate"
        );
        assert!(
            justfile.contains("--cap-lints true"),
            "mutation tests must cap strict lints so survivors measure behavior, not warning policy"
        );

        let ci_recipe = justfile
            .lines()
            .find(|line| line.starts_with("ci:"))
            .ok_or("missing ci recipe")?;
        assert!(
            !ci_recipe.contains("mutants"),
            "normal ci must stay fast; mutation testing is a manual engineering gate"
        );

        Ok(())
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
