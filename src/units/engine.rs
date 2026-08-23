use anyhow::{Context as _, Result, anyhow, bail};
use numbat::markup::plain_text_format;
use numbat::module_importer::BuiltinModuleImporter;
use numbat::resolver::CodeSource;
use numbat::value::Value;
use numbat::{Context, InterpreterResult, InterpreterSettings};
use rust_decimal::Decimal;

use super::normalize::{FuelUnit, LegacyConversion, SpecialUnit, TemperatureUnit};
use crate::format::{format_decimal, parse_decimal};

const COMPATIBILITY: &str = include_str!("compatibility.nbt");

/// Render-ready result returned by Numbat.
#[derive(Clone, Debug, PartialEq)]
pub struct UnitEvaluation {
    /// Plain-text value produced by Numbat.
    pub result: String,
    /// Optional legacy fact string for old shorthand queries.
    pub legacy_fact: Option<String>,
    /// Optional legacy dimension emoji.
    pub emoji: Option<&'static str>,
}

/// Metadata used by Alfred's `units` catalogue.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnitListing {
    /// Canonical Numbat identifier.
    pub identifier: String,
    /// Human-readable name.
    pub name: String,
    /// Human-readable dimension.
    pub dimension: String,
    /// Accepted identifiers and aliases.
    pub aliases: Vec<String>,
}

/// Lazy, isolated Numbat evaluation context.
pub struct UnitEngine {
    context: Context,
}

impl UnitEngine {
    /// Loads Numbat's prelude and the embedded legacy compatibility definitions.
    ///
    /// # Errors
    /// Returns an error if either module fails to load.
    pub fn new() -> Result<Self> {
        let mut context = Context::new(BuiltinModuleImporter::default());
        let mut settings = quiet_settings();
        let _ = context
            .interpret_with_settings(&mut settings, "use prelude", CodeSource::Internal)
            .map_err(|error| anyhow!(error.to_string()))
            .context("failed to load Numbat prelude")?;
        let _ = context
            .interpret_with_settings(&mut settings, COMPATIBILITY, CodeSource::Internal)
            .map_err(|error| anyhow!(error.to_string()))
            .context("failed to load legacy unit compatibility definitions")?;
        Ok(Self { context })
    }

    /// Evaluates a native Numbat expression without rewriting it.
    ///
    /// # Errors
    /// Returns an error for invalid expressions or statements without a value.
    pub fn evaluate_native(&mut self, expression: &str) -> Result<UnitEvaluation> {
        let value = self.evaluate(expression)?;
        Ok(UnitEvaluation {
            result: plain_value(&value),
            legacy_fact: None,
            emoji: None,
        })
    }

    /// Evaluates historical Alfred shorthand through Numbat.
    ///
    /// # Errors
    /// Returns an error for incompatible units or failed evaluation.
    pub fn evaluate_legacy(&mut self, conversion: LegacyConversion<'_>) -> Result<UnitEvaluation> {
        if conversion.from.dimension != conversion.to.dimension {
            bail!(
                "Can not convert {} to \"{}\"",
                conversion.from.symbol,
                conversion.to.symbol
            );
        }
        let converted = self.legacy_value(conversion.amount, conversion.from, conversion.to)?;
        let single = self.legacy_value("1", conversion.from, conversion.to)?;
        let amount =
            parse_decimal(conversion.amount).ok_or_else(|| anyhow!("invalid decimal amount"))?;
        Ok(UnitEvaluation {
            result: format!(
                "{} {} = {} {}",
                format_decimal(amount),
                conversion.from.symbol,
                format_decimal(converted),
                conversion.to.symbol
            ),
            legacy_fact: Some(format!(
                "Based on the fact that 1 {} = {} {}",
                conversion.from.symbol,
                format_decimal(single),
                conversion.to.symbol
            )),
            emoji: Some(conversion.from.emoji),
        })
    }

    /// Returns all public, non-money units loaded from Numbat metadata.
    #[must_use]
    pub fn listings(&self) -> Vec<UnitListing> {
        let mut units = self
            .context
            .unit_representations()
            .filter_map(|(identifier, (_, metadata))| {
                let dimension = plain_text_format(&metadata.readable_type, false).to_string();
                if identifier.starts_with('_') || dimension == "Money" {
                    return None;
                }
                let name = metadata
                    .name
                    .unwrap_or_else(|| identifier.clone())
                    .to_string();
                let mut aliases = metadata
                    .aliases
                    .into_iter()
                    .map(|(alias, _)| alias.to_string())
                    .collect::<Vec<_>>();
                aliases.sort_by_key(|alias| alias.to_lowercase());
                aliases.dedup();
                Some(UnitListing {
                    identifier: identifier.to_string(),
                    name,
                    dimension,
                    aliases,
                })
            })
            .collect::<Vec<_>>();
        units.sort_by_key(|unit| (unit.dimension.to_lowercase(), unit.name.to_lowercase()));
        units.dedup_by(|left, right| left.identifier == right.identifier);
        units
    }

    fn evaluate(&mut self, expression: &str) -> Result<Value> {
        let mut settings = quiet_settings();
        let (_, result) = self
            .context
            .interpret_with_settings(&mut settings, expression, CodeSource::Text)
            .map_err(|error| anyhow!(error.to_string()))?;
        match result {
            InterpreterResult::Value(value) => Ok(value),
            InterpreterResult::Continue => bail!("Numbat expression did not produce a value"),
        }
    }

    fn legacy_value(
        &mut self,
        amount: &str,
        from: super::normalize::LegacyUnit,
        to: super::normalize::LegacyUnit,
    ) -> Result<Decimal> {
        let expression = legacy_expression(amount, from, to)?;
        match self.evaluate(&expression)? {
            Value::Quantity(quantity) => legacy_decimal(quantity.unsafe_value().to_f64())
                .ok_or_else(|| anyhow!("Numbat returned a non-finite legacy result")),
            value => {
                let rendered = plain_value(&value);
                parse_decimal(&rendered)
                    .ok_or_else(|| anyhow!("Numbat returned {rendered} instead of a quantity"))
            }
        }
    }
}

fn legacy_decimal(value: f64) -> Option<Decimal> {
    let nearest_integer = value.round();
    let unit_in_last_place = (value.next_up() - value).abs();
    let value = if (value - nearest_integer).abs() <= unit_in_last_place {
        nearest_integer
    } else {
        value
    };
    Decimal::from_f64_retain(value)
}

fn quiet_settings() -> InterpreterSettings {
    InterpreterSettings {
        print_fn: Box::new(|_| {}),
    }
}

fn plain_value(value: &Value) -> String {
    plain_text_format(&value.pretty_print(), false)
        .trim()
        .to_owned()
}

fn legacy_expression(
    amount: &str,
    from: super::normalize::LegacyUnit,
    to: super::normalize::LegacyUnit,
) -> Result<String> {
    match (from.special, to.special) {
        (SpecialUnit::None, SpecialUnit::None) | (SpecialUnit::Pace, SpecialUnit::Pace) => Ok(
            format!("({amount} * ({})) to ({})", from.expression, to.expression),
        ),
        (SpecialUnit::None, SpecialUnit::Pace) | (SpecialUnit::Pace, SpecialUnit::None) => {
            Ok(format!(
                "(1 / ({amount} * ({}))) to ({})",
                from.expression, to.expression
            ))
        }
        (SpecialUnit::Fuel(from), SpecialUnit::Fuel(to)) => Ok(fuel_expression(amount, from, to)),
        (SpecialUnit::Temperature(from), SpecialUnit::Temperature(to)) => {
            Ok(temperature_expression(amount, from, to))
        }
        _ => bail!("legacy unit families are incompatible"),
    }
}

fn fuel_expression(amount: &str, from: FuelUnit, to: FuelUnit) -> String {
    if from == to {
        return amount.to_owned();
    }
    let to_base = match from {
        FuelUnit::KilometersPerLiter => amount.to_owned(),
        FuelUnit::LitersPer100Kilometers => {
            format!("legacy_l_per_100km_to_km_per_l({amount})")
        }
        FuelUnit::MilesPerUsGallon => format!("legacy_us_mpg_to_km_per_l({amount})"),
        FuelUnit::MilesPerImperialGallon => {
            format!("legacy_imperial_mpg_to_km_per_l({amount})")
        }
    };
    match to {
        FuelUnit::KilometersPerLiter => to_base,
        FuelUnit::LitersPer100Kilometers => {
            format!("legacy_km_per_l_to_l_per_100km({to_base})")
        }
        FuelUnit::MilesPerUsGallon => format!("legacy_km_per_l_to_us_mpg({to_base})"),
        FuelUnit::MilesPerImperialGallon => {
            format!("legacy_km_per_l_to_imperial_mpg({to_base})")
        }
    }
}

fn temperature_expression(amount: &str, from: TemperatureUnit, to: TemperatureUnit) -> String {
    if from == to {
        return amount.to_owned();
    }
    let celsius = match from {
        TemperatureUnit::Celsius => amount.to_owned(),
        TemperatureUnit::Fahrenheit => format!("(({amount}) - 32) * 5 / 9"),
        TemperatureUnit::Kelvin => format!("({amount}) - 273.15"),
        TemperatureUnit::Reamur => format!("legacy_reamur_to_celsius({amount})"),
        TemperatureUnit::Romer => format!("legacy_romer_to_celsius({amount})"),
        TemperatureUnit::Delisle => format!("legacy_delisle_to_celsius({amount})"),
        TemperatureUnit::Rankine => format!("({amount}) * 5 / 9 - 273.15"),
    };
    match to {
        TemperatureUnit::Celsius => celsius,
        TemperatureUnit::Fahrenheit => format!("({celsius}) * 9 / 5 + 32"),
        TemperatureUnit::Kelvin => format!("({celsius}) + 273.15"),
        TemperatureUnit::Reamur => format!("legacy_celsius_to_reamur({celsius})"),
        TemperatureUnit::Romer => format!("legacy_celsius_to_romer({celsius})"),
        TemperatureUnit::Delisle => format!("legacy_celsius_to_delisle({celsius})"),
        TemperatureUnit::Rankine => format!("(({celsius}) + 273.15) * 9 / 5"),
    }
}
