use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use flate2::Compression;
use flate2::write::GzEncoder;
use jiff::Timestamp;
use jiff::civil::Date;
use tempfile::TempDir;
use url::Url;

use super::{
    CONNECT_TIMEOUT, EcbClient, ExchangeRateCache, MAX_RESPONSE_BYTES, REQUEST_TIMEOUT,
    is_fixed_holiday, is_target_closing_day, parse_exchange_rates, platform_agent, should_refresh,
};

#[test]
fn current_cache_should_not_refresh() -> anyhow::Result<()> {
    let now = "2026-08-21T17:00:00Z".parse::<Timestamp>()?;
    assert!(!should_refresh(now, now)?);
    Ok(())
}

#[test]
fn cache_older_than_two_days_should_refresh_even_on_weekend() -> anyhow::Result<()> {
    let cached = "2026-08-20T00:00:00Z".parse::<Timestamp>()?;
    let now = "2026-08-23T12:00:00Z".parse::<Timestamp>()?;
    assert!(should_refresh(cached, now)?);
    Ok(())
}

#[test]
fn recent_cache_should_not_refresh_on_weekend() -> anyhow::Result<()> {
    let cached = "2026-08-21T00:00:00Z".parse::<Timestamp>()?;
    let now = "2026-08-22T15:01:00Z".parse::<Timestamp>()?;
    assert!(!should_refresh(cached, now)?);
    Ok(())
}

#[test]
fn recent_cache_should_refresh_after_brussels_cutoff() -> anyhow::Result<()> {
    let cached = "2026-08-20T00:00:00Z".parse::<Timestamp>()?;
    let now = "2026-08-21T14:01:00Z".parse::<Timestamp>()?;
    assert!(should_refresh(cached, now)?);
    Ok(())
}

#[test]
fn successful_response_should_parse_supported_rates_and_cache_them() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0"?><Envelope><Cube><Cube time="2026-08-21"><Cube currency="USD" rate="1.1732"/><Cube currency="GBP" rate="0.8665"/></Cube></Cube></Envelope>"#;
    let endpoint = serve_once(
        200,
        &[("Last-Modified", "Fri, 21 Aug 2026 14:00:00 GMT")],
        xml.as_bytes(),
    )?;
    let client =
        EcbClient::with_endpoint(endpoint, platform_agent(CONNECT_TIMEOUT, REQUEST_TIMEOUT));
    let directory = TempDir::new()?;
    let cache = ExchangeRateCache::new(directory.path());
    let now = "2026-08-21T17:00:00Z".parse::<Timestamp>()?;
    let rates = client
        .latest(&cache, now, &mut |_| {})
        .ok_or_else(|| anyhow::anyhow!("ECB response was not returned"))?;
    assert_eq!(
        rates.rates.get("USD").map(ToString::to_string).as_deref(),
        Some("1.1732")
    );
    assert!(
        directory
            .path()
            .join("exchange_rates_cache/latest.json")
            .is_file()
    );
    Ok(())
}

#[test]
fn xml_source_date_should_remain_on_the_same_utc_day() -> anyhow::Result<()> {
    let xml = r#"<?xml version="1.0"?><Envelope><Cube><Cube time="2026-08-21"><Cube currency="USD" rate="1.1732"/></Cube></Cube></Envelope>"#;
    let now = "2026-08-21T17:00:00Z".parse::<Timestamp>()?;
    let rates = parse_exchange_rates(xml, None, now)?;
    assert_eq!(
        (
            rates.date.to_string().as_str(),
            should_refresh(rates.date, now)?
        ),
        ("2026-08-21T00:00:00Z", false)
    );
    Ok(())
}

#[test]
fn failed_refresh_should_return_stale_cache_with_bounded_diagnostic() -> anyhow::Result<()> {
    let endpoint = serve_once(503, &[], b"temporarily unavailable")?;
    let client =
        EcbClient::with_endpoint(endpoint, platform_agent(CONNECT_TIMEOUT, REQUEST_TIMEOUT));
    let directory = TempDir::new()?;
    let cache = ExchangeRateCache::new(directory.path());
    let stale = crate::currency::ExchangeRates {
        date: "2026-08-01T00:00:00Z".parse()?,
        rates: std::collections::BTreeMap::from([
            ("EUR".to_owned(), rust_decimal::Decimal::ONE),
            ("USD".to_owned(), rust_decimal::Decimal::new(11_732, 4)),
        ]),
    };
    cache.store(&stale)?;
    let mut diagnostics = Vec::new();
    let actual = client.latest(&cache, "2026-08-21T17:00:00Z".parse()?, &mut |message| {
        diagnostics.push(message);
    });
    assert_eq!(actual, Some(stale));
    assert!(diagnostics.join("\n").contains("HTTP status 503"));
    Ok(())
}

#[test]
fn compressed_response_should_enforce_the_decoded_body_limit() -> anyhow::Result<()> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
    encoder.write_all(&vec![b'x'; usize::try_from(MAX_RESPONSE_BYTES)? + 1])?;
    let body = encoder.finish()?;
    let endpoint = serve_once(200, &[("Content-Encoding", "gzip")], &body)?;
    let client =
        EcbClient::with_endpoint(endpoint, platform_agent(CONNECT_TIMEOUT, REQUEST_TIMEOUT));
    let error = client
        .download("2026-08-21T17:00:00Z".parse()?)
        .err()
        .ok_or_else(|| anyhow::anyhow!("oversized decoded body unexpectedly succeeded"))?;
    assert!(error.to_string().contains("exceeds 2097152 bytes"));
    Ok(())
}

#[test]
fn malformed_gzip_should_be_a_refresh_failure() -> anyhow::Result<()> {
    let endpoint = serve_once(200, &[("Content-Encoding", "gzip")], b"not gzip")?;
    let client =
        EcbClient::with_endpoint(endpoint, platform_agent(CONNECT_TIMEOUT, REQUEST_TIMEOUT));
    let error = client
        .download("2026-08-21T17:00:00Z".parse()?)
        .err()
        .ok_or_else(|| anyhow::anyhow!("malformed gzip unexpectedly succeeded"))?;
    assert!(error.to_string().contains("body read failed"));
    Ok(())
}

#[test]
fn good_friday_should_be_a_target_closing_day_across_years() -> anyhow::Result<()> {
    for date in ["2024-03-29", "2025-04-18", "2026-04-03", "2038-04-23"] {
        assert!(
            is_target_closing_day(date.parse::<Date>()?)?,
            "date: {date}"
        );
    }
    Ok(())
}

#[test]
fn easter_monday_should_be_a_target_closing_day_across_years() -> anyhow::Result<()> {
    for date in ["2024-04-01", "2025-04-21", "2026-04-06", "2038-04-26"] {
        assert!(
            is_target_closing_day(date.parse::<Date>()?)?,
            "date: {date}"
        );
    }
    Ok(())
}

#[test]
fn fixed_target_holidays_should_remain_closing_days() -> anyhow::Result<()> {
    for date in ["2026-01-01", "2026-05-01", "2026-12-25", "2026-12-26"] {
        assert!(is_fixed_holiday(date.parse::<Date>()?), "date: {date}");
    }
    Ok(())
}

#[test]
fn ordinary_weekday_should_not_be_a_target_closing_day() -> anyhow::Result<()> {
    assert!(!is_target_closing_day("2026-08-21".parse::<Date>()?)?);
    Ok(())
}

#[test]
fn non_target_religious_holidays_should_remain_working_days() -> anyhow::Result<()> {
    for date in ["2026-05-14", "2026-05-25"] {
        assert!(
            !is_target_closing_day(date.parse::<Date>()?)?,
            "date: {date}"
        );
    }
    Ok(())
}

#[test]
fn weekend_should_be_a_target_closing_day() -> anyhow::Result<()> {
    for date in ["2026-08-22", "2026-08-23"] {
        assert!(
            is_target_closing_day(date.parse::<Date>()?)?,
            "date: {date}"
        );
    }
    Ok(())
}

#[test]
fn computus_easter_should_match_the_dart_table_through_2050() -> anyhow::Result<()> {
    let expected = [
        "2025-04-20",
        "2026-04-05",
        "2027-03-28",
        "2028-04-16",
        "2029-04-01",
        "2030-04-21",
        "2031-04-13",
        "2032-03-28",
        "2033-04-17",
        "2034-04-09",
        "2035-03-25",
        "2036-04-13",
        "2037-04-05",
        "2038-04-25",
        "2039-04-10",
        "2040-04-01",
        "2041-04-21",
        "2042-04-06",
        "2043-03-29",
        "2044-04-17",
        "2045-04-09",
        "2046-03-25",
        "2047-04-14",
        "2048-04-05",
        "2049-04-18",
        "2050-04-10",
    ];
    for (offset, expected) in expected.into_iter().enumerate() {
        let year = i32::try_from(2025 + offset)?;
        assert_eq!(computus::gregorian_jiff_date(year)?.to_string(), expected);
    }
    Ok(())
}

fn serve_once(status: u16, headers: &[(&str, &str)], body: &[u8]) -> anyhow::Result<Url> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let address = listener.local_addr()?;
    let headers = headers
        .iter()
        .fold(String::new(), |mut output, (name, value)| {
            let _ = write!(output, "{name}: {value}\r\n");
            output
        });
    let response = format!(
        "HTTP/1.1 {status} Test\r\nContent-Length: {}\r\n{headers}Connection: close\r\n\r\n",
        body.len()
    );
    let body = body.to_vec();
    thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request);
            let _ = stream.write_all(response.as_bytes());
            let _ = stream.write_all(&body);
        }
    });
    Ok(Url::parse(&format!("http://{address}/rates.xml"))?)
}
