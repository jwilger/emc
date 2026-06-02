#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};
    use std::error::Error;
    use std::io;
    use std::process::Command;

    #[test]
    fn cargo_manifest_enumerates_clippy_all_lints_exactly() -> Result<(), Box<dyn Error>> {
        let manifest = include_str!("../Cargo.toml");
        let clippy_lints_section_body = clippy_lint_section_body(manifest)?;
        let all_lint_names = current_clippy_all_lint_names()?;
        let configured_lint_levels = configured_clippy_lint_levels(clippy_lints_section_body)?;
        let exact_deny_carve_outs = BTreeSet::from([
            "diverging_sub_expression".to_owned(),
            "enum_variant_names".to_owned(),
            "expect_used".to_owned(),
            "needless_return".to_owned(),
            "question_mark".to_owned(),
        ]);

        let violations = all_lint_names
            .iter()
            .filter_map(|lint_name| {
                let expected_level = if exact_deny_carve_outs.contains(lint_name) {
                    "deny"
                } else {
                    "forbid"
                };

                (configured_lint_levels.get(lint_name).map(String::as_str) != Some(expected_level))
                    .then(|| format!("{lint_name} must be configured as {expected_level}"))
            })
            .chain(configured_lint_levels.contains_key("all").then(|| {
                "remove group-level `all = ...`; enumerate exact clippy::all lints instead"
                    .to_owned()
            }))
            .collect::<Vec<_>>();

        assert_eq!(
            violations,
            Vec::<String>::new(),
            "[lints.clippy] must enumerate every current clippy::all lint"
        );

        Ok(())
    }

    fn clippy_lint_section_body(manifest_text: &str) -> Result<&str, Box<dyn Error>> {
        let header = "[lints.clippy]\n";
        let body_start = manifest_text
            .find(header)
            .map(|index| index + header.len())
            .ok_or_else(|| io::Error::other("Cargo.toml must define [lints.clippy]"))?;
        let rest = &manifest_text[body_start..];
        let body_end = rest
            .find("\n[")
            .map_or(manifest_text.len(), |offset| body_start + offset + 1);

        Ok(&manifest_text[body_start..body_end])
    }

    fn current_clippy_all_lint_names() -> Result<BTreeSet<String>, Box<dyn Error>> {
        let output = Command::new("clippy-driver")
            .args(["-W", "help"])
            .output()?;
        assert!(
            output.status.success(),
            "clippy-driver -W help must run successfully"
        );

        let stdout = String::from_utf8(output.stdout)?;
        let members = stdout
            .lines()
            .find_map(|line| {
                let trimmed = line.trim_start();
                trimmed
                    .split_once(char::is_whitespace)
                    .and_then(|(name, members)| (name == "clippy::all").then_some(members.trim()))
            })
            .map(str::trim)
            .ok_or_else(|| io::Error::other("clippy-driver -W help must list clippy::all"))?;

        Ok(members
            .split(',')
            .map(|lint_name| {
                lint_name
                    .trim()
                    .trim_start_matches("clippy::")
                    .replace('-', "_")
            })
            .collect())
    }

    fn configured_clippy_lint_levels(
        section_body: &str,
    ) -> Result<BTreeMap<String, String>, Box<dyn Error>> {
        section_body
            .lines()
            .filter_map(|line| {
                let trimmed = line.trim();
                (!trimmed.is_empty() && !trimmed.starts_with('#')).then_some(trimmed)
            })
            .map(|lint_line| {
                let (lint_name, level) = lint_line
                    .split_once('=')
                    .ok_or_else(|| io::Error::other("lint line must contain level assignment"))?;
                let quoted_level = level
                    .split('"')
                    .nth(1)
                    .ok_or_else(|| io::Error::other("lint level must be quoted"))?;
                Ok((lint_name.trim().to_owned(), quoted_level.to_owned()))
            })
            .collect()
    }
}
