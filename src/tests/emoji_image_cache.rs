use std::net::TcpListener;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use tempfile::TempDir;
use url::Url;

use super::{EmojiImageCache, image_filename, png_crc32};

#[test]
fn image_filename_should_preserve_meaningful_multi_scalar_sequences() -> anyhow::Result<()> {
    assert_eq!(image_filename("🇬🇧")?, "1f1ec-1f1e7.png");
    Ok(())
}

#[test]
fn image_filename_should_omit_presentation_selectors() -> anyhow::Result<()> {
    assert_eq!(image_filename("☀️")?, "2600.png");
    Ok(())
}

#[test]
fn image_filename_should_reject_selector_only_input() {
    assert!(image_filename("\u{fe0f}").is_err());
}

#[test]
fn image_filename_should_reject_empty_input() {
    assert!(image_filename("").is_err());
}

#[test]
fn duplicate_emojis_should_download_once_and_reuse_valid_cache() -> anyhow::Result<()> {
    let directory = TempDir::new()?;
    let fetches = Arc::new(AtomicUsize::new(0));
    let fetch_count = Arc::clone(&fetches);
    let image = valid_png();
    let cache = EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        Arc::new(move |_| {
            fetch_count.fetch_add(1, Ordering::SeqCst);
            Ok(image.clone())
        }),
        Arc::new(|_| {}),
    )?;
    let emojis = vec!["🇬🇧".to_owned(), "🇬🇧".to_owned()];
    let first = cache.resolve_many(&emojis);
    let second = cache.resolve_many(&emojis);
    assert_eq!(fetches.load(Ordering::SeqCst), 1);
    assert_eq!(first, second);
    assert!(first.iter().all(Option::is_some));
    Ok(())
}

#[test]
fn zero_byte_cache_entry_should_be_replaced_atomically() -> anyhow::Result<()> {
    let directory = TempDir::new()?;
    let target = directory.path().join("1f4be.png");
    std::fs::write(&target, [])?;
    let image = valid_png();
    let cache = EmojiImageCache::with_test_fetcher(
        directory.path().to_path_buf(),
        Arc::new(move |_| Ok(image.clone())),
        Arc::new(|_| {}),
    )?;
    assert_eq!(
        cache.resolve_many(&["💾".to_owned()]),
        vec![Some(target.clone())]
    );
    assert!(!std::fs::read(target)?.is_empty());
    assert!(std::fs::read_dir(directory.path())?.all(|entry| {
        entry.is_ok_and(|entry| !entry.file_name().to_string_lossy().ends_with(".tmp"))
    }));
    Ok(())
}

#[test]
fn batch_budget_should_bound_a_stalled_download() -> anyhow::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let server = thread::spawn(move || {
        if let Ok((_stream, _)) = listener.accept() {
            thread::sleep(Duration::from_millis(800));
        }
    });
    let directory = TempDir::new()?;
    let cache = EmojiImageCache::with_test_endpoint(
        directory.path().to_path_buf(),
        Url::parse(&format!("http://{address}/"))?,
        Duration::from_millis(100),
    )?;

    let started = Instant::now();
    let _ = cache.resolve_many(&["💾".to_owned()]);
    let elapsed = started.elapsed();
    server
        .join()
        .map_err(|_| anyhow::anyhow!("stalled image server panicked"))?;

    assert!(
        elapsed < Duration::from_millis(500),
        "batch exceeded its deadline tolerance: {elapsed:?}"
    );
    Ok(())
}

fn valid_png() -> Vec<u8> {
    let mut png = vec![137, 80, 78, 71, 13, 10, 26, 10];
    append_chunk(&mut png, *b"IHDR", &[0; 13]);
    append_chunk(&mut png, *b"IDAT", &[]);
    append_chunk(&mut png, *b"IEND", &[]);
    png
}

fn append_chunk(png: &mut Vec<u8>, kind: [u8; 4], data: &[u8]) {
    png.extend(u32::try_from(data.len()).unwrap_or_default().to_be_bytes());
    png.extend(kind);
    png.extend(data);
    let mut crc_input = kind.to_vec();
    crc_input.extend(data);
    png.extend(png_crc32(&crc_input).to_be_bytes());
}
