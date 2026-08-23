use crate::app::{
    DefaultAction, conversion_item, conversion_item_with_customary_system, currency_items,
    invalid_item, placeholder_item, unit_items,
};
use crate::currency::{Currency, ExchangeRates};
use crate::units::{CustomarySystem, UnitListing};
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
        DefaultAction::OpenWebsite,
        DefaultAction::OpenWebsite,
        None,
        &mut engine,
    )?;
    let item = pending.into_item(None);
    let command = item.modifiers().and_then(|modifiers| modifiers.get("cmd"));
    assert_eq!(
        (
            item.title(),
            item.arg(),
            item.quick_look_url(),
            command.and_then(alfred_workflow_rs::Modifier::arg),
            command.and_then(alfred_workflow_rs::Modifier::subtitle),
        ),
        (
            "2in to cm = 5.08 cm",
            Some("https://www.wolframalpha.com/input?i=2in+to+cm"),
            Some("https://www.wolframalpha.com/input?i=2in+to+cm"),
            Some("5.08 cm"),
            Some("Copy 5.08 cm to clipboard"),
        )
    );
    Ok(())
}

#[test]
fn legacy_symbol_should_be_used_in_item_and_wolfram_action() -> anyhow::Result<()> {
    let home = Currency::from_code("USD").ok_or_else(|| anyhow::anyhow!("USD missing"))?;
    let mut engine = None;
    let item = conversion_item(
        "60 ' deg",
        home,
        DefaultAction::OpenWebsite,
        DefaultAction::OpenWebsite,
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
    let command_arg = item
        .modifiers()
        .and_then(|modifiers| modifiers.get("cmd"))
        .and_then(alfred_workflow_rs::Modifier::arg);
    let command_subtitle = item
        .modifiers()
        .and_then(|modifiers| modifiers.get("cmd"))
        .and_then(alfred_workflow_rs::Modifier::subtitle);
    assert_eq!(
        (
            item.title(),
            item.subtitle(),
            wolfram_query.as_deref(),
            command_arg,
            command_subtitle,
        ),
        (
            "60 ' = 1 °",
            Some("Based on the fact that 1 ' = 0.017 °"),
            Some("60.0 ' to °"),
            Some("1 °"),
            Some("Copy 1 ° to clipboard"),
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
        DefaultAction::OpenWebsite,
        DefaultAction::CopyToClipboard,
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
        DefaultAction::OpenWebsite,
        DefaultAction::CopyToClipboard,
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
        DefaultAction::OpenWebsite,
        DefaultAction::CopyToClipboard,
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
        DefaultAction::CopyToClipboard,
        DefaultAction::OpenWebsite,
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
        DefaultAction::OpenWebsite,
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
        DefaultAction::CopyToClipboard,
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
    let items = currency_items(currency("USD")?, DefaultAction::CopyToClipboard, None)?;
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
fn currency_catalogue_should_fall_back_when_a_small_rate_rounds_to_zero() -> anyhow::Result<()> {
    let rates = ExchangeRates {
        date: "2026-08-21T14:00:00Z".parse::<Timestamp>()?,
        rates: std::collections::BTreeMap::from([
            ("EUR".to_owned(), Decimal::ONE),
            ("USD".to_owned(), Decimal::new(11_699, 4)),
            ("IDR".to_owned(), Decimal::new(2_065_938, 2)),
        ]),
    };
    let item = currency_items(currency("USD")?, DefaultAction::OpenWebsite, Some(&rates))?
        .into_iter()
        .map(|pending| pending.into_item(None))
        .find(|item| item.title() == "Indonesian rupiah (IDR)")
        .ok_or_else(|| anyhow::anyhow!("IDR catalogue item missing"))?;
    assert_eq!(
        item.arg(),
        Some("https://www.oanda.com/currency-converter/en/currencies/majors/idr/")
    );
    Ok(())
}

#[test]
fn default_action_should_fall_back_to_open_website() {
    assert_eq!(
        (
            DefaultAction::default(),
            DefaultAction::from_preference(None),
            DefaultAction::from_preference(Some("unexpected")),
        ),
        (
            DefaultAction::OpenWebsite,
            DefaultAction::OpenWebsite,
            DefaultAction::OpenWebsite,
        )
    );
}

#[test]
fn default_action_should_parse_open_website_preference() {
    assert_eq!(
        DefaultAction::from_preference(Some("open_website")),
        DefaultAction::OpenWebsite
    );
}

#[test]
fn default_action_should_parse_copy_preference() {
    assert_eq!(
        DefaultAction::from_preference(Some("copy_to_clipboard")),
        DefaultAction::CopyToClipboard
    );
}

#[test]
fn native_unit_conversion_should_copy_only_the_value_by_default() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let mut engine = None;
    let item = conversion_item(
        "2in to cm",
        home,
        DefaultAction::OpenWebsite,
        DefaultAction::CopyToClipboard,
        None,
        &mut engine,
    )?
    .into_item(None);
    let command = item.modifiers().and_then(|modifiers| modifiers.get("cmd"));
    assert_eq!(
        (
            item.title(),
            item.arg(),
            item.quick_look_url(),
            command.and_then(alfred_workflow_rs::Modifier::arg),
            command.and_then(alfred_workflow_rs::Modifier::subtitle),
        ),
        (
            "2in to cm = 5.08 cm",
            Some("5.08 cm"),
            Some("https://www.wolframalpha.com/input?i=2in+to+cm"),
            Some("https://www.wolframalpha.com/input?i=2in+to+cm"),
            Some("Open evaluation details on WolframAlpha.com"),
        )
    );
    Ok(())
}

#[test]
fn legacy_unit_conversion_should_copy_only_the_value_by_default() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let mut engine = None;
    let item = conversion_item(
        "10 mi km",
        home,
        DefaultAction::OpenWebsite,
        DefaultAction::CopyToClipboard,
        None,
        &mut engine,
    )?
    .into_item(None);
    let command = item.modifiers().and_then(|modifiers| modifiers.get("cmd"));
    assert_eq!(
        (
            item.title(),
            item.subtitle(),
            item.arg(),
            item.quick_look_url(),
            command.and_then(alfred_workflow_rs::Modifier::arg),
            command.and_then(alfred_workflow_rs::Modifier::subtitle),
        ),
        (
            "10 mi = 16.093 km",
            Some("Based on the fact that 1 mi = 1.609 km"),
            Some("16.093 km"),
            Some("https://www.wolframalpha.com/input?i=10.0+mi+to+km"),
            Some("https://www.wolframalpha.com/input?i=10.0+mi+to+km"),
            Some("Open conversion details on WolframAlpha.com"),
        )
    );
    Ok(())
}

#[test]
fn customary_system_should_change_legacy_fluid_ounce_rendering() -> anyhow::Result<()> {
    let home = currency("USD")?;
    for (system, expected_title, expected_subtitle, expected_value, expected_url) in [
        (
            CustomarySystem::Imperial,
            "1 imp fl oz = 28.413 ml",
            "Based on the fact that 1 imp fl oz = 28.413 ml",
            "28.413 ml",
            "https://www.wolframalpha.com/input?i=1.0+imp+fl+oz+to+ml",
        ),
        (
            CustomarySystem::UsCustomary,
            "1 US fl oz = 29.574 ml",
            "Based on the fact that 1 US fl oz = 29.574 ml",
            "29.574 ml",
            "https://www.wolframalpha.com/input?i=1.0+US+fl+oz+to+ml",
        ),
    ] {
        let mut engine = None;
        let item = conversion_item_with_customary_system(
            "1 floz ml",
            home,
            DefaultAction::OpenWebsite,
            DefaultAction::OpenWebsite,
            system,
            None,
            &mut engine,
        )?
        .into_item(None);
        assert_eq!(
            (
                item.title(),
                item.subtitle(),
                item.arg(),
                item.quick_look_url()
            ),
            (
                expected_title,
                Some(expected_subtitle),
                Some(expected_url),
                Some(expected_url),
            )
        );

        let mut engine = None;
        let copy_item = conversion_item_with_customary_system(
            "1 floz ml",
            home,
            DefaultAction::OpenWebsite,
            DefaultAction::CopyToClipboard,
            system,
            None,
            &mut engine,
        )?
        .into_item(None);
        let command = copy_item
            .modifiers()
            .and_then(|modifiers| modifiers.get("cmd"));
        assert_eq!(
            (
                copy_item.arg(),
                command.and_then(alfred_workflow_rs::Modifier::arg),
                command.and_then(alfred_workflow_rs::Modifier::subtitle),
            ),
            (
                Some(expected_value),
                Some(expected_url),
                Some("Open conversion details on WolframAlpha.com"),
            )
        );
    }
    Ok(())
}

#[test]
fn explicit_customary_aliases_should_keep_labels_and_actions() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let expected_url = "https://www.wolframalpha.com/input?i=1.0+UK+fl+oz+to+US+fl+oz";
    for customary_system in [CustomarySystem::Imperial, CustomarySystem::UsCustomary] {
        let mut engine = None;
        let website_item = conversion_item_with_customary_system(
            "1 uk_floz to us_floz",
            home,
            DefaultAction::OpenWebsite,
            DefaultAction::OpenWebsite,
            customary_system,
            None,
            &mut engine,
        )?
        .into_item(None);
        let website_command = website_item
            .modifiers()
            .and_then(|modifiers| modifiers.get("cmd"));
        assert_eq!(
            (
                website_item.title(),
                website_item.subtitle(),
                website_item.arg(),
                website_item.quick_look_url(),
                website_command.and_then(alfred_workflow_rs::Modifier::arg),
            ),
            (
                "1 UK fl oz = 0.961 US fl oz",
                Some("Based on the fact that 1 UK fl oz = 0.961 US fl oz"),
                Some(expected_url),
                Some(expected_url),
                Some("0.961 US fl oz"),
            )
        );

        let mut engine = None;
        let copy_item = conversion_item_with_customary_system(
            "1 uk_floz to us_floz",
            home,
            DefaultAction::OpenWebsite,
            DefaultAction::CopyToClipboard,
            customary_system,
            None,
            &mut engine,
        )?
        .into_item(None);
        let copy_command = copy_item
            .modifiers()
            .and_then(|modifiers| modifiers.get("cmd"));
        assert_eq!(
            (
                copy_item.arg(),
                copy_item.quick_look_url(),
                copy_command.and_then(alfred_workflow_rs::Modifier::arg),
            ),
            (
                Some("0.961 US fl oz"),
                Some(expected_url),
                Some(expected_url),
            )
        );
    }
    Ok(())
}

#[test]
fn invalid_unit_expression_should_not_receive_result_actions() -> anyhow::Result<()> {
    let home = currency("USD")?;
    let mut engine = None;
    let item = conversion_item(
        "not a valid expression",
        home,
        DefaultAction::CopyToClipboard,
        DefaultAction::CopyToClipboard,
        None,
        &mut engine,
    )?
    .into_item(None);
    assert_eq!(
        (item.title(), item.arg(), item.modifiers()),
        ("Invalid format.", None, None)
    );
    Ok(())
}

#[test]
fn unit_catalogue_should_keep_copying_the_identifier() -> anyhow::Result<()> {
    let item = unit_items(&[UnitListing {
        identifier: "metre".to_owned(),
        name: "meter".to_owned(),
        dimension: "Length".to_owned(),
        aliases: vec!["m".to_owned()],
    }])
    .into_iter()
    .next()
    .ok_or_else(|| anyhow::anyhow!("unit catalogue item missing"))?
    .into_item(None);
    assert_eq!((item.arg(), item.modifiers()), (Some("metre"), None));
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
