use std::collections::BTreeMap;

use jiff::Timestamp;
use rust_decimal::Decimal;

use super::{Currency, ExchangeRates, divide_like_dart};

#[test]
fn divide_should_keep_finite_precision_and_truncate_repeating_values() -> anyhow::Result<()> {
    assert_eq!(
        divide_like_dart(Decimal::from(1), Decimal::from(8))?,
        Decimal::new(125, 3)
    );
    assert_eq!(
        divide_like_dart(Decimal::from(2), Decimal::from(3))?,
        Decimal::new(6666, 4)
    );
    Ok(())
}

#[test]
fn pair_rate_should_use_ecb_base_euro_values() -> anyhow::Result<()> {
    let rates = ExchangeRates {
        date: "2026-08-21T13:56:22Z".parse::<Timestamp>()?,
        rates: BTreeMap::from([
            ("EUR".to_owned(), Decimal::ONE),
            ("GBP".to_owned(), Decimal::new(8654, 4)),
            ("USD".to_owned(), Decimal::new(11732, 4)),
        ]),
    };
    assert_eq!(
        rates.pair_rate(
            Currency::from_code("GBP").ok_or_else(|| anyhow::anyhow!("GBP missing"))?,
            Currency::from_code("USD").ok_or_else(|| anyhow::anyhow!("USD missing"))?
        )?,
        Decimal::new(13556, 4)
    );
    Ok(())
}
