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
            flake.contains("runtimeTools = [ pkgs.lean4 pkgs.quint ];"),
            "Nix package must pin Lean4 and Quint runtime tools"
        );
        assert!(
            flake.contains("craneLib.filterCargoSources"),
            "Nix package source filtering must preserve Cargo sources explicitly"
        );
        assert!(
            flake.contains("/browser"),
            "Nix package source filtering must preserve embedded browser assets"
        );
        assert!(
            flake.contains("/.github"),
            "Nix package source filtering must preserve CI metadata inspected by tests"
        );
        assert!(
            flake.contains("flake.nix"),
            "Nix package source filtering must preserve flake metadata inspected by tests"
        );
        assert!(
            flake.contains("/justfile"),
            "Nix package source filtering must preserve just recipes inspected by tests"
        );
        assert!(
            flake.contains("nativeBuildInputs = [ pkgs.makeWrapper ];"),
            "Nix package must wrap the EMC executable"
        );
        assert!(
            flake.contains("wrapProgram $out/bin/emc"),
            "Nix package must wrap emc so verification tools are hidden behind the binary"
        );
        assert!(
            flake.contains("--prefix PATH : ${pkgs.lib.makeBinPath runtimeTools}"),
            "wrapped EMC package must put Lean4 and Quint on PATH"
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
            flake.contains("${package}/bin/emc add workflow"),
            "package smoke checks must create a non-empty model before verification"
        );
        assert!(
            flake.contains("${package}/bin/emc add slice"),
            "package smoke checks must exercise packaged slice mutation"
        );
        assert!(
            flake.contains("${package}/bin/emc connect workflow"),
            "package smoke checks must exercise packaged workflow transition mutation"
        );
        assert!(
            flake.contains("${package}/bin/emc verify"),
            "package smoke checks must run packaged `emc verify` through pinned tools"
        );
        assert!(
            flake.contains("${package}/bin/emc review record"),
            "package smoke checks must run packaged `emc review record`"
        );
        assert!(
            flake.contains("${package}/bin/emc review gate"),
            "package smoke checks must run packaged `emc review gate`"
        );
        assert!(
            flake.contains("${package}/bin/emc generate site"),
            "package smoke checks must run packaged `emc generate site`"
        );
        assert!(
            flake.contains("pkgs.chromium"),
            "package smoke checks must use a Nix-provided headless browser for rendered site verification"
        );
        assert!(
            flake.contains("--headless")
                && flake.contains("--dump-dom")
                && flake.contains("--virtual-time-budget=5000"),
            "package smoke checks must execute the generated site in a real headless browser"
        );
        assert!(
            flake.contains("grep 'Package Smoke Event Model Browser' rendered-site.html")
                && flake.contains("grep 'Package smoke' rendered-site.html")
                && flake.contains("grep 'Capture smoke' rendered-site.html"),
            "package smoke checks must assert rendered project and model content, not only static site files"
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
            flake.contains("\"method\":\"tools/call\"")
                && flake.contains("\"name\":\"check_project\""),
            "package smoke HTTP checks must invoke a packaged EMC MCP tool, not only initialize the transport"
        );
        assert!(
            flake.contains("http_body_length=\"''${#http_body}\""),
            "package smoke HTTP request must compute Content-Length from the exact body"
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
            flake.contains("emc-container-image = containerImage;"),
            "flake checks must build the Docker-compatible EMC image"
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

    #[test]
    fn cargo_manifest_carries_the_planned_mcp_sdk_dependency() -> Result<(), Box<dyn Error>> {
        let manifest = fs::read_to_string(workspace_root().join("Cargo.toml"))?;

        assert!(
            manifest.contains("rmcp = { version = \"1.7.0\", default-features = false }"),
            "Cargo manifest must carry the planned rmcp dependency at the current selected version without unreviewed default features"
        );

        Ok(())
    }

    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
    }
}
