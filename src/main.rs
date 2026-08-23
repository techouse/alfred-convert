#![forbid(unsafe_code)]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;

use alfred_convert::app::{
    DefaultAction, PendingItem, conversion_item_with_customary_system, currency_items,
    invalid_item, placeholder_item, unit_items,
};
use alfred_convert::cli::Cli;
use alfred_convert::currency::{Currency, EcbClient, ExchangeRateCache};
use alfred_convert::format::parse_decimal;
use alfred_convert::services::emoji_image_cache::EmojiImageCache;
use alfred_convert::units::{CustomarySystem, UnitEngine};
use alfred_workflow_rs::{Icon, Item, RenderOptions, Updater, UserConfiguration, Workflow};
use anyhow::{Result, anyhow};
use jiff::Timestamp;

const GITHUB_REPOSITORY_URL: &str = "https://github.com/techouse/alfred-convert";
const UPDATE_INTERVAL: Duration = Duration::from_secs(7 * 24 * 60 * 60);

fn main() -> ExitCode {
    let cli = match Cli::parse(std::env::args().skip(1)) {
        Ok(cli) => cli,
        Err(error) => return render_parse_error(&error),
    };
    if cli.update {
        return update_workflow();
    }

    let mut workflow = Workflow::new();
    let (options, exit_code) = match populate_workflow(&mut workflow, &cli) {
        Ok(()) => (update_render_options(&cli), ExitCode::SUCCESS),
        Err(error) => {
            if cli.verbose {
                eprintln!("{error:#}");
            }
            if let Err(render_error) = replace_items_with_runtime_error(&mut workflow, &error) {
                eprintln!("failed to render workflow error: {render_error}");
                return ExitCode::from(1);
            }
            (RenderOptions::new(), ExitCode::from(1))
        }
    };
    if let Err(error) = workflow.write_stdout_with(options) {
        eprintln!("failed to write Script Filter JSON: {error}");
        return ExitCode::from(1);
    }
    exit_code
}

fn render_parse_error(error: &anyhow::Error) -> ExitCode {
    let mut workflow = Workflow::new();
    if workflow.add_item(Item::new(error.to_string())).is_err() || workflow.write_stdout().is_err()
    {
        eprintln!("{error}");
        return ExitCode::from(1);
    }
    ExitCode::from(2)
}

fn populate_workflow(workflow: &mut Workflow, cli: &Cli) -> Result<()> {
    let query = cli.normalized_query();
    if cli.verbose {
        eprintln!("Query: \"{query}\"");
    }
    let directory = workflow_directory()?;
    let settings = workflow_settings(workflow, &directory)?;

    if let Some(catalogue) = catalogue_mode(cli, &query) {
        return match catalogue {
            Catalogue::Currencies => {
                let rates = latest_rates(&directory, cli.verbose)?;
                add_pending_items(
                    workflow,
                    currency_items(
                        settings.home_currency,
                        settings.default_monetary_action,
                        rates.as_ref(),
                    )?,
                    &directory,
                    cli.verbose,
                )
            }
            Catalogue::Units => {
                let engine = UnitEngine::new()?;
                add_pending_items(
                    workflow,
                    unit_items(&engine.listings()),
                    &directory,
                    cli.verbose,
                )
            }
        };
    }
    if query.is_empty() {
        workflow.add_item(placeholder_item())?;
        return Ok(());
    }

    let currency_query = query_requires_exchange_rates(&query);
    let rates = if currency_query {
        latest_rates(&directory, cli.verbose)?
    } else {
        None
    };
    let mut unit_engine = None;
    let item = conversion_item_with_customary_system(
        &query,
        settings.home_currency,
        settings.default_monetary_action,
        settings.default_non_monetary_action,
        settings.customary_system,
        rates.as_ref(),
        &mut unit_engine,
    )?;
    add_pending_items(workflow, vec![item], &directory, cli.verbose)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Catalogue {
    Currencies,
    Units,
}

fn catalogue_mode(cli: &Cli, query: &str) -> Option<Catalogue> {
    if cli.currencies {
        Some(Catalogue::Currencies)
    } else if cli.units {
        Some(Catalogue::Units)
    } else {
        match query {
            "money" => Some(Catalogue::Currencies),
            "units" => Some(Catalogue::Units),
            _ => None,
        }
    }
}

fn query_requires_exchange_rates(query: &str) -> bool {
    let mut parts = query.split(' ');
    parts.next().and_then(parse_decimal).is_some()
        && parts.next().and_then(Currency::from_code).is_some()
}

fn latest_rates(
    directory: &Path,
    verbose: bool,
) -> Result<Option<alfred_convert::currency::ExchangeRates>> {
    let client = EcbClient::new()?;
    let cache = ExchangeRateCache::new(directory);
    let mut diagnostic = |message| {
        if verbose {
            eprintln!("{message}");
        }
    };
    Ok(client.latest(&cache, Timestamp::now(), &mut diagnostic))
}

fn add_pending_items(
    workflow: &mut Workflow,
    pending: Vec<PendingItem>,
    directory: &Path,
    verbose: bool,
) -> Result<()> {
    if pending.iter().all(|item| item.emoji().is_none()) {
        workflow.add_items(pending.into_iter().map(|item| item.into_item(None)))?;
        return Ok(());
    }
    let emojis = pending
        .iter()
        .map(|item| item.emoji().unwrap_or_default().to_owned())
        .collect::<Vec<_>>();
    let cache = EmojiImageCache::new(directory.join("image_cache"), verbose)?;
    let images = cache.resolve_many(&emojis);
    workflow.add_items(
        pending
            .into_iter()
            .zip(images)
            .map(|(item, image)| item.into_item(image.as_deref())),
    )?;
    Ok(())
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct WorkflowSettings {
    home_currency: Currency,
    default_monetary_action: DefaultAction,
    default_non_monetary_action: DefaultAction,
    customary_system: CustomarySystem,
}

fn workflow_settings(workflow: &Workflow, directory: &Path) -> Result<WorkflowSettings> {
    let defaults =
        workflow.get_user_defaults(directory.join("info.plist"), directory.join("prefs.plist"))?;
    workflow_settings_from_defaults(&defaults)
}

fn workflow_settings_from_defaults(
    defaults: &BTreeMap<String, UserConfiguration>,
) -> Result<WorkflowSettings> {
    let configured_currency = match defaults.get("default_currency") {
        Some(UserConfiguration::Select(configuration)) => Some(configuration.config.value.as_str()),
        _ => None,
    };
    let home_currency = configured_currency
        .and_then(Currency::from_code)
        .or_else(|| Currency::from_code("USD"))
        .ok_or_else(|| anyhow!("USD currency metadata is missing"))?;
    let configured_monetary_action = match defaults.get("default_monetary_action") {
        Some(UserConfiguration::Select(configuration)) => Some(configuration.config.value.as_str()),
        _ => None,
    };
    let configured_non_monetary_action = match defaults.get("default_non_monetary_action") {
        Some(UserConfiguration::Select(configuration)) => Some(configuration.config.value.as_str()),
        _ => None,
    };
    let configured_customary_system = match defaults.get("default_customary_system") {
        Some(UserConfiguration::Select(configuration)) => Some(configuration.config.value.as_str()),
        _ => None,
    };
    Ok(WorkflowSettings {
        home_currency,
        default_monetary_action: DefaultAction::from_preference(configured_monetary_action),
        default_non_monetary_action: DefaultAction::from_preference(configured_non_monetary_action),
        customary_system: CustomarySystem::from_preference(configured_customary_system),
    })
}

fn workflow_directory() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    executable
        .parent()
        .map(Path::to_path_buf)
        .ok_or_else(|| anyhow!("workflow executable has no parent directory"))
}

fn replace_items_with_runtime_error(
    workflow: &mut Workflow,
    error: &anyhow::Error,
) -> alfred_workflow_rs::Result<()> {
    if let Err(clear_error) = workflow.clear_items() {
        eprintln!("failed to clear workflow items: {clear_error}");
    }
    workflow.clear_cache_key();
    workflow.set_use_automatic_cache(false);
    workflow.add_item(invalid_item(Some(&error.to_string())))
}

fn update_render_options(cli: &Cli) -> RenderOptions {
    let updater = match updater() {
        Ok(updater) => updater,
        Err(error) => {
            if cli.verbose {
                eprintln!("could not create updater: {error}");
            }
            return RenderOptions::new();
        }
    };
    match updater.update_available() {
        Ok(true) => RenderOptions::new().add_to_beginning(update_item()),
        Ok(false) => RenderOptions::new(),
        Err(error) => {
            if cli.verbose {
                eprintln!("could not check for updates: {error}");
            }
            RenderOptions::new()
        }
    }
}

fn update_workflow() -> ExitCode {
    eprintln!("Updating workflow...");
    match updater().and_then(|updater| updater.update().map_err(Into::into)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}

fn updater() -> Result<Updater> {
    Ok(
        Updater::builder(GITHUB_REPOSITORY_URL.parse()?, env!("CARGO_PKG_VERSION"))?
            .update_interval(UPDATE_INTERVAL)
            .build()?,
    )
}

fn update_item() -> Item {
    Item::with_arg("Auto-Update available!", "update:workflow")
        .set_subtitle("Press <enter> to auto-update to a new version of this workflow.")
        .set_match_text(
            "Auto-Update available! Press <enter> to auto-update to a new version of this workflow.",
        )
        .set_icon(Icon::new("alfredhatcog.png"))
        .set_valid(true)
}

#[cfg(test)]
#[path = "tests/main.rs"]
mod tests;
