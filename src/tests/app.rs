use crate::app::{
    CurrencyDefaultAction, conversion_item, currency_items, invalid_item, placeholder_item,
};
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
    let pending = conversion_item(
        "2in to cm",
        home,
        CurrencyDefaultAction::OpenWebsite,
        None,
        &mut engine,
    )?;
    let item = pending.into_item(None);
    assert_eq!(item.title(), "2in to cm = 5.08 cm");
    Ok(())
}

#[test]
fn legacy_symbol_should_be_used_in_item_and_wolfram_action() -> anyhow::Result<()> {
    let home = Currency::from_code("USD").ok_or_else(|| anyhow::anyhow!("USD missing"))?;
    let mut engine = None;
    let item = conversion_item(
        "60 ' deg",
        home,
        CurrencyDefaultAction::OpenWebsite,
        None,
        &mut engine,
    )?
    .into_item(None);
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
    let pending = conversion_item(
        "10 EUR USD ignored",
        home,
        CurrencyDefaultAction::OpenWebsite,
        Some(&rates),
        &mut engine,
    )?;
    assert_eq!(pending.into_item(None).title(), "Invalid format.");
    Ok(())
}

#[test]
fn malformed_currency_amount_should_not_initialize_numbat() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let mut engine = None;
    let pending = conversion_item(
        "abc USD EUR",
        home,
        CurrencyDefaultAction::OpenWebsite,
        None,
        &mut engine,
    )?;

    assert_eq!(pending.into_item(None).title(), "Invalid format.");
    assert!(engine.is_none());
    Ok(())
}

#[test]
fn currency_item_should_open_xe_by_default_and_copy_with_command() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let rates = sample_rates()?;
    let mut engine = None;
    let item = conversion_item(
        "10 EUR USD",
        home,
        CurrencyDefaultAction::OpenWebsite,
        Some(&rates),
        &mut engine,
    )?
    .into_item(None);
    let command = item.modifiers().and_then(|modifiers| modifiers.get("cmd"));
    let option = item.modifiers().and_then(|modifiers| modifiers.get("alt"));
    assert_eq!(
        (
            item.title(),
            item.subtitle(),
            item.arg(),
            item.quick_look_url(),
            command.and_then(alfred_workflow_rs::Modifier::arg),
            command.and_then(alfred_workflow_rs::Modifier::subtitle),
            option.and_then(alfred_workflow_rs::Modifier::arg),
            option.and_then(alfred_workflow_rs::Modifier::subtitle),
        ),
        (
            "10 EUR 🇪🇺 ≃ 11.732 USD 🇺🇸",
            Some("Based on ECB exchange rates from Aug 21, 2026"),
            Some("https://www.xe.com/currencycharts?from=EUR&to=USD"),
            Some("https://www.xe.com/currencycharts?from=EUR&to=USD"),
            Some("11.732 USD"),
            Some("Copy 11.732 USD 🇺🇸 to clipboard"),
            None,
            Some("10 USD 🇺🇸 ≃ 8.523 EUR 🇪🇺"),
        )
    );
    Ok(())
}

#[test]
fn currency_item_should_copy_by_default_and_open_xe_with_command() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let rates = sample_rates()?;
    let mut engine = None;
    let item = conversion_item(
        "10 EUR USD",
        home,
        CurrencyDefaultAction::CopyToClipboard,
        Some(&rates),
        &mut engine,
    )?
    .into_item(None);
    let modifiers = item
        .modifiers()
        .ok_or_else(|| anyhow::anyhow!("currency modifiers missing"))?;
    assert_eq!(
        (
            item.arg(),
            item.subtitle(),
            item.quick_look_url(),
            modifiers
                .get("cmd")
                .and_then(alfred_workflow_rs::Modifier::arg),
            modifiers
                .get("cmd")
                .and_then(alfred_workflow_rs::Modifier::subtitle),
            modifiers
                .get("alt")
                .and_then(alfred_workflow_rs::Modifier::arg),
            modifiers
                .get("alt")
                .and_then(alfred_workflow_rs::Modifier::subtitle),
        ),
        (
            Some("11.732 USD"),
            Some("Based on ECB exchange rates from Aug 21, 2026"),
            Some("https://www.xe.com/currencycharts?from=EUR&to=USD"),
            Some("https://www.xe.com/currencycharts?from=EUR&to=USD"),
            Some("Open currency-pair chart on Xe.com"),
            Some("8.523 EUR"),
            Some("10 USD 🇺🇸 ≃ 8.523 EUR 🇪🇺"),
        )
    );
    Ok(())
}

#[test]
fn currency_catalogue_should_open_xe_by_default_and_copy_with_command() -> anyhow::Result<()> {
    let items = currency_items(
        currency("USD")?,
        CurrencyDefaultAction::OpenWebsite,
        Some(&sample_rates()?),
    )?;
    let item = items
        .into_iter()
        .map(|pending| pending.into_item(None))
        .find(|item| item.title() == "Euro (EUR)")
        .ok_or_else(|| anyhow::anyhow!("EUR catalogue item missing"))?;
    let modifiers = item
        .modifiers()
        .ok_or_else(|| anyhow::anyhow!("currency modifiers missing"))?;
    assert_eq!(
        (
            item.arg(),
            modifiers
                .get("cmd")
                .and_then(alfred_workflow_rs::Modifier::arg),
            modifiers
                .get("alt")
                .and_then(alfred_workflow_rs::Modifier::arg),
        ),
        (
            Some("https://www.xe.com/currencycharts?from=EUR&to=USD"),
            Some("1.173 USD"),
            None,
        )
    );
    Ok(())
}

#[test]
fn currency_catalogue_should_apply_copy_default_to_rate_backed_rows() -> anyhow::Result<()> {
    let items = currency_items(
        currency("USD")?,
        CurrencyDefaultAction::CopyToClipboard,
        Some(&sample_rates()?),
    )?;
    let item = items
        .into_iter()
        .map(|pending| pending.into_item(None))
        .find(|item| item.title() == "Euro (EUR)")
        .ok_or_else(|| anyhow::anyhow!("EUR catalogue item missing"))?;
    let modifiers = item
        .modifiers()
        .ok_or_else(|| anyhow::anyhow!("currency modifiers missing"))?;
    assert_eq!(
        (
            item.arg(),
            item.subtitle(),
            item.quick_look_url(),
            modifiers
                .get("cmd")
                .and_then(alfred_workflow_rs::Modifier::arg),
            modifiers
                .get("cmd")
                .and_then(alfred_workflow_rs::Modifier::subtitle),
            modifiers
                .get("alt")
                .and_then(alfred_workflow_rs::Modifier::arg),
            modifiers
                .get("alt")
                .and_then(alfred_workflow_rs::Modifier::subtitle),
        ),
        (
            Some("1.173 USD"),
            Some("1 EUR ≃ 1.173 USD"),
            Some("https://www.xe.com/currencycharts?from=EUR&to=USD"),
            Some("https://www.xe.com/currencycharts?from=EUR&to=USD"),
            Some("Open currency-pair chart on Xe.com"),
            Some("0.852 EUR"),
            Some("1 USD ≃ 0.852 EUR"),
        )
    );
    Ok(())
}

#[test]
fn currency_catalogue_should_keep_oanda_fallback_in_copy_mode() -> anyhow::Result<()> {
    let items = currency_items(
        currency("USD")?,
        CurrencyDefaultAction::CopyToClipboard,
        None,
    )?;
    let item = items
        .into_iter()
        .map(|pending| pending.into_item(None))
        .find(|item| item.title() == "US dollar (USD)")
        .ok_or_else(|| anyhow::anyhow!("USD catalogue item missing"))?;
    let command_arg = item
        .modifiers()
        .and_then(|modifiers| modifiers.get("cmd"))
        .and_then(alfred_workflow_rs::Modifier::arg);
    assert_eq!(
        (item.arg(), command_arg),
        (
            Some("https://www.oanda.com/currency-converter/en/currencies/majors/usd/"),
            Some("US dollar USD"),
        )
    );
    Ok(())
}

#[test]
fn currency_default_action_should_fall_back_to_open_website() {
    assert_eq!(
        (
            CurrencyDefaultAction::default(),
            CurrencyDefaultAction::from_preference(None),
            CurrencyDefaultAction::from_preference(Some("unexpected")),
        ),
        (
            CurrencyDefaultAction::OpenWebsite,
            CurrencyDefaultAction::OpenWebsite,
            CurrencyDefaultAction::OpenWebsite,
        )
    );
}

#[test]
fn currency_default_action_should_parse_open_website_preference() {
    assert_eq!(
        CurrencyDefaultAction::from_preference(Some("open_website")),
        CurrencyDefaultAction::OpenWebsite
    );
}

#[test]
fn currency_default_action_should_parse_copy_preference() {
    assert_eq!(
        CurrencyDefaultAction::from_preference(Some("copy_to_clipboard")),
        CurrencyDefaultAction::CopyToClipboard
    );
}

#[test]
fn unit_conversion_should_keep_its_wolfram_action() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let mut engine = None;
    let item = conversion_item(
        "2in to cm",
        home,
        CurrencyDefaultAction::CopyToClipboard,
        None,
        &mut engine,
    )?
    .into_item(None);
    assert_eq!(
        (item.arg(), item.modifiers()),
        (Some("https://www.wolframalpha.com/input?i=2in+to+cm"), None,)
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
