// Copyright 2026 John Wilger

#[cfg(test)]
mod tests {
    use std::{error::Error, io};

    #[test]
    fn library_callers_can_retrieve_modeling_guidance() -> Result<(), Box<dyn Error>> {
        let catalog = emc::guidance_catalog();
        let topics = catalog.topics();
        let topic = topics
            .first()
            .ok_or_else(|| io::Error::other("missing modeling process topic"))?;

        assert_eq!(emc::VERSION, env!("CARGO_PKG_VERSION"));
        assert_eq!(topics.len(), 1);
        assert_eq!(topic.id(), "modeling-process");
        assert_eq!(topic.title(), "EMC Modeling Process");

        let guide = catalog
            .get("modeling-process")
            .ok_or_else(|| io::Error::other("missing modeling process guide"))?;

        assert_eq!(guide.id(), "modeling-process");
        assert_eq!(guide.title(), "EMC Modeling Process");
        assert_eq!(emc::modeling_process_guide(), guide.body());
        assert!(guide.body().contains("Phase-By-Phase Modeling Order"));
        assert!(guide.body().contains("Acceptance Scenarios"));
        assert!(guide.body().contains("external actor's point of view"));
        assert!(guide.body().contains("Contract Scenarios"));
        assert!(
            guide
                .body()
                .contains("internal provenance and traceability requirements")
        );

        Ok(())
    }
}
