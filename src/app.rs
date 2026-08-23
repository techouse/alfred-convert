//! Alfred item construction and query routing.

use std::fmt::Write as _;
use std::path::Path;

use alfred_workflow_rs::{Icon, Item, ItemText, Modifier, ModifierKey};
use anyhow::{Result, anyhow};
use jiff::Timestamp;
use rust_decimal::Decimal;
use url::Url;

use crate::currency::{CURRENCIES, Currency, ExchangeRates, divide_like_dart};
use crate::format::{format_decimal, parse_decimal};
use crate::units::{UnitEngine, UnitListing, legacy_conversion};

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// An Alfred item plus the emoji image it would prefer to use.
pub struct PendingItem {
    item: Item,
    emoji: Option<String>,
}

/// Default Return action for conversion results.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DefaultAction {
    /// Open the result's website.
    #[default]
    OpenWebsite,
    /// Copy the converted value to the clipboard.
    CopyToClipboard,
}

impl DefaultAction {
    /// Parses the stored Alfred preference, falling back to the website action.
    #[must_use]
    pub fn from_preference(value: Option<&str>) -> Self {
        match value {
            Some("copy_to_clipboard") => Self::CopyToClipboard,
            _ => Self::OpenWebsite,
        }
    }
}

impl PendingItem {
    fn new(item: Item, emoji: Option<&str>) -> Self {
        Self {
            item,
            emoji: emoji.map(str::to_owned),
        }
    }

    /// Returns the requested emoji, if any.
    #[must_use]
    pub fn emoji(&self) -> Option<&str> {
        self.emoji.as_deref()
    }

    /// Finishes the item with either a cached image or the workflow icon.
    #[must_use]
    pub fn into_item(self, image: Option<&Path>) -> Item {
        self.item.set_icon(Icon::new(
            image.map_or_else(|| "icon.png".into(), |path| path.display().to_string()),
        ))
    }
}

/// Builds the empty-query placeholder.
#[must_use]
pub fn placeholder_item() -> Item {
    Item::new("Convert from ... to ...").set_icon(Icon::new("icon.png"))
}

/// Builds the historical invalid-format item.
#[must_use]
pub fn invalid_item(message: Option<&str>) -> Item {
    Item::new(message.unwrap_or("Invalid format."))
        .set_subtitle("Usage: conv 123.45 gbp usd")
        .set_icon(Icon::new("icon.png"))
}

/// Converts a normalized query through the ECB or Numbat engine.
///
/// # Errors
/// Returns an error only when Alfred item metadata cannot be constructed.
pub fn conversion_item(
    query: &str,
    home: Currency,
    monetary_default_action: DefaultAction,
    non_monetary_default_action: DefaultAction,
    rates: Option<&ExchangeRates>,
    unit_engine: &mut Option<UnitEngine>,
) -> Result<PendingItem> {
    let parts = query.split(' ').collect::<Vec<_>>();
    if let Some(from) = parts.get(1).and_then(|part| Currency::from_code(part)) {
        if parse_decimal(parts[0]).is_none() {
            return Ok(PendingItem::new(invalid_item(None), None));
        }
        return currency_conversion_item(&parts, from, home, monetary_default_action, rates);
    }
    if parts
        .iter()
        .skip(1)
        .any(|part| Currency::from_code(part).is_some())
    {
        return Ok(PendingItem::new(invalid_item(None), None));
    }

    let engine = match unit_engine {
        Some(engine) => engine,
        slot @ None => slot.insert(UnitEngine::new()?),
    };
    if let Some(conversion) = legacy_conversion(query) {
        return match engine.evaluate_legacy(conversion) {
            Ok(evaluation) => {
                let url = wolfram_url(&format!(
                    "{} {} to {}",
                    dart_double(conversion.amount),
                    conversion.from.symbol,
                    conversion.to.symbol
                ))?;
                let copy_subtitle = format!("Copy {} to clipboard", evaluation.copy_value);
                let actions = result_actions(
                    non_monetary_default_action,
                    url.as_str(),
                    evaluation.copy_value,
                    copy_subtitle,
                    "Open conversion details on WolframAlpha.com",
                );
                let item = Item::with_arg(evaluation.result, actions.default_arg)
                    .set_subtitle(evaluation.legacy_fact.unwrap_or_default())
                    .set_quick_look_url(url.as_str())
                    .set_valid(true)
                    .try_set_modifier([ModifierKey::Cmd], actions.command_modifier)?;
                Ok(PendingItem::new(item, evaluation.emoji))
            }
            Err(error) => Ok(PendingItem::new(
                invalid_item(Some(&legacy_error_message(&error.to_string()))),
                None,
            )),
        };
    }

    match engine.evaluate_native(query) {
        Ok(evaluation) => {
            let url = wolfram_url(query)?;
            let copy_subtitle = format!("Copy {} to clipboard", evaluation.copy_value);
            let actions = result_actions(
                non_monetary_default_action,
                url.as_str(),
                evaluation.copy_value,
                copy_subtitle,
                "Open evaluation details on WolframAlpha.com",
            );
            let item = Item::with_arg(
                format!("{query} = {}", evaluation.result),
                actions.default_arg,
            )
            .set_subtitle("Evaluated with Numbat")
            .set_quick_look_url(url.as_str())
            .set_valid(true)
            .try_set_modifier([ModifierKey::Cmd], actions.command_modifier)?;
            Ok(PendingItem::new(item, None))
        }
        Err(_) => Ok(PendingItem::new(invalid_item(None), None)),
    }
}

fn currency_conversion_item(
    parts: &[&str],
    from: Currency,
    home: Currency,
    default_action: DefaultAction,
    rates: Option<&ExchangeRates>,
) -> Result<PendingItem> {
    let to_code = match parts {
        [_, _] => home.code(),
        [_, _, separator] if separator.eq_ignore_ascii_case("to") => home.code(),
        [_, _, to] => *to,
        [_, _, separator, to] if separator.eq_ignore_ascii_case("to") => *to,
        _ => return Ok(PendingItem::new(invalid_item(None), None)),
    };
    let Some(rates) = rates else {
        return Ok(PendingItem::new(invalid_item(None), None));
    };
    let Some(to) = Currency::from_code(to_code) else {
        return Ok(PendingItem::new(
            invalid_item(Some(&format!(
                "Can not convert {} to \"{to_code}\"",
                from.name()
            ))),
            None,
        ));
    };
    let amount = parse_decimal(parts[0]).ok_or_else(|| anyhow!("invalid decimal amount"))?;
    let rate = rates.pair_rate(from, to)?;
    let inverted_rate = divide_like_dart(Decimal::ONE, rate)?;
    let converted = amount
        .checked_mul(rate)
        .ok_or_else(|| anyhow!("currency conversion overflowed"))?;
    let inverted = amount
        .checked_mul(inverted_rate)
        .ok_or_else(|| anyhow!("inverse currency conversion overflowed"))?;
    let url = xe_url(from, to)?;
    let actions = result_actions(
        default_action,
        url.as_str(),
        format!("{} {}", format_decimal(converted), to.code()),
        format!(
            "Copy {} {} {} to clipboard",
            format_decimal(converted),
            to.code(),
            to.flag()
        ),
        "Open currency-pair chart on Xe.com",
    );
    let mut inverse_modifier = Modifier::new().with_subtitle(format!(
        "{} {} {} ≃ {} {} {}",
        format_decimal(amount),
        to.code(),
        to.flag(),
        format_decimal(inverted),
        from.code(),
        from.flag()
    ));
    if default_action == DefaultAction::CopyToClipboard {
        inverse_modifier =
            inverse_modifier.with_arg(format!("{} {}", format_decimal(inverted), from.code()));
    }
    let item = Item::with_arg(
        format!(
            "{} {} {} ≃ {} {} {}",
            format_decimal(amount),
            from.code(),
            from.flag(),
            format_decimal(converted),
            to.code(),
            to.flag()
        ),
        actions.default_arg,
    )
    .set_subtitle(exchange_rate_subtitle(rates.date))
    .set_quick_look_url(url.as_str())
    .set_valid(true)
    .try_set_modifier([ModifierKey::Alt], inverse_modifier)?
    .try_set_modifier([ModifierKey::Cmd], actions.command_modifier)?;
    Ok(PendingItem::new(item, Some(to.flag())))
}

/// Builds the stable currency catalogue.
///
/// # Errors
/// Returns an error if a URL or modifier cannot be built.
pub fn currency_items(
    home: Currency,
    default_action: DefaultAction,
    rates: Option<&ExchangeRates>,
) -> Result<Vec<PendingItem>> {
    CURRENCIES
        .iter()
        .copied()
        .map(|currency| currency_catalog_item(currency, home, default_action, rates))
        .collect()
}

fn currency_catalog_item(
    currency: Currency,
    home: Currency,
    default_action: DefaultAction,
    rates: Option<&ExchangeRates>,
) -> Result<PendingItem> {
    if currency != home
        && let Some(rate) = rates.and_then(|rates| rates.pair_rate(currency, home).ok())
    {
        let inverse = divide_like_dart(Decimal::ONE, rate)?;
        let url = xe_url(currency, home)?;
        let actions = result_actions(
            default_action,
            url.as_str(),
            format!("{} {}", format_decimal(rate), home.code()),
            format!(
                "Copy {} {} {} to clipboard",
                format_decimal(rate),
                home.code(),
                home.flag()
            ),
            "Open currency-pair chart on Xe.com",
        );
        let mut inverse_modifier = Modifier::new().with_subtitle(format!(
            "1 {} ≃ {} {}",
            home.code(),
            format_decimal(inverse),
            currency.code()
        ));
        if default_action == DefaultAction::CopyToClipboard {
            inverse_modifier = inverse_modifier.with_arg(format!(
                "{} {}",
                format_decimal(inverse),
                currency.code()
            ));
        }
        let item = Item::with_arg(
            format!("{} ({})", currency.name(), currency.code()),
            actions.default_arg,
        )
        .set_subtitle(format!(
            "1 {} ≃ {} {}",
            currency.code(),
            format_decimal(rate),
            home.code()
        ))
        .set_quick_look_url(url.as_str())
        .set_match_text(format!("{} ({})", currency.name(), currency.code()))
        .set_text(ItemText::new(currency.code()).with_large_type(currency.code()))
        .set_valid(true)
        .try_set_modifier([ModifierKey::Alt], inverse_modifier)?
        .try_set_modifier([ModifierKey::Cmd], actions.command_modifier)?;
        return Ok(PendingItem::new(item, Some(currency.flag())));
    }

    let url = oanda_url(currency)?;
    let item = Item::with_arg(
        format!("{} ({})", currency.name(), currency.code()),
        url.as_str(),
    )
    .set_subtitle("Open currency fact sheet")
    .set_quick_look_url(url.as_str())
    .set_match_text(format!("{} ({})", currency.name(), currency.code()))
    .set_text(ItemText::new(currency.code()).with_large_type(currency.code()))
    .set_valid(true)
    .try_set_modifier(
        [ModifierKey::Cmd],
        Modifier::new()
            .with_subtitle(format!(
                "Copy {} ({}) {} to clipboard",
                home.name(),
                home.code(),
                home.flag()
            ))
            .with_arg(format!("{} {}", home.name(), home.code())),
    )?;
    Ok(PendingItem::new(item, Some(currency.flag())))
}

struct ResultActions {
    default_arg: String,
    command_modifier: Modifier,
}

fn result_actions(
    default_action: DefaultAction,
    website_url: &str,
    copy_arg: String,
    copy_subtitle: String,
    website_subtitle: &str,
) -> ResultActions {
    match default_action {
        DefaultAction::OpenWebsite => ResultActions {
            default_arg: website_url.to_owned(),
            command_modifier: Modifier::new()
                .with_subtitle(copy_subtitle)
                .with_arg(copy_arg),
        },
        DefaultAction::CopyToClipboard => ResultActions {
            default_arg: copy_arg,
            command_modifier: Modifier::new()
                .with_subtitle(website_subtitle)
                .with_arg(website_url),
        },
    }
}

/// Builds Alfred items from Numbat's public unit metadata.
#[must_use]
pub fn unit_items(listings: &[UnitListing]) -> Vec<PendingItem> {
    listings
        .iter()
        .map(|unit| {
            let aliases = unit.aliases.join(", ");
            let subtitle = if aliases.is_empty() {
                unit.dimension.clone()
            } else {
                format!("{} • {aliases}", unit.dimension)
            };
            let match_text = format!("{} [{}] {aliases}", unit.name, unit.identifier);
            let item = Item::with_arg(&unit.name, &unit.identifier)
                .set_uid(&unit.identifier)
                .set_subtitle(subtitle)
                .set_match_text(match_text)
                .set_text(ItemText::new(&unit.identifier).with_large_type(&unit.identifier))
                .set_valid(true);
            PendingItem::new(item, dimension_emoji(&unit.dimension))
        })
        .collect()
}

fn dimension_emoji(dimension: &str) -> Option<&'static str> {
    let lower = dimension.to_lowercase();
    [
        ("angle", "📐"),
        ("area", "🏠"),
        ("information", "💾"),
        ("energy", "☀"),
        ("force", "🐘"),
        ("fuel", "⛽"),
        ("illumin", "💡"),
        ("length", "📏"),
        ("mass", "⚖"),
        ("power", "⚡"),
        ("pressure", "🧯"),
        ("velocity", "🏎"),
        ("temperature", "🌡"),
        ("time", "⏱"),
        ("volume", "🧪"),
    ]
    .into_iter()
    .find_map(|(needle, emoji)| lower.contains(needle).then_some(emoji))
}

fn xe_url(from: Currency, to: Currency) -> Result<Url> {
    let mut url = Url::parse("https://www.xe.com/currencycharts")?;
    url.query_pairs_mut()
        .append_pair("from", from.code())
        .append_pair("to", to.code());
    Ok(url)
}

fn oanda_url(currency: Currency) -> Result<Url> {
    Url::parse(&format!(
        "https://www.oanda.com/currency-converter/en/currencies/majors/{}/",
        currency.code().to_lowercase()
    ))
    .map_err(Into::into)
}

fn wolfram_url(expression: &str) -> Result<Url> {
    let mut url = Url::parse("https://www.wolframalpha.com/input")?;
    url.query_pairs_mut().append_pair("i", expression);
    Ok(url)
}

fn dart_double(amount: &str) -> String {
    parse_decimal(amount).map_or_else(
        || amount.to_owned(),
        |value| {
            let value = value.normalize().to_string();
            if value.contains('.') {
                value
            } else {
                format!("{value}.0")
            }
        },
    )
}

fn legacy_error_message(message: &str) -> String {
    if message.starts_with("Can not convert") {
        message.to_owned()
    } else {
        "Invalid format.".to_owned()
    }
}

fn exchange_rate_subtitle(timestamp: Timestamp) -> String {
    let value = timestamp.to_zoned(jiff::tz::TimeZone::UTC);
    let month = usize::try_from(value.month() - 1)
        .ok()
        .and_then(|index| MONTHS.get(index))
        .copied()
        .unwrap_or("");
    let mut subtitle = format!(
        "Based on ECB exchange rates from {month} {}, {}",
        value.day(),
        value.year()
    );
    if value.hour() > 0 && value.minute() > 0 {
        let _ = write!(subtitle, " {:02}:{:02} UTC", value.hour(), value.minute());
    }
    subtitle
}

#[cfg(test)]
#[path = "tests/app.rs"]
mod tests;
