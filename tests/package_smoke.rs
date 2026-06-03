#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::fs;
    use std::path::PathBuf;

    #[test]
    fn nix_flake_check_builds_the_packaged_emc_app() -> Result<(), Box<dyn Error>> {
        let flake = fs::read_to_string(workspace_root().join("flake.nix"))?;
        let ci_workflow = fs::read_to_string(workspace_root().join(".github/workflows/ci.yml"))?;

        assert!(
            flake.contains("checks = pkgs.lib.optionalAttrs hasCargoProject"),
            "flake checks must exist so nix flake check exercises the packaged EMC app"
        );
        assert!(
            flake.contains("default = package;"),
            "default flake check must build the EMC package"
        );
        assert!(
            flake.contains("emc = package;"),
            "named emc flake check must build the EMC package"
        );
        assert!(
            ci_workflow.contains("nix flake check"),
            "hosted CI must run the Nix package smoke gate"
        );

        Ok(())
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
