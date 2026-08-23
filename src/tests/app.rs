use crate::app::{conversion_item, invalid_item, placeholder_item};
use crate::currency::{Currency, ExchangeRates};
use jiff::Timestamp;
use rust_decimal::Decimal;

#[test]
fn placeholder_should_match_the_dart_title() {
    assert_eq!(placeholder_item().title(), "Convert from ... to ...");
}

#[test]
fn invalid_item_should_keep_the_historical_usage() {
    assert_eq!(
        invalid_item(None).subtitle(),
        Some("Usage: conv 123.45 gbp usd")
    );
}

#[test]
fn native_numbat_query_should_not_require_a_separate_numeric_token() -> anyhow::Result<()> {
    let home = Currency::from_code("USD").ok_or_else(|| anyhow::anyhow!("USD missing"))?;
    let mut engine = None;
    let pending = conversion_item("2in to cm", home, None, &mut engine)?;
    let item = pending.into_item(None);
    assert_eq!(item.title(), "2in to cm = 5.08 cm");
    Ok(())
}

#[test]
fn legacy_symbol_should_be_used_in_item_and_wolfram_action() -> anyhow::Result<()> {
    let home = Currency::from_code("USD").ok_or_else(|| anyhow::anyhow!("USD missing"))?;
    let mut engine = None;
    let item = conversion_item("60 ' deg", home, None, &mut engine)?.into_item(None);
    let wolfram_query = item
        .arg()
        .and_then(|argument| url::Url::parse(argument).ok())
        .and_then(|url| {
            url.query_pairs()
                .find(|(name, _)| name == "i")
                .map(|(_, value)| value.into_owned())
        });
    assert_eq!(
        (item.title(), item.subtitle(), wolfram_query.as_deref()),
        (
            "60 ' = 1 °",
            Some("Based on the fact that 1 ' = 0.017 °"),
            Some("60.0 ' to °")
        )
    );
    Ok(())
}

#[test]
fn trailing_currency_tokens_should_be_rejected_instead_of_ignored() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let rates = sample_rates()?;
    let mut engine = None;
    let pending = conversion_item("10 EUR USD ignored", home, Some(&rates), &mut engine)?;
    assert_eq!(pending.into_item(None).title(), "Invalid format.");
    Ok(())
}

#[test]
fn malformed_currency_amount_should_not_initialize_numbat() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let mut engine = None;
    let pending = conversion_item("abc USD EUR", home, None, &mut engine)?;

    assert_eq!(pending.into_item(None).title(), "Invalid format.");
    assert!(engine.is_none());
    Ok(())
}

#[test]
fn currency_item_should_preserve_conversion_title_and_xe_action() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let rates = sample_rates()?;
    let mut engine = None;
    let item = conversion_item("10 EUR USD", home, Some(&rates), &mut engine)?.into_item(None);
    assert_eq!(item.title(), "10 EUR 🇪🇺 ≃ 11.732 USD 🇺🇸");
    assert!(
        item.arg()
            .is_some_and(|argument| argument.starts_with("https://www.xe.com/"))
    );
    Ok(())
}

fn currency(code: &str) -> anyhow::Result<Currency> {
    Currency::from_code(code).ok_or_else(|| anyhow::anyhow!("currency {code} missing"))
}

fn sample_rates() -> anyhow::Result<ExchangeRates> {
    Ok(ExchangeRates {
        date: "2026-08-21T14:00:00Z".parse::<Timestamp>()?,
        rates: std::collections::BTreeMap::from([
            ("EUR".to_owned(), Decimal::ONE),
            ("USD".to_owned(), Decimal::new(11_732, 4)),
        ]),
    })
}
