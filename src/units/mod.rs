//! Numbat-backed physical-unit evaluation and legacy syntax normalization.

mod engine;
mod normalize;

pub use engine::{UnitEngine, UnitEvaluation, UnitListing};
pub use normalize::{LegacyConversion, legacy_conversion, legacy_unit};

#[cfg(test)]
#[path = "../tests/units.rs"]
mod tests;
