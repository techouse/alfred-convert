//! Numbat-backed physical-unit evaluation and legacy syntax normalization.

mod engine;
mod normalize;

pub use engine::{UnitEngine, UnitEvaluation, UnitListing};
pub use normalize::{
    CustomarySystem, LegacyConversion, legacy_conversion, legacy_conversion_with_customary_system,
    legacy_unit, legacy_unit_with_customary_system,
};

#[cfg(test)]
#[path = "../tests/units.rs"]
mod tests;
