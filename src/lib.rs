// Copyright 2026 John Wilger

#![cfg_attr(
    test,
    allow(
        dead_code,
        reason = "test-only modules expose helpers not all exercised in every build"
    )
)]

#[cfg(test)]
pub(crate) mod command;
#[cfg(test)]
pub(crate) mod core;
#[cfg(test)]
pub(crate) mod io;
#[cfg(test)]
pub(crate) mod mcp;
#[cfg(test)]
pub(crate) mod shell;

#[cfg(test)]
mod internal_semantic_tests;
