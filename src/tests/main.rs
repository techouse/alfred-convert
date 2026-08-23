use alfred_workflow_rs::{AutomaticCache, Workflow};

use super::{query_requires_exchange_rates, replace_items_with_runtime_error};

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
