use std::collections::BTreeMap;

use alfred_convert::app::DefaultAction;
use alfred_convert::units::CustomarySystem;
use alfred_workflow_rs::{
    AutomaticCache, CheckBoxConfiguration, CheckBoxUserConfiguration, SelectConfiguration,
    SelectUserConfiguration, UserConfiguration, Workflow,
};

use super::{
    Catalogue, catalogue_mode, query_requires_exchange_rates, replace_items_with_runtime_error,
    update_item, workflow_settings_from_defaults,
};

#[test]
fn runtime_error_should_disable_automatic_cache() -> anyhow::Result<()> {
    let cache = AutomaticCache::try_with_loose_reload(60, Some(true))?;
    let mut workflow = Workflow::with_automatic_cache(cache);
    replace_items_with_runtime_error(&mut workflow, &anyhow::anyhow!("temporary failure"))?;
    assert!(!workflow.to_json_string()?.contains("\"cache\""));
    Ok(())
}

#[test]
fn valid_currency_query_should_require_exchange_rates() {
    assert!(query_requires_exchange_rates("10 USD EUR"));
}

#[test]
fn malformed_currency_amount_should_not_require_exchange_rates() {
    assert!(!query_requires_exchange_rates("abc USD EUR"));
}

#[test]
fn normalized_money_query_should_select_the_currency_catalogue() {
    let cli = super::Cli {
        query: "  money\t".to_owned(),
        ..super::Cli::default()
    };
    assert_eq!(
        catalogue_mode(&cli, &cli.normalized_query()),
        Some(Catalogue::Currencies)
    );
}

#[test]
fn units_query_should_select_the_unit_catalogue() {
    let cli = super::Cli {
        query: "units".to_owned(),
        ..super::Cli::default()
    };
    assert_eq!(
        catalogue_mode(&cli, &cli.normalized_query()),
        Some(Catalogue::Units)
    );
}

#[test]
fn capitalized_money_query_should_remain_a_conversion_query() {
    let cli = super::Cli {
        query: "Money".to_owned(),
        ..super::Cli::default()
    };
    assert_eq!(catalogue_mode(&cli, &cli.normalized_query()), None);
}

#[test]
fn conversion_query_should_remain_outside_catalogues() {
    let cli = super::Cli {
        query: "10 USD EUR".to_owned(),
        ..super::Cli::default()
    };
    assert_eq!(catalogue_mode(&cli, &cli.normalized_query()), None);
}

#[test]
fn explicit_units_flag_should_override_money_query_keyword() {
    let cli = super::Cli {
        query: "money".to_owned(),
        units: true,
        ..super::Cli::default()
    };
    assert_eq!(
        catalogue_mode(&cli, &cli.normalized_query()),
        Some(Catalogue::Units)
    );
}

#[test]
fn currencies_flag_should_override_units_query_keyword() {
    let cli = super::Cli {
        query: "units".to_owned(),
        currencies: true,
        ..super::Cli::default()
    };
    assert_eq!(
        catalogue_mode(&cli, &cli.normalized_query()),
        Some(Catalogue::Currencies)
    );
}

#[test]
fn currencies_flag_should_retain_precedence_over_units_flag() {
    let cli = super::Cli {
        currencies: true,
        units: true,
        ..super::Cli::default()
    };
    assert_eq!(catalogue_mode(&cli, ""), Some(Catalogue::Currencies));
}

#[test]
fn update_item_should_keep_the_update_action() {
    let item = update_item();
    assert_eq!(
        (item.title(), item.arg()),
        ("Auto-Update available!", Some("update:workflow"))
    );
}

#[test]
fn workflow_settings_should_load_all_select_preferences() -> anyhow::Result<()> {
    let defaults = BTreeMap::from([
        select_configuration("default_currency", "EUR"),
        select_configuration("default_monetary_action", "copy_to_clipboard"),
        select_configuration("default_non_monetary_action", "open_website"),
        select_configuration("default_customary_system", "us_customary"),
    ]);
    let settings = workflow_settings_from_defaults(&defaults)?;
    assert_eq!(
        (
            settings.home_currency.code(),
            settings.default_monetary_action,
            settings.default_non_monetary_action,
            settings.customary_system,
        ),
        (
            "EUR",
            DefaultAction::CopyToClipboard,
            DefaultAction::OpenWebsite,
            CustomarySystem::UsCustomary,
        )
    );
    Ok(())
}

#[test]
fn workflow_settings_should_fall_back_for_wrong_action_configuration_type() -> anyhow::Result<()> {
    let defaults = BTreeMap::from([
        select_configuration("default_currency", "USD"),
        (
            "default_monetary_action".to_owned(),
            UserConfiguration::CheckBox(CheckBoxUserConfiguration {
                variable: "default_monetary_action".to_owned(),
                description: None,
                label: None,
                config: CheckBoxConfiguration {
                    default_value: true,
                    value: true,
                    required: false,
                    text: None,
                },
            }),
        ),
        select_configuration("default_non_monetary_action", "copy_to_clipboard"),
    ]);
    let settings = workflow_settings_from_defaults(&defaults)?;
    assert_eq!(
        (
            settings.default_monetary_action,
            settings.default_non_monetary_action,
            settings.customary_system,
        ),
        (
            DefaultAction::OpenWebsite,
            DefaultAction::CopyToClipboard,
            CustomarySystem::Imperial,
        )
    );
    Ok(())
}

#[test]
fn workflow_settings_should_ignore_obsolete_and_unknown_action_values() -> anyhow::Result<()> {
    let defaults = BTreeMap::from([
        select_configuration("default_currency", "USD"),
        select_configuration("default_action", "copy_to_clipboard"),
        select_configuration("default_non_monetary_action", "unexpected"),
    ]);
    let settings = workflow_settings_from_defaults(&defaults)?;
    assert_eq!(
        (
            settings.default_monetary_action,
            settings.default_non_monetary_action,
            settings.customary_system,
        ),
        (
            DefaultAction::OpenWebsite,
            DefaultAction::OpenWebsite,
            CustomarySystem::Imperial,
        )
    );
    Ok(())
}

#[test]
fn workflow_settings_should_fall_back_for_invalid_customary_system_preferences()
-> anyhow::Result<()> {
    for configuration in [
        UserConfiguration::CheckBox(CheckBoxUserConfiguration {
            variable: "default_customary_system".to_owned(),
            description: None,
            label: None,
            config: CheckBoxConfiguration {
                default_value: true,
                value: true,
                required: false,
                text: None,
            },
        }),
        UserConfiguration::Select(SelectUserConfiguration {
            variable: "default_customary_system".to_owned(),
            description: None,
            label: None,
            config: SelectConfiguration {
                default_value: "unexpected".to_owned(),
                value: "unexpected".to_owned(),
                pairs: Vec::new(),
            },
        }),
    ] {
        let defaults = BTreeMap::from([("default_customary_system".to_owned(), configuration)]);
        let settings = workflow_settings_from_defaults(&defaults)?;
        assert_eq!(settings.customary_system, CustomarySystem::Imperial);
    }
    Ok(())
}

fn select_configuration(variable: &str, value: &str) -> (String, UserConfiguration) {
    (
        variable.to_owned(),
        UserConfiguration::Select(SelectUserConfiguration {
            variable: variable.to_owned(),
            description: None,
            label: None,
            config: SelectConfiguration {
                default_value: value.to_owned(),
                value: value.to_owned(),
                pairs: Vec::new(),
            },
        }),
    )
}
