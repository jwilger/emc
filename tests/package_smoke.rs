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
            flake.contains("packageSmoke = pkgs.runCommand"),
            "flake checks must include packaged EMC command smoke tests"
        );
        assert!(
            flake.contains("package-smoke = packageSmoke;"),
            "flake checks must export packaged EMC command smoke tests"
        );
        assert!(
            flake.contains("${package}/bin/emc check"),
            "package smoke checks must run packaged `emc check`"
        );
        assert!(
            flake.contains("${package}/bin/emc generate site"),
            "package smoke checks must run packaged `emc generate site`"
        );
        assert!(
            flake.contains("${package}/bin/emc mcp stdio"),
            "package smoke checks must run packaged `emc mcp stdio`"
        );
        assert!(
            flake.contains("${package}/bin/emc mcp http"),
            "package smoke checks must run packaged `emc mcp http`"
        );
        assert!(
            flake.contains("containerImage = pkgs.dockerTools.buildImage"),
            "flake packages must include a Docker-compatible EMC image"
        );
        assert!(
            flake.contains("emc-container = containerImage;"),
            "flake packages must export the Docker-compatible EMC image"
        );
        assert!(
            flake.contains("copyToRoot = pkgs.buildEnv"),
            "EMC container image must include a runnable package closure"
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
