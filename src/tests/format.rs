use rust_decimal::Decimal;

use super::{format_decimal, parse_decimal};

#[test]
fn format_decimal_should_match_dart_rounding_and_grouping() {
    let cases = [
        ("1.2345", "1.235"),
        ("-1.2345", "-1.235"),
        ("0.0005", "0.001"),
        ("0.0004", "0"),
        ("1000", "1,000"),
        ("12.340", "12.34"),
    ];
    for (input, expected) in cases {
        let value = input.parse::<Decimal>();
        assert_eq!(value.map(format_decimal), Ok(expected.to_owned()));
    }
}

#[test]
fn parse_decimal_should_accept_scientific_notation() {
    assert_eq!(
        parse_decimal("1e3").map(format_decimal).as_deref(),
        Some("1,000")
    );
}
