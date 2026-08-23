use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use anyhow::{Context, Result, anyhow, bail};
use rayon::ThreadPool;
use rayon::ThreadPoolBuilder;
use rayon::prelude::*;
use ureq::Agent;
use url::Url;

use super::http::platform_agent;

const IMAGE_CACHE_URL: &str =
    "https://raw.githubusercontent.com/joypixels/emoji-assets/master/png/64/";
const MAX_WORKERS: usize = 8;
const MAX_IMAGE_BYTES: u64 = 1024 * 1024;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const IMAGE_TIMEOUT: Duration = Duration::from_secs(3);
const BATCH_BUDGET: Duration = Duration::from_secs(5);
const PNG_SIGNATURE: [u8; 8] = [137, 80, 78, 71, 13, 10, 26, 10];

static NEXT_TEMP_FILE_ID: AtomicU64 = AtomicU64::new(0);
type Diagnostic = dyn Fn(String) + Send + Sync;

#[cfg(test)]
type TestFetcher = dyn Fn(&Url) -> Result<Vec<u8>> + Send + Sync;

/// Best-effort concurrent cache for `JoyPixels` image assets.
pub struct EmojiImageCache {
    directory: PathBuf,
    base_url: Url,
    agent: Agent,
    pool: ThreadPool,
    verbose: bool,
    diagnostic: Arc<Diagnostic>,
    batch_budget: Duration,
    #[cfg(test)]
    test_fetcher: Option<Arc<TestFetcher>>,
}

impl EmojiImageCache {
    /// Creates an image cache that downloads validated `JoyPixels` PNGs.
    ///
    /// # Errors
    /// Returns an error when the URL or bounded worker pool cannot be created.
    pub fn new(directory: impl Into<PathBuf>, verbose: bool) -> Result<Self> {
        let agent = platform_agent(CONNECT_TIMEOUT, IMAGE_TIMEOUT);
        let pool = ThreadPoolBuilder::new()
            .num_threads(MAX_WORKERS)
            .thread_name(|index| format!("convert-image-{index}"))
            .build()
            .map_err(|error| anyhow!("failed to create image worker pool: {error}"))?;
        Ok(Self {
            directory: directory.into(),
            base_url: Url::parse(IMAGE_CACHE_URL)?,
            agent,
            pool,
            verbose,
            diagnostic: Arc::new(|message| eprintln!("{message}")),
            batch_budget: BATCH_BUDGET,
            #[cfg(test)]
            test_fetcher: None,
        })
    }

    /// Resolves images in input order while downloading each distinct emoji once.
    #[must_use]
    pub fn resolve_many(&self, emojis: &[String]) -> Vec<Option<PathBuf>> {
        let deadline = Instant::now() + self.batch_budget;
        let unique = emojis.iter().map(String::as_str).collect::<BTreeSet<_>>();
        let unique = unique.into_iter().collect::<Vec<_>>();
        let resolved = self.pool.install(|| {
            unique
                .par_iter()
                .map(|emoji| ((*emoji).to_owned(), self.resolve_one(emoji, deadline)))
                .collect::<std::collections::BTreeMap<_, _>>()
        });
        emojis
            .iter()
            .map(|emoji| resolved.get(emoji).cloned().flatten())
            .collect()
    }

    fn resolve_one(&self, emoji: &str, deadline: Instant) -> Option<PathBuf> {
        let filename = match image_filename(emoji) {
            Ok(filename) => filename,
            Err(error) => {
                self.log(format!("could not resolve image for {emoji}: {error}"));
                return None;
            }
        };
        let target = self.directory.join(&filename);
        if self.valid_cached_image(&target) {
            return Some(target);
        }
        let image_url = match self.base_url.join(&filename) {
            Ok(url) => url,
            Err(error) => {
                self.log(format!("could not build image URL for {emoji}: {error}"));
                return None;
            }
        };
        let Some(remaining) = deadline
            .checked_duration_since(Instant::now())
            .filter(|remaining| !remaining.is_zero())
        else {
            self.log(format!(
                "skipping {emoji} because the image batch deadline elapsed"
            ));
            return None;
        };
        let bytes = match self.download(&image_url, remaining.min(IMAGE_TIMEOUT)) {
            Ok(bytes) => bytes,
            Err(error) => {
                self.log(format!("could not download image for {emoji}: {error}"));
                return None;
            }
        };
        if let Err(error) = self.write_atomically(&target, &bytes) {
            self.log(format!("could not cache image for {emoji}: {error}"));
            return None;
        }
        Some(target)
    }

    fn download(&self, url: &Url, timeout: Duration) -> Result<Vec<u8>> {
        #[cfg(test)]
        if let Some(fetcher) = &self.test_fetcher {
            return fetcher(url);
        }
        let mut response = self
            .agent
            .get(url.as_str())
            .config()
            .timeout_global(Some(timeout))
            .build()
            .call()
            .map_err(|error| anyhow!("image request failed: {error}"))?;
        let bytes = response
            .body_mut()
            .with_config()
            .limit(MAX_IMAGE_BYTES)
            .read_to_vec()
            .map_err(|error| anyhow!("failed to read image response: {error}"))?;
        validate_png(&bytes)?;
        Ok(bytes)
    }

    fn write_atomically(&self, target: &Path, bytes: &[u8]) -> Result<()> {
        validate_png(bytes)?;
        fs::create_dir_all(&self.directory)
            .with_context(|| format!("failed to create {}", self.directory.display()))?;
        if self.valid_cached_image(target) {
            return Ok(());
        }
        let file_name = target
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| anyhow!("cache target does not have a UTF-8 file name"))?;
        let temp_id = NEXT_TEMP_FILE_ID.fetch_add(1, Ordering::Relaxed);
        let temporary = self.directory.join(format!(
            ".{file_name}.{}.{}.tmp",
            std::process::id(),
            temp_id
        ));
        fs::write(&temporary, bytes)
            .with_context(|| format!("failed to write {}", temporary.display()))?;
        match fs::rename(&temporary, target) {
            Ok(()) => Ok(()),
            Err(_error) if self.valid_cached_image(target) => {
                let _ = fs::remove_file(&temporary);
                Ok(())
            }
            Err(error) => {
                let _ = fs::remove_file(&temporary);
                bail!("failed to move image into cache: {error}");
            }
        }
    }

    fn valid_cached_image(&self, path: &Path) -> bool {
        if !path.is_file() {
            return false;
        }
        let cached = fs::metadata(path)
            .ok()
            .filter(|metadata| metadata.len() <= MAX_IMAGE_BYTES)
            .and_then(|_| fs::read(path).ok());
        if cached
            .as_deref()
            .is_some_and(|bytes| validate_png(bytes).is_ok())
        {
            return true;
        }
        self.log(format!("removing invalid cached image {}", path.display()));
        let _ = fs::remove_file(path);
        false
    }

    fn log(&self, message: String) {
        if self.verbose {
            (self.diagnostic)(message);
        }
    }

    #[cfg(test)]
    fn with_test_fetcher(
        directory: PathBuf,
        fetcher: Arc<TestFetcher>,
        diagnostic: Arc<Diagnostic>,
    ) -> Result<Self> {
        let pool = ThreadPoolBuilder::new().num_threads(2).build()?;
        Ok(Self {
            directory,
            base_url: Url::parse("https://images.example/")?,
            agent: platform_agent(CONNECT_TIMEOUT, IMAGE_TIMEOUT),
            pool,
            verbose: true,
            diagnostic,
            batch_budget: BATCH_BUDGET,
            test_fetcher: Some(fetcher),
        })
    }

    #[cfg(test)]
    fn with_test_endpoint(
        directory: PathBuf,
        base_url: Url,
        batch_budget: Duration,
    ) -> Result<Self> {
        let pool = ThreadPoolBuilder::new().num_threads(2).build()?;
        Ok(Self {
            directory,
            base_url,
            agent: platform_agent(CONNECT_TIMEOUT, IMAGE_TIMEOUT),
            pool,
            verbose: true,
            diagnostic: Arc::new(|_| {}),
            batch_budget,
            test_fetcher: None,
        })
    }
}

fn image_filename(emoji: &str) -> Result<String> {
    if emoji.is_empty() {
        bail!("emoji must not be empty");
    }
    let scalars = emoji
        .chars()
        .filter(|character| !matches!(character, '\u{fe0e}' | '\u{fe0f}'))
        .map(|character| format!("{:x}", u32::from(character)))
        .collect::<Vec<_>>();
    if scalars.is_empty() {
        bail!("emoji contains no asset-bearing Unicode scalar");
    }
    Ok(format!("{}.png", scalars.join("-")))
}

fn validate_png(bytes: &[u8]) -> Result<()> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > MAX_IMAGE_BYTES {
        bail!("image exceeds the 1 MiB limit");
    }
    if !bytes.starts_with(&PNG_SIGNATURE) {
        bail!("image response is not a PNG");
    }
    let mut position = PNG_SIGNATURE.len();
    let mut saw_header = false;
    let mut saw_image_data = false;
    while position < bytes.len() {
        let remaining = &bytes[position..];
        if remaining.len() < 12 {
            bail!("PNG chunk is incomplete");
        }
        let data_length = u32::from_be_bytes(
            remaining[..4]
                .try_into()
                .map_err(|_| anyhow!("PNG chunk length is invalid"))?,
        );
        let data_start = position + 8;
        let data_end = data_start
            .checked_add(data_length as usize)
            .ok_or_else(|| anyhow!("PNG chunk length overflows"))?;
        let chunk_end = data_end
            .checked_add(4)
            .ok_or_else(|| anyhow!("PNG chunk length overflows"))?;
        if chunk_end > bytes.len() {
            bail!("PNG chunk is incomplete");
        }
        let chunk_type = &bytes[position + 4..data_start];
        let data = &bytes[data_start..data_end];
        let expected_crc = u32::from_be_bytes(
            bytes[data_end..chunk_end]
                .try_into()
                .map_err(|_| anyhow!("PNG CRC is invalid"))?,
        );
        if png_crc32(&bytes[position + 4..data_end]) != expected_crc {
            bail!("PNG chunk CRC does not match");
        }
        match chunk_type {
            b"IHDR" if !saw_header && data.len() == 13 => saw_header = true,
            b"IHDR" => bail!("PNG header is invalid or duplicated"),
            b"IDAT" if saw_header => saw_image_data = true,
            b"IDAT" => bail!("PNG data appears before its header"),
            b"IEND"
                if saw_header && saw_image_data && data.is_empty() && chunk_end == bytes.len() =>
            {
                return Ok(());
            }
            b"IEND" => bail!("PNG end chunk is invalid"),
            _ if !saw_header => bail!("PNG does not begin with an IHDR chunk"),
            _ => {}
        }
        position = chunk_end;
    }
    bail!("PNG is missing its end chunk")
}

fn png_crc32(bytes: &[u8]) -> u32 {
    let mut crc = u32::MAX;
    for byte in bytes {
        crc ^= u32::from(*byte);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xedb8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

#[cfg(test)]
#[path = "../tests/emoji_image_cache.rs"]
mod tests;
