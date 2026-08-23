use std::collections::BTreeMap;

use alfred_convert::app::CurrencyDefaultAction;
use alfred_workflow_rs::{
    AutomaticCache, CheckBoxConfiguration, CheckBoxUserConfiguration, SelectConfiguration,
    SelectUserConfiguration, UserConfiguration, Workflow,
};

use super::{
    query_requires_exchange_rates, replace_items_with_runtime_error, update_item,
    workflow_settings_from_defaults,
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
fn update_item_should_keep_the_update_action() {
    let item = update_item();
    assert_eq!(
        (item.title(), item.arg()),
        ("Auto-Update available!", Some("update:workflow"))
    );
}

#[test]
fn workflow_settings_should_load_both_select_preferences() -> anyhow::Result<()> {
    let defaults = BTreeMap::from([
        select_configuration("default_currency", "EUR"),
        select_configuration("default_action", "copy_to_clipboard"),
    ]);
    let settings = workflow_settings_from_defaults(&defaults)?;
    assert_eq!(
        (
            settings.home_currency.code(),
            settings.currency_default_action
        ),
        ("EUR", CurrencyDefaultAction::CopyToClipboard)
    );
    Ok(())
}

#[test]
fn workflow_settings_should_ignore_wrong_action_configuration_type() -> anyhow::Result<()> {
    let defaults = BTreeMap::from([
        select_configuration("default_currency", "USD"),
        (
            "default_action".to_owned(),
            UserConfiguration::CheckBox(CheckBoxUserConfiguration {
                variable: "default_action".to_owned(),
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
    ]);
    let settings = workflow_settings_from_defaults(&defaults)?;
    assert_eq!(
        settings.currency_default_action,
        CurrencyDefaultAction::OpenWebsite
    );
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
