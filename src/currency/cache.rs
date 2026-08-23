use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::{Context, Result, anyhow, bail};
use jiff::Timestamp;
use rmpv::Value;
use rust_decimal::Decimal;
use serde::Deserialize;

use super::ExchangeRates;

const LEGACY_STASH_HEADER_BYTES: usize = 40;
static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);

/// Durable ECB cache with one-time Dart Stash import support.
#[derive(Clone, Debug)]
pub struct ExchangeRateCache {
    directory: PathBuf,
}

#[derive(Deserialize)]
struct LegacyExchangeRates {
    date: String,
    rates: Vec<LegacyExchangeRate>,
}

#[derive(Deserialize)]
struct LegacyExchangeRate {
    currency: String,
    rate: String,
}

impl ExchangeRateCache {
    /// Creates a cache rooted in the workflow directory.
    #[must_use]
    pub fn new(workflow_directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: workflow_directory.into().join("exchange_rates_cache"),
        }
    }

    /// Loads the Rust JSON cache, or imports the legacy Dart entry once.
    pub fn load(&self, diagnostic: &mut dyn FnMut(String)) -> Option<ExchangeRates> {
        let json_path = self.json_path();
        if json_path.is_file() {
            match Self::load_json(&json_path) {
                Ok(rates) => return Some(rates),
                Err(error) => diagnostic(format!(
                    "could not read ECB cache {}: {error:#}",
                    json_path.display()
                )),
            }
        }

        let legacy_path = self.legacy_path();
        if !legacy_path.is_file() {
            return None;
        }
        match Self::load_legacy(&legacy_path) {
            Ok(rates) => {
                if let Err(error) = self.store(&rates) {
                    diagnostic(format!(
                        "could not persist imported ECB cache {}: {error:#}",
                        self.json_path().display()
                    ));
                }
                Some(rates)
            }
            Err(error) => {
                diagnostic(format!(
                    "could not import legacy ECB cache {}: {error:#}",
                    legacy_path.display()
                ));
                None
            }
        }
    }

    /// Stores rates as JSON using an atomic rename.
    ///
    /// # Errors
    /// Returns an error when serialization or filesystem operations fail.
    pub fn store(&self, rates: &ExchangeRates) -> Result<()> {
        fs::create_dir_all(&self.directory)
            .with_context(|| format!("failed to create {}", self.directory.display()))?;
        let bytes = serde_json::to_vec(rates).context("failed to serialize ECB rates")?;
        let target = self.json_path();
        let temp_id = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = self.directory.join(format!(
            ".latest.json.{}.{}.tmp",
            std::process::id(),
            temp_id
        ));
        fs::write(&temporary, bytes)
            .with_context(|| format!("failed to write {}", temporary.display()))?;
        if let Err(error) = fs::rename(&temporary, &target) {
            let _ = fs::remove_file(&temporary);
            return Err(error).with_context(|| format!("failed to replace {}", target.display()));
        }
        Ok(())
    }

    fn load_json(path: &Path) -> Result<ExchangeRates> {
        let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let rates = serde_json::from_slice(&bytes).context("invalid ECB cache JSON")?;
        validate_rates(rates)
    }

    fn load_legacy(path: &Path) -> Result<ExchangeRates> {
        let bytes = fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let payload = bytes
            .get(LEGACY_STASH_HEADER_BYTES..)
            .ok_or_else(|| anyhow!("legacy Stash entry is shorter than its header"))?;
        let outer = rmpv::decode::read_value(&mut Cursor::new(payload))
            .context("invalid outer MessagePack document")?;
        let Value::Ext(0, inner) = outer else {
            bail!("legacy Stash entry does not contain extension type 0");
        };
        let value = rmpv::decode::read_value(&mut Cursor::new(inner))
            .context("invalid inner MessagePack document")?;
        let legacy: LegacyExchangeRates =
            rmpv::ext::from_value(value).context("invalid legacy ECB cache fields")?;
        let date = legacy
            .date
            .parse::<Timestamp>()
            .context("invalid legacy ECB timestamp")?;
        let mut rates = std::collections::BTreeMap::new();
        for rate in legacy.rates {
            let value = rate
                .rate
                .parse::<Decimal>()
                .with_context(|| format!("invalid rate for {}", rate.currency))?;
            rates.insert(rate.currency, value);
        }
        validate_rates(ExchangeRates { date, rates })
    }

    fn json_path(&self) -> PathBuf {
        self.directory.join("latest.json")
    }

    fn legacy_path(&self) -> PathBuf {
        self.directory.join("latest")
    }
}

fn validate_rates(rates: ExchangeRates) -> Result<ExchangeRates> {
    if rates.rates.get("EUR") != Some(&Decimal::ONE) {
        bail!("ECB cache must contain EUR at rate 1");
    }
    if rates
        .rates
        .iter()
        .any(|(code, rate)| code.len() != 3 || rate.is_sign_negative() || rate.is_zero())
    {
        bail!("ECB cache contains an invalid currency rate");
    }
    Ok(rates)
}

#[cfg(test)]
#[path = "../tests/currency_cache.rs"]
mod tests;
