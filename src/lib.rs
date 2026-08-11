//! groundcontrol library facade.
//!
//! Exposes the core types, grounding strategies, and OpenSpec parsing as a
//! library so that consumer crates (e.g. veriplan) can use
//! `groundcontrol::types`, `groundcontrol::grounders`, and
//! `groundcontrol::parse` directly, in addition to the standalone binary
//! (`groundcontrol check ...`).

pub mod grounders;
pub mod parse;
pub mod types;
