//! Stable English-number formatting shared by currency and legacy unit results.

use rust_decimal::Decimal;
use rust_decimal::RoundingStrategy;

/// Parses ordinary or scientific decimal input.
#[must_use]
pub fn parse_decimal(value: &str) -> Option<Decimal> {
    value
        .parse()
        .ok()
        .or_else(|| Decimal::from_scientific(value).ok())
}

/// Formats a decimal like Dart's default `NumberFormat` used by the workflow.
#[must_use]
pub fn format_decimal(value: Decimal) -> String {
    let rounded = value
        .round_dp_with_strategy(3, RoundingStrategy::MidpointAwayFromZero)
        .normalize();
    let raw = rounded.to_string();
    let (sign, unsigned) = raw
        .strip_prefix('-')
        .map_or(("", raw.as_str()), |rest| ("-", rest));
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |parts| (parts.0, Some(parts.1)));

    let mut grouped = String::with_capacity(raw.len() + integer.len() / 3);
    grouped.push_str(sign);
    for (index, character) in integer.chars().enumerate() {
        if index > 0 && (integer.len() - index).is_multiple_of(3) {
            grouped.push(',');
        }
        grouped.push(character);
    }
    if let Some(fraction) = fraction {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    grouped
}

#[cfg(test)]
#[path = "tests/format.rs"]
mod tests;
