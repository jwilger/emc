# Copyright 2026 John Wilger

{
  description = "Development environment and release builds for EMC";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    crane = {
      url = "github:ipetkov/crane";
    };
  };

  outputs =
    { self
    , nixpkgs
    , flake-utils
    , fenix
    , crane
    }:
    let
      supportedSystems = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
    in
    flake-utils.lib.eachSystem supportedSystems (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ fenix.overlays.default ];
        };

        rustToolchain = pkgs.fenix.complete.withComponents [
          "cargo"
          "clippy"
          "rust-src"
          "rustc"
          "rustfmt"
        ];

        craneLib = (crane.mkLib pkgs).overrideToolchain rustToolchain;

        commonBuildInputs = with pkgs; [
          openssl
          sqlite
        ] ++ lib.optionals stdenv.isDarwin [
          darwin.apple_sdk.frameworks.AppKit
          darwin.apple_sdk.frameworks.CoreFoundation
          darwin.apple_sdk.frameworks.CoreServices
          darwin.apple_sdk.frameworks.Security
          darwin.apple_sdk.frameworks.SystemConfiguration
        ];

        commonNativeBuildInputs = with pkgs; [
          installShellFiles
          pkg-config
        ];

        cargoToml = ./Cargo.toml;
        hasCargoProject = builtins.pathExists cargoToml;
        packageSource = pkgs.lib.cleanSourceWith {
          src = ./.;
          filter = path: type:
            let
              pathText = builtins.toString path;
            in
            craneLib.filterCargoSources path type
            || pkgs.lib.hasSuffix "/flake.nix" pathText
            || pkgs.lib.hasSuffix "/justfile" pathText
            || pkgs.lib.hasSuffix "/.github" pathText
            || pkgs.lib.hasInfix "/.github/" pathText
            || pkgs.lib.hasSuffix "/docs" pathText
            || pkgs.lib.hasInfix "/docs/" pathText
            || pkgs.lib.hasSuffix "/tests" pathText
            || pkgs.lib.hasInfix "/tests/" pathText;
        };

        emcBinary = craneLib.buildPackage {
          src = packageSource;
          strictDeps = true;
          buildInputs = commonBuildInputs;
          nativeBuildInputs = commonNativeBuildInputs;
        };

        runtimeTools = [ pkgs.lean4 pkgs.quint ];

        package = pkgs.symlinkJoin {
          name = "emc";
          paths = [ emcBinary ];
          nativeBuildInputs = [ pkgs.makeWrapper ];
          postBuild = ''
            wrapProgram $out/bin/emc \
              --prefix PATH : ${pkgs.lib.makeBinPath runtimeTools}
          '';
        };

        containerImage = pkgs.dockerTools.buildImage {
          name = "emc";
          tag = "latest";
          copyToRoot = pkgs.buildEnv {
            name = "emc-container-root";
            paths = [ package ];
            pathsToLink = [ "/bin" ];
          };
          config = {
            Entrypoint = [ "${package}/bin/emc" ];
          };
        };

        app = flake-utils.lib.mkApp { drv = package; } // {
          meta.description = "Event Model Compiler";
        };

        overlayPkgs = import nixpkgs {
          inherit system;
          overlays = [ self.overlays.default ];
        };

        overlaySmoke = pkgs.runCommand "emc-overlay-smoke" { } ''
          test "${overlayPkgs.emc}" = "${package}"
          test "${overlayPkgs.emc-container}" = "${containerImage}"
          test -x "${overlayPkgs.emc}/bin/emc"
          grep '${pkgs.lean4}/bin' "${overlayPkgs.emc}/bin/emc"
          grep '${pkgs.quint}/bin' "${overlayPkgs.emc}/bin/emc"
          touch "$out"
        '';

        packageSmoke = pkgs.runCommand "emc-package-smoke"
          {
            nativeBuildInputs = [
              pkgs.netcat
            ];
          }
          ''
            workdir="$(mktemp -d)"
            cd "$workdir"

            ${package}/bin/emc init --name "Package Smoke"
            ${package}/bin/emc add workflow --slug package-smoke --name "Package smoke" --description "Packaged EMC verification smoke workflow."
            ${package}/bin/emc add slice --workflow package-smoke --slug capture-smoke --name "Capture smoke" --type state_view --description "Capture package smoke details."
            ${package}/bin/emc add slice --workflow package-smoke --slug review-smoke --name "Review smoke" --type state_view --description "Review package smoke details."
            ${package}/bin/emc connect workflow --workflow package-smoke --from capture-smoke --to review-smoke --via navigation --name review-smoke-screen
            ${package}/bin/emc check
            ${package}/bin/emc verify
            ${package}/bin/emc review record --workflow package-smoke --reviewer package-smoke --reviewed-at 2026-06-03T00:00:00.000Z
            ${package}/bin/emc review gate --workflow package-smoke

            printf '%s\n' \
              '{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"package-smoke","version":"0.0.0"}}}' \
              | ${package}/bin/emc mcp stdio \
              | grep '"serverInfo"'

            ${package}/bin/emc mcp http --host 127.0.0.1 --port 7332 > http.log &
            server_pid="$!"
            trap 'kill "$server_pid" || true; wait "$server_pid" || true' EXIT
            http_ready=0
            for attempt in $(seq 1 50); do
              http_body='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-11-25","capabilities":{},"clientInfo":{"name":"package-smoke","version":"0.0.0"}}}'
              http_body_length="''${#http_body}"
              if printf 'POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:7332\r\nOrigin: http://127.0.0.1:7332\r\nContent-Type: application/json\r\nContent-Length: %s\r\nConnection: close\r\n\r\n%s' "$http_body_length" "$http_body" \
                | nc 127.0.0.1 7332 \
                | grep '"serverInfo"'; then
                http_ready=1
                break
              fi
              sleep 0.1
            done
            if [ "$http_ready" != 1 ]; then
              cat http.log
              exit 1
            fi

            http_body='{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"check_project","arguments":{}}}'
            http_body_length="''${#http_body}"
            printf 'POST /mcp HTTP/1.1\r\nHost: 127.0.0.1:7332\r\nOrigin: http://127.0.0.1:7332\r\nContent-Type: application/json\r\nContent-Length: %s\r\nConnection: close\r\n\r\n%s' "$http_body_length" "$http_body" \
              | nc 127.0.0.1 7332 \
              | grep 'project layout is complete'

            trap - EXIT
            kill "$server_pid"
            wait "$server_pid" || true
            touch "$out"
            exit 0
          '';

      in
      {
        packages = pkgs.lib.optionalAttrs hasCargoProject {
          default = package;
          emc = package;
          emc-container = containerImage;
        };

        apps = pkgs.lib.optionalAttrs hasCargoProject {
          default = app;
          emc = app;
        };

        checks = pkgs.lib.optionalAttrs hasCargoProject {
          default = package;
          emc = package;
          emc-container-image = containerImage;
          overlay-smoke = overlaySmoke;
          package-smoke = packageSmoke;
        };

        devShells.default = pkgs.mkShell {
          packages = with pkgs; [
            rustToolchain
            cargo-deny
            cargo-mutants
            cargo-nextest
            cargo-watch
            forgejo-mcp
            jq
            just
            lefthook
            nodejs
            openssl
            pkg-config
            sqlite
          ] ++ runtimeTools;

          RUST_SRC_PATH = "${rustToolchain}/lib/rustlib/src/rust/library";
          OPENSSL_NO_VENDOR = "1";
          PKG_CONFIG_PATH = pkgs.lib.makeSearchPath "lib/pkgconfig" commonBuildInputs;
        };
      }) // {
        overlays.default = final: _previous: {
          emc = self.packages.${final.stdenv.hostPlatform.system}.emc;
          emc-container = self.packages.${final.stdenv.hostPlatform.system}.emc-container;
        };
      };
}
