#[cfg(test)]
mod tests {
    use std::error::Error;

    use emc::{
        core::effect::ProjectPath,
        io::dto::{
            parse_lean_module_name, parse_model_digest, parse_model_name, parse_quint_module_name,
            parse_slice_slug, parse_workflow_slug,
        },
    };

    #[test]
    fn boundary_parsers_convert_raw_strings_to_semantic_types() -> Result<(), Box<dyn Error>> {
        let model_name = parse_model_name(" Repair Desk ")?;
        let workflow_slug = parse_workflow_slug(" Organization Access ")?;
        let slice_slug = parse_slice_slug(" Resolve Application Entry ")?;
        let lean_module = parse_lean_module_name(" RepairDesk ")?;
        let quint_module = parse_quint_module_name(" RepairDesk ")?;
        let digest = parse_model_digest(" abc123 ")?;

        assert_eq!(model_name.as_ref(), "Repair Desk", "model name is trimmed");
        assert_eq!(
            workflow_slug.as_ref(),
            "organization-access",
            "workflow slug is normalized"
        );
        assert_eq!(
            slice_slug.as_ref(),
            "resolve-application-entry",
            "slice slug is normalized"
        );
        assert_eq!(
            lean_module.as_ref(),
            "RepairDesk",
            "Lean module name is trimmed"
        );
        assert_eq!(
            quint_module.as_ref(),
            "RepairDesk",
            "Quint module name is trimmed"
        );
        assert_eq!(digest.as_ref(), "abc123", "digest is trimmed");

        Ok(())
    }

    #[test]
    fn boundary_parsers_reject_empty_semantic_values() {
        assert!(
            parse_model_name("   ").is_err(),
            "blank model names must not enter the core"
        );
        assert!(
            parse_workflow_slug("   ").is_err(),
            "blank workflow slugs must not enter the core"
        );
        assert!(
            parse_slice_slug("   ").is_err(),
            "blank slice slugs must not enter the core"
        );
        assert!(
            parse_lean_module_name("   ").is_err(),
            "blank Lean module names must not enter the core"
        );
        assert!(
            parse_quint_module_name("   ").is_err(),
            "blank Quint module names must not enter the core"
        );
        assert!(
            parse_model_digest("   ").is_err(),
            "blank model digests must not enter the core"
        );
    }

    #[test]
    fn project_paths_are_relative_to_the_current_project() -> Result<(), Box<dyn Error>> {
        let path = ProjectPath::try_new("model/browser/data/index.json".to_owned())?;

        assert_eq!(path.as_ref(), "model/browser/data/index.json");
        assert!(
            ProjectPath::try_new("/tmp/site".to_owned()).is_err(),
            "absolute paths must not enter project-local effects"
        );
        assert!(
            ProjectPath::try_new("../outside-model".to_owned()).is_err(),
            "parent traversal must not enter project-local effects"
        );

        Ok(())
    }
}
