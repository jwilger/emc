// Copyright 2026 John Wilger

/// EMC crate version exposed to embedders.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Return the current EMC modeling guidance catalog.
#[inline]
#[must_use]
pub fn guidance_catalog() -> GuidanceCatalog {
    GuidanceCatalog::new()
}

/// Return the full EMC modeling process guide.
#[inline]
#[must_use]
pub fn modeling_process_guide() -> &'static str {
    MODELING_PROCESS_GUIDE.body()
}

/// Versioned guidance records available to embedders.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub struct GuidanceCatalog {
    _private: (),
}

impl GuidanceCatalog {
    /// Build a guidance catalog view over EMC's current bundled guidance.
    #[inline]
    #[must_use]
    pub const fn new() -> Self {
        Self { _private: () }
    }

    /// List all available guidance topics.
    #[inline]
    #[must_use]
    pub const fn topics(self) -> &'static [GuidanceTopic] {
        let Self { _private: () } = self;
        &[MODELING_PROCESS_GUIDE]
    }

    /// Retrieve a guidance topic by its stable id.
    #[inline]
    #[must_use]
    pub fn get(self, id: &str) -> Option<GuidanceTopic> {
        self.topics().iter().find(|topic| topic.id() == id).copied()
    }
}

impl Default for GuidanceCatalog {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

/// A single stable guidance topic.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct GuidanceTopic {
    id: &'static str,
    title: &'static str,
    body: &'static str,
}

impl GuidanceTopic {
    /// Stable identifier for the guidance topic.
    #[inline]
    #[must_use]
    pub const fn id(self) -> &'static str {
        self.id
    }

    /// Human-readable topic title.
    #[inline]
    #[must_use]
    pub const fn title(self) -> &'static str {
        self.title
    }

    /// Full guidance text.
    #[inline]
    #[must_use]
    pub const fn body(self) -> &'static str {
        self.body
    }
}

const MODELING_PROCESS_GUIDE: GuidanceTopic = GuidanceTopic {
    id: "modeling-process",
    title: "EMC Modeling Process",
    body: include_str!("../docs/event-model/modeling-process.md"),
};
