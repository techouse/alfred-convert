//! ECB-backed monetary conversion.

mod cache;
mod catalog;
mod ecb;

use std::collections::BTreeMap;

use anyhow::{Result, anyhow, bail};
use jiff::Timestamp;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use cache::ExchangeRateCache;
pub use catalog::{CURRENCIES, Currency};
pub use ecb::{EcbClient, should_refresh};

/// A dated set of ECB base-Euro rates.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExchangeRates {
    /// Source publication timestamp.
    pub date: Timestamp,
    /// ISO code to Euro-relative rate.
    pub rates: BTreeMap<String, Decimal>,
}

impl ExchangeRates {
    /// Converts from one supported currency to another.
    ///
    /// # Errors
    /// Returns an error when either rate is absent or decimal arithmetic overflows.
    pub fn pair_rate(&self, from: Currency, to: Currency) -> Result<Decimal> {
        let from_rate = self
            .rates
            .get(from.code())
            .copied()
            .ok_or_else(|| anyhow!("Invalid from currency pair."))?;
        let to_rate = self
            .rates
            .get(to.code())
            .copied()
            .ok_or_else(|| anyhow!("Invalid to currency pair."))?;
        divide_like_dart(to_rate, from_rate)
    }
}

/// Divides decimal rates using Dart Decimal's finite-or-four-place rule.
pub(crate) fn divide_like_dart(numerator: Decimal, denominator: Decimal) -> Result<Decimal> {
    if denominator.is_zero() {
        bail!("currency rate denominator must not be zero");
    }

    let mut top = numerator
        .mantissa()
        .checked_mul(power_of_ten(denominator.scale())?)
        .ok_or_else(|| anyhow!("currency rate numerator overflowed"))?;
    let mut bottom = denominator
        .mantissa()
        .checked_mul(power_of_ten(numerator.scale())?)
        .ok_or_else(|| anyhow!("currency rate denominator overflowed"))?;
    if bottom < 0 {
        top = -top;
        bottom = -bottom;
    }
    let divisor = gcd(top.unsigned_abs(), bottom.unsigned_abs());
    let divisor = i128::try_from(divisor).map_err(|_| anyhow!("currency divisor overflowed"))?;
    top /= divisor;
    bottom /= divisor;

    let mut reduced = bottom;
    let mut twos = 0_u32;
    let mut fives = 0_u32;
    while reduced % 2 == 0 {
        reduced /= 2;
        twos += 1;
    }
    while reduced % 5 == 0 {
        reduced /= 5;
        fives += 1;
    }

    if reduced == 1 {
        let scale = twos.max(fives);
        let scaled = top
            .checked_mul(
                2_i128
                    .checked_pow(scale - twos)
                    .ok_or_else(|| anyhow!("rate overflow"))?,
            )
            .and_then(|value| value.checked_mul(5_i128.checked_pow(scale - fives)?))
            .ok_or_else(|| anyhow!("currency rate overflowed"))?;
        return Ok(Decimal::from_i128_with_scale(scaled, scale).normalize());
    }

    let scaled = top
        .checked_mul(10_000)
        .ok_or_else(|| anyhow!("currency rate overflowed"))?
        / bottom;
    Ok(Decimal::from_i128_with_scale(scaled, 4).normalize())
}

fn power_of_ten(scale: u32) -> Result<i128> {
    10_i128
        .checked_pow(scale)
        .ok_or_else(|| anyhow!("decimal scale is too large"))
}

const fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        let remainder = left % right;
        left = right;
        right = remainder;
    }
    left
}

#[cfg(test)]
#[path = "../tests/currency.rs"]
mod tests;
