use std::io::Read;
use std::time::Duration;

use anyhow::{Context, Result, bail};
use jiff::civil::{Date, Weekday};
use jiff::fmt::rfc2822;
use jiff::tz::TimeZone;
use jiff::{Timestamp, ToSpan};
use quick_xml::Reader;
use quick_xml::events::{BytesStart, Event};
use rust_decimal::Decimal;
use ureq::Agent;
use url::Url;

use super::{CURRENCIES, ExchangeRateCache, ExchangeRates};
use crate::services::http::platform_agent;

const ECB_URL: &str = "https://www.ecb.europa.eu/stats/eurofxref/eurofxref-daily.xml";
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const REQUEST_TIMEOUT: Duration = Duration::from_secs(5);
const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;

/// Synchronous client for the ECB daily reference-rate feed.
#[derive(Clone, Debug)]
pub struct EcbClient {
    endpoint: Url,
    agent: Agent,
}

impl EcbClient {
    /// Creates a client with explicit macOS platform certificate verification.
    ///
    /// # Errors
    /// Returns an error if the fixed ECB endpoint cannot be parsed.
    pub fn new() -> Result<Self> {
        Ok(Self {
            endpoint: Url::parse(ECB_URL)?,
            agent: platform_agent(CONNECT_TIMEOUT, REQUEST_TIMEOUT),
        })
    }

    /// Returns current rates, refreshing the cache when policy requires it.
    ///
    /// Network and response failures fall back to any cached entry.
    pub fn latest(
        &self,
        cache: &ExchangeRateCache,
        now: Timestamp,
        diagnostic: &mut dyn FnMut(String),
    ) -> Option<ExchangeRates> {
        let cached = cache.load(diagnostic);
        let refresh = cached
            .as_ref()
            .is_none_or(|rates| should_refresh(rates.date, now).unwrap_or(true));
        if !refresh {
            return cached;
        }

        match self.download(now) {
            Ok(latest) => {
                if let Err(error) = cache.store(&latest) {
                    diagnostic(format!("could not store ECB rates: {error:#}"));
                }
                Some(latest)
            }
            Err(error) => {
                diagnostic(format!("could not refresh ECB rates: {error:#}"));
                cached
            }
        }
    }

    fn download(&self, now: Timestamp) -> Result<ExchangeRates> {
        let request = self
            .agent
            .get(self.endpoint.as_str())
            .config()
            .http_status_as_error(false)
            .timeout_connect(Some(CONNECT_TIMEOUT))
            .timeout_global(Some(REQUEST_TIMEOUT))
            .build();
        let mut response = request
            .call()
            .with_context(|| format!("ECB request failed for {}", self.endpoint))?;
        let status = response.status().as_u16();
        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|value| value.to_str().ok())
            .map(str::to_owned);
        let mut reader = response
            .body_mut()
            .with_config()
            .limit(MAX_RESPONSE_BYTES + 1)
            .reader()
            .take(MAX_RESPONSE_BYTES + 1);
        let mut body = Vec::new();
        reader.read_to_end(&mut body).with_context(|| {
            format!(
                "ECB response body read failed for {} with HTTP status {status}",
                self.endpoint
            )
        })?;
        if body.len() as u64 > MAX_RESPONSE_BYTES {
            bail!(
                "ECB response from {} with HTTP status {status} exceeds {MAX_RESPONSE_BYTES} bytes",
                self.endpoint
            );
        }
        if !(200..=299).contains(&status) {
            let bounded = String::from_utf8_lossy(&body);
            bail!(
                "ECB request to {} failed with HTTP status {status}: {}",
                self.endpoint,
                bounded.chars().take(1024).collect::<String>()
            );
        }
        let xml = std::str::from_utf8(&body).with_context(|| {
            format!(
                "ECB response from {} with HTTP status {status} is not valid UTF-8",
                self.endpoint
            )
        })?;
        parse_exchange_rates(xml, last_modified.as_deref(), now)
            .with_context(|| format!("invalid ECB response from {}", self.endpoint))
    }

    #[cfg(test)]
    pub(crate) fn with_endpoint(endpoint: Url, agent: Agent) -> Self {
        Self { endpoint, agent }
    }
}

/// Returns whether cached rates should be refreshed at `now`.
///
/// # Errors
/// Returns an error if the Brussels time-zone database is unavailable.
pub fn should_refresh(cached_at: Timestamp, now: Timestamp) -> Result<bool> {
    let cached_date = cached_at.to_zoned(TimeZone::UTC).date();
    let today = now.to_zoned(TimeZone::UTC).date();
    if cached_date >= today {
        return Ok(false);
    }
    if (today - cached_date).get_days() > 2 {
        return Ok(true);
    }

    let brussels = now.to_zoned(TimeZone::get("Europe/Brussels")?);
    let date = brussels.date();
    if is_target_closing_day(date)? {
        return Ok(false);
    }
    Ok(brussels.hour() > 16 || (brussels.hour() == 16 && brussels.minute() > 0))
}

fn parse_exchange_rates(
    xml: &str,
    last_modified: Option<&str>,
    now: Timestamp,
) -> Result<ExchangeRates> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut source_date = None;
    let mut rates = std::collections::BTreeMap::from([("EUR".to_owned(), Decimal::ONE)]);

    loop {
        match reader.read_event() {
            Ok(Event::Start(element) | Event::Empty(element))
                if element.name().as_ref() == "Cube" =>
            {
                read_cube(&element, &mut source_date, &mut rates)?;
            }
            Ok(Event::Eof) => break,
            Ok(_) => {}
            Err(error) => return Err(error).context("XML document cannot be parsed"),
        }
    }
    if rates.len() == 1 {
        bail!("ECB response contains no supported rates");
    }

    let date = last_modified
        .and_then(|value| rfc2822::parse(value).ok())
        .map(|value| value.timestamp())
        .or_else(|| source_date.and_then(|date| midnight_utc(date).ok()))
        .unwrap_or(now);
    Ok(ExchangeRates { date, rates })
}

fn read_cube(
    element: &BytesStart<'_>,
    source_date: &mut Option<Date>,
    rates: &mut std::collections::BTreeMap<String, Decimal>,
) -> Result<()> {
    let mut currency = None;
    let mut rate = None;
    for attribute in element.attributes() {
        let attribute = attribute.context("invalid XML attribute")?;
        let value = attribute.value;
        match attribute.key.as_ref() {
            "time" => *source_date = value.parse::<Date>().ok(),
            "currency" => currency = Some(value.into_owned()),
            "rate" => rate = Some(value.into_owned()),
            _ => {}
        }
    }
    if let (Some(currency), Some(rate)) = (currency, rate)
        && CURRENCIES.iter().any(|known| known.code() == currency)
    {
        rates.insert(
            currency.clone(),
            rate.parse::<Decimal>()
                .with_context(|| format!("invalid ECB rate for {currency}"))?,
        );
    }
    Ok(())
}

fn midnight_utc(date: Date) -> Result<Timestamp> {
    Ok(date.at(0, 0, 0, 0).to_zoned(TimeZone::UTC)?.timestamp())
}

fn is_fixed_holiday(date: Date) -> bool {
    matches!((date.month(), date.day()), (1 | 5, 1) | (12, 25 | 26))
}

fn is_target_closing_day(date: Date) -> Result<bool> {
    Ok(
        matches!(date.weekday(), Weekday::Saturday | Weekday::Sunday)
            || is_fixed_holiday(date)
            || is_easter_holiday(date)?,
    )
}

fn is_easter_holiday(date: Date) -> Result<bool> {
    let easter = computus::gregorian_jiff_date(i32::from(date.year()))?;
    Ok(date == easter.checked_sub(2.days())? || date == easter.checked_add(1.day())?)
}

#[cfg(test)]
#[path = "../tests/ecb.rs"]
mod tests;
