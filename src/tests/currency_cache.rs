use std::collections::BTreeMap;

use jiff::Timestamp;
use rmpv::Value;
use rust_decimal::Decimal;
use tempfile::TempDir;

use super::{ExchangeRateCache, ExchangeRates};

#[test]
fn json_cache_should_round_trip_rates() -> anyhow::Result<()> {
    let directory = TempDir::new()?;
    let cache = ExchangeRateCache::new(directory.path());
    let expected = ExchangeRates {
        date: "2026-08-21T13:56:22Z".parse::<Timestamp>()?,
        rates: BTreeMap::from([
            ("EUR".to_owned(), Decimal::ONE),
            ("USD".to_owned(), Decimal::new(11732, 4)),
        ]),
    };
    cache.store(&expected)?;
    assert_eq!(cache.load(&mut |_| {}), Some(expected));
    Ok(())
}

#[test]
fn corrupt_cache_should_be_a_non_fatal_miss() -> anyhow::Result<()> {
    let directory = TempDir::new()?;
    let cache_directory = directory.path().join("exchange_rates_cache");
    std::fs::create_dir_all(&cache_directory)?;
    std::fs::write(cache_directory.join("latest.json"), b"not json")?;
    let cache = ExchangeRateCache::new(directory.path());
    let mut diagnostics = Vec::new();
    assert_eq!(cache_load(&cache, &mut diagnostics), None);
    assert_eq!(diagnostics.len(), 1);
    Ok(())
}

#[test]
fn legacy_stash_cache_should_be_imported_and_rewritten_as_json() -> anyhow::Result<()> {
    let directory = TempDir::new()?;
    let cache_directory = directory.path().join("exchange_rates_cache");
    std::fs::create_dir_all(&cache_directory)?;
    let legacy = Value::Map(vec![
        (Value::from("date"), Value::from("2026-08-21T13:56:22Z")),
        (
            Value::from("rates"),
            Value::Array(vec![
                Value::Map(vec![
                    (Value::from("currency"), Value::from("EUR")),
                    (Value::from("rate"), Value::from("1")),
                ]),
                Value::Map(vec![
                    (Value::from("currency"), Value::from("USD")),
                    (Value::from("rate"), Value::from("1.1732")),
                ]),
            ]),
        ),
    ]);
    let mut inner = Vec::new();
    rmpv::encode::write_value(&mut inner, &legacy)?;
    let mut outer = Vec::new();
    rmpv::encode::write_value(&mut outer, &Value::Ext(0, inner))?;
    let mut stash = vec![0_u8; 40];
    stash.extend(outer);
    std::fs::write(cache_directory.join("latest"), stash)?;

    let cache = ExchangeRateCache::new(directory.path());
    let rates = cache
        .load(&mut |_| {})
        .ok_or_else(|| anyhow::anyhow!("legacy cache was not imported"))?;
    assert_eq!(rates.rates.get("USD"), Some(&Decimal::new(11_732, 4)));
    assert!(cache_directory.join("latest.json").is_file());
    Ok(())
}

fn cache_load(cache: &ExchangeRateCache, diagnostics: &mut Vec<String>) -> Option<ExchangeRates> {
    cache.load(&mut |message| diagnostics.push(message))
}
