use std::error::Error;
use std::fmt::{Display, Formatter, Result as FormatResult};

use serde_json::{Value, json};

use crate::core::effect::FileContents;

pub fn ensure_navigation_control_in_slice(
    contents: &FileContents,
    navigation: &str,
) -> Result<FileContents, NavigationControlError> {
    let mut document = serde_json::from_str::<Value>(contents.as_ref())
        .map_err(|error| NavigationControlError::new(format!("invalid slice JSON: {error}")))?;
    let views = document
        .get_mut("views")
        .and_then(Value::as_array_mut)
        .ok_or_else(|| NavigationControlError::new("slice views must be an array"))?;

    ensure_local_navigation_target_view(views, navigation);
    let view = views
        .first_mut()
        .and_then(Value::as_object_mut)
        .ok_or_else(|| {
            NavigationControlError::new(format!(
                "slice has no source view to own navigation '{navigation}'"
            ))
        })?;

    ensure_navigation_control(view, navigation)?;
    let updated = serde_json::to_string_pretty(&document).map_err(|error| {
        NavigationControlError::new(format!("failed to serialize slice JSON: {error}"))
    })?;
    FileContents::try_new(format!("{updated}\n"))
        .map_err(|error| NavigationControlError::new(error.to_string()))
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NavigationControlError {
    message: String,
}

impl NavigationControlError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for NavigationControlError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        formatter.write_str(&self.message)
    }
}

impl Error for NavigationControlError {}

fn ensure_local_navigation_target_view(views: &mut Vec<Value>, navigation: &str) {
    if views.iter().any(|view| {
        view.get("name")
            .and_then(Value::as_str)
            .is_some_and(|name| name == navigation)
    }) {
        return;
    }

    views.push(json!({
        "name": navigation,
        "wireframe": "<section></section>",
        "uses_read_models": [],
        "controls": []
    }));
}

fn ensure_navigation_control(
    view: &mut serde_json::Map<String, Value>,
    navigation: &str,
) -> Result<(), NavigationControlError> {
    let controls = view
        .entry("controls")
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| NavigationControlError::new("source view controls must be an array"))?;

    if !controls.iter().any(|control| {
        control
            .get("navigation")
            .and_then(Value::as_str)
            .is_some_and(|modeled_navigation| modeled_navigation == navigation)
    }) {
        controls.push(json!({
            "label": navigation,
            "navigation": navigation,
            "navigation_type": "modeled_view"
        }));
    }

    let wireframe = view
        .get("wireframe")
        .and_then(Value::as_str)
        .unwrap_or("<section></section>");
    if !wireframe.contains(&format!("data-ref=\"{navigation}\"")) {
        view.insert(
            "wireframe".to_owned(),
            Value::String(wireframe_with_navigation(wireframe, navigation)),
        );
    }

    Ok(())
}

fn wireframe_with_navigation(wireframe: &str, navigation: &str) -> String {
    let button = format!("<button data-ref=\"{navigation}\"></button>");
    wireframe.strip_suffix("</section>").map_or_else(
        || format!("{wireframe}{button}"),
        |prefix| format!("{prefix}{button}</section>"),
    )
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use serde_json::Value;

    use super::*;

    #[test]
    fn ensure_navigation_control_preserves_unrelated_navigation_controls()
    -> Result<(), Box<dyn Error>> {
        let contents = FileContents::try_new(
            r#"{
  "name": "Capture intake",
  "views": [
    {
      "name": "capture-intake",
      "wireframe": "<section><button data-ref=\"existing-screen\"></button></section>",
      "uses_read_models": [],
      "controls": [
        {
          "label": "existing-screen",
          "navigation": "existing-screen",
          "navigation_type": "modeled_view"
        }
      ]
    }
  ]
}"#
            .to_owned(),
        )?;

        let updated = ensure_navigation_control_in_slice(&contents, "triage-screen")?;
        let document = serde_json::from_str::<Value>(updated.as_ref())?;
        let controls = document["views"][0]["controls"]
            .as_array()
            .ok_or("source view controls must remain an array")?;

        assert!(
            controls.iter().any(|control| {
                control["navigation"].as_str() == Some("existing-screen")
                    && control["navigation_type"].as_str() == Some("modeled_view")
            }),
            "existing navigation control must be preserved"
        );
        assert!(
            controls.iter().any(|control| {
                control["navigation"].as_str() == Some("triage-screen")
                    && control["navigation_type"].as_str() == Some("modeled_view")
            }),
            "new navigation control must be added when existing controls target other views"
        );
        assert_eq!(
            controls.len(),
            2,
            "adding a modeled navigation control must not duplicate existing controls"
        );

        Ok(())
    }
}
