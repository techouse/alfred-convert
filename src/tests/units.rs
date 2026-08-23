use super::{
    CustomarySystem, LegacyConversion, UnitEngine, legacy_conversion,
    legacy_conversion_with_customary_system, legacy_unit, legacy_unit_with_customary_system,
};

const CONVERTIBLE_DART_ALIASES: &[&str] = &[
    "°",
    "deg",
    "'",
    "''",
    "\"",
    "rad",
    "m2",
    "cm2",
    "in2",
    "ft2",
    "mi2",
    "yd2",
    "mm2",
    "km2",
    "ha",
    "ac",
    "a",
    "b",
    "kb",
    "Mb",
    "Gb",
    "Tb",
    "Pb",
    "Eb",
    "Kibit",
    "Mibit",
    "Gibit",
    "Tibit",
    "Pibit",
    "Eibit",
    "B",
    "kB",
    "MB",
    "GB",
    "TB",
    "PB",
    "EB",
    "KiB",
    "MiB",
    "GiB",
    "TiB",
    "PiB",
    "EiB",
    "J",
    "j",
    "kJ",
    "cal",
    "kcal",
    "kwh",
    "eV",
    "ev",
    "ft⋅lbf",
    "ftlbf",
    "Wh",
    "BTU",
    "N",
    "n",
    "dyn",
    "lbf",
    "kgf",
    "pdl",
    "km/l",
    "l/100km",
    "us.mpg",
    "mpg",
    "lx",
    "fc",
    "m",
    "cm",
    "in",
    "ft",
    "M",
    "yd",
    "mi",
    "mm",
    "µm",
    "nm",
    "Å",
    "å",
    "pm",
    "km",
    "au",
    "ly",
    "pc",
    "th",
    "g",
    "hg",
    "kg",
    "lb",
    "oz",
    "t",
    "mg",
    "u",
    "ct",
    "cg",
    "dwt",
    "ozt",
    "st",
    "W",
    "mW",
    "kW",
    "MW",
    "GW",
    "eu.hp",
    "hp",
    "pa",
    "atm",
    "bar",
    "mbar",
    "psi",
    "mmhg",
    "torr",
    "kpa",
    "hpa",
    "inhg",
    "ksi",
    "MPa",
    "GPa",
    "m/s",
    "km/h",
    "kph",
    "mi/h",
    "mph",
    "kts",
    "ft/s",
    "min/km",
    "min/mi",
    "c",
    "°F",
    "F",
    "°C",
    "C",
    "K",
    "°Re",
    "Re",
    "°Rø",
    "Rø",
    "°De",
    "De",
    "°R",
    "R",
    "s",
    "ds",
    "cs",
    "ms",
    "µs",
    "ns",
    "min",
    "h",
    "d",
    "c.",
    "N·m",
    "Nm",
    "dyn·m",
    "dynm",
    "lbf·ft",
    "lbfft",
    "kgf·m",
    "kgfm",
    "pdl·m",
    "pdlm",
    "lbf·in",
    "m3",
    "l",
    "L",
    "gal",
    "us.gal",
    "pt",
    "us.pt",
    "ml",
    "tbsp.",
    "cup",
    "cm3",
    "ft3",
    "in3",
    "mm3",
    "fl.oz",
    "floz",
    "us.fl.oz",
    "us.floz",
    "US. liq. gi",
    "US. liq. qt",
    "fl",
    "pl",
    "nl",
    "µl",
    "dl",
    "cl",
    "tsp.",
];

#[test]
fn legacy_shorthand_should_be_evaluated_by_numbat() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let conversion = legacy_conversion("10 mi km")
        .ok_or_else(|| anyhow::anyhow!("legacy conversion missing"))?;
    let result = engine.evaluate_legacy(conversion)?;
    assert_eq!(
        (result.result.as_str(), result.copy_value.as_str()),
        ("10 mi = 16.093 km", "16.093 km")
    );
    assert_eq!(
        result.legacy_fact.as_deref(),
        Some("Based on the fact that 1 mi = 1.609 km")
    );
    Ok(())
}

#[test]
fn pace_should_convert_to_velocity_reciprocally() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let conversion = legacy_conversion("5 min/km km/h")
        .ok_or_else(|| anyhow::anyhow!("legacy conversion missing"))?;
    let result = engine.evaluate_legacy(conversion)?;
    assert_eq!(
        (result.result.as_str(), result.legacy_fact.as_deref()),
        (
            "5 min/km = 12 km/h",
            Some("Based on the fact that 1 min/km = 60 km/h")
        )
    );
    Ok(())
}

#[test]
fn velocity_should_convert_to_pace_reciprocally() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let conversion = legacy_conversion("12 km/h min/km")
        .ok_or_else(|| anyhow::anyhow!("legacy conversion missing"))?;
    let result = engine.evaluate_legacy(conversion)?;
    assert_eq!(
        (result.result.as_str(), result.legacy_fact.as_deref()),
        (
            "12 km/h = 5 min/km",
            Some("Based on the fact that 1 km/h = 60 min/km")
        )
    );
    Ok(())
}

#[test]
fn pace_should_convert_directly_to_another_pace_unit() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let conversion = legacy_conversion("5 min/km min/mi")
        .ok_or_else(|| anyhow::anyhow!("legacy conversion missing"))?;
    let result = engine.evaluate_legacy(conversion)?;
    assert_eq!(
        (result.result.as_str(), result.legacy_fact.as_deref()),
        (
            "5 min/km = 8.047 min/mi",
            Some("Based on the fact that 1 min/km = 1.609 min/mi")
        )
    );
    Ok(())
}

#[test]
fn pace_should_remain_incompatible_with_non_speed_units() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let conversion = legacy_conversion("5 min/km kg")
        .ok_or_else(|| anyhow::anyhow!("legacy conversion missing"))?;
    let error = engine
        .evaluate_legacy(conversion)
        .err()
        .ok_or_else(|| anyhow::anyhow!("incompatible legacy conversion unexpectedly succeeded"))?;
    assert_eq!(error.to_string(), "Can not convert min/km to \"kg\"");
    Ok(())
}

#[test]
fn legacy_symbols_should_match_dart_rendering() -> anyhow::Result<()> {
    for (alias, expected) in [
        ("'", "'"),
        ("''", "''"),
        ("\"", "''"),
        ("kts", "kts"),
        ("ozt", "oz t"),
        ("st", "st."),
    ] {
        let actual = legacy_unit(alias)
            .ok_or_else(|| anyhow::anyhow!("legacy unit {alias:?} missing"))?
            .symbol;
        assert_eq!(actual, expected, "unexpected symbol for {alias:?}");
    }
    Ok(())
}

#[test]
fn kilogram_force_torque_should_keep_the_dart_coefficient() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let conversion = legacy_conversion("1000 kgfm Nm")
        .ok_or_else(|| anyhow::anyhow!("legacy conversion missing"))?;
    let result = engine.evaluate_legacy(conversion)?;
    assert_eq!(result.result, "1,000 kgf·m = 9,807 N·m");
    Ok(())
}

#[test]
fn native_numbat_syntax_should_pass_through_unchanged() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let result = engine.evaluate_native("2in to cm")?;
    assert_eq!(
        (result.result.as_str(), result.copy_value.as_str()),
        ("5.08 cm", "5.08 cm")
    );
    Ok(())
}

#[test]
fn legacy_coefficients_should_match_units_converter_3_1_0() -> anyhow::Result<()> {
    let cases = [
        (
            "1e12 rad deg",
            "1,000,000,000,000 rad = 57,295,779,513,000 °",
            "Based on the fact that 1 rad = 57.296 °",
        ),
        (
            "1e9 ft⋅lbf J",
            "1,000,000,000 ft⋅lbf = 1,355,817,948.331 J",
            "Based on the fact that 1 ft⋅lbf = 1.356 J",
        ),
        (
            "1e6 lbf N",
            "1,000,000 lbf = 4,448,221.615 N",
            "Based on the fact that 1 lbf = 4.448 N",
        ),
        (
            "1 ly au",
            "1 ly = 63,241.1 au",
            "Based on the fact that 1 ly = 63,241.1 au",
        ),
        (
            "1 pc ly",
            "1 pc = 3.26 ly",
            "Based on the fact that 1 pc = 3.26 ly",
        ),
        (
            "1e28 u g",
            "10,000,000,000,000,000,000,000,000,000 u = 16,605.39 g",
            "Based on the fact that 1 u = 0 g",
        ),
        (
            "1e6 psi pa",
            "1,000,000 psi = 6,894,757,293.168 Pa",
            "Based on the fact that 1 psi = 6,894.757 Pa",
        ),
        (
            "1e9 torr pa",
            "1,000,000,000 torr = 133,322,368,421 Pa",
            "Based on the fact that 1 torr = 133.322 Pa",
        ),
        (
            "1e6 inhg pa",
            "1,000,000 inHg = 3,386,388,157.893 Pa",
            "Based on the fact that 1 inHg = 3,386.388 Pa",
        ),
        (
            "1e6 ksi pa",
            "1,000,000 ksi = 6,894,757,293,178.301 Pa",
            "Based on the fact that 1 ksi = 6,894,757.293 Pa",
        ),
        (
            "1e9 lbf·ft N·m",
            "1,000,000,000 lbf·ft = 1,355,817,949.025 N·m",
            "Based on the fact that 1 lbf·ft = 1.356 N·m",
        ),
        (
            "1e9 lbf·in N·m",
            "1,000,000,000 lbf·in = 112,984,829.085 N·m",
            "Based on the fact that 1 lbf·in = 0.113 N·m",
        ),
    ];
    let mut engine = UnitEngine::new()?;

    for (query, expected_result, expected_fact) in cases {
        let conversion = legacy_conversion(query)
            .ok_or_else(|| anyhow::anyhow!("legacy query should parse: {query}"))?;
        let evaluated = engine.evaluate_legacy(conversion)?;
        assert_eq!(evaluated.result, expected_result, "query: {query}");
        assert_eq!(
            evaluated.legacy_fact.as_deref(),
            Some(expected_fact),
            "query: {query}"
        );
    }
    Ok(())
}

#[test]
fn native_numbat_astronomy_definitions_should_remain_unchanged() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let evaluated = engine.evaluate_native("1 parsec to lightyear")?;
    assert_eq!(evaluated.result, "3.26156 ly");
    Ok(())
}

#[test]
fn compatibility_aliases_should_keep_imperial_gallon_semantics() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    let conversion =
        legacy_conversion("1 gal l").ok_or_else(|| anyhow::anyhow!("legacy conversion missing"))?;
    let result = engine.evaluate_legacy(conversion)?;
    assert_eq!(result.result, "1 imp gal = 4.546 l");
    Ok(())
}

#[test]
fn customary_system_preference_should_default_to_imperial() {
    assert_eq!(CustomarySystem::default(), CustomarySystem::Imperial);
    assert_eq!(
        CustomarySystem::from_preference(Some("imperial")),
        CustomarySystem::Imperial
    );
    assert_eq!(
        CustomarySystem::from_preference(Some("us_customary")),
        CustomarySystem::UsCustomary
    );
    assert_eq!(
        CustomarySystem::from_preference(None),
        CustomarySystem::Imperial
    );
    assert_eq!(
        CustomarySystem::from_preference(Some("unknown")),
        CustomarySystem::Imperial
    );
}

#[test]
fn customary_system_should_resolve_ambiguous_aliases_in_both_positions() -> anyhow::Result<()> {
    for alias in ["gal", "pt", "fl.oz", "floz", "tbsp.", "tsp.", "mpg"] {
        let imperial = legacy_unit_with_customary_system(alias, CustomarySystem::Imperial)
            .ok_or_else(|| anyhow::anyhow!("imperial alias {alias} missing"))?;
        let us = legacy_unit_with_customary_system(alias, CustomarySystem::UsCustomary)
            .ok_or_else(|| anyhow::anyhow!("US alias {alias} missing"))?;
        if alias == "mpg" {
            assert_ne!(imperial.special, us.special, "alias: {alias}");
        } else {
            assert_ne!(imperial.expression, us.expression, "alias: {alias}");
        }

        let companion = if alias == "mpg" { "km/l" } else { "ml" };
        let imperial_query = format!("1 {alias} {companion}");
        let us_query = format!("1 {companion} {alias}");
        let imperial_conversion =
            legacy_conversion_with_customary_system(&imperial_query, CustomarySystem::Imperial)
                .ok_or_else(|| anyhow::anyhow!("source alias: {alias}"))?;
        let us_conversion =
            legacy_conversion_with_customary_system(&us_query, CustomarySystem::UsCustomary)
                .ok_or_else(|| anyhow::anyhow!("target alias: {alias}"))?;
        assert_eq!(imperial_conversion.from, imperial, "source alias: {alias}");
        assert_eq!(us_conversion.to, us, "target alias: {alias}");
    }
    Ok(())
}

#[test]
fn customary_system_should_preserve_explicit_us_aliases_and_native_units() -> anyhow::Result<()> {
    for alias in ["us.gal", "us.pt", "us.fl.oz", "us.floz", "us.mpg"] {
        let imperial = legacy_unit_with_customary_system(alias, CustomarySystem::Imperial)
            .ok_or_else(|| anyhow::anyhow!("imperial lookup missing {alias}"))?;
        let us = legacy_unit_with_customary_system(alias, CustomarySystem::UsCustomary)
            .ok_or_else(|| anyhow::anyhow!("US lookup missing {alias}"))?;
        assert_eq!(imperial, us, "explicit alias changed: {alias}");
    }
    let mut engine = UnitEngine::new()?;
    assert_eq!(
        engine.evaluate_native("1 gallon to liter")?.result,
        "3.78541 l"
    );
    assert_eq!(
        engine.evaluate_native("1 imperial_gallon to liter")?.result,
        "4.54609 l"
    );
    assert_eq!(engine.evaluate_native("1 cup to mL")?.result, "236.588 ml");
    Ok(())
}

#[test]
fn customary_system_should_match_legacy_volume_and_mpg_coefficients() -> anyhow::Result<()> {
    let cases = [
        (
            "1 floz ml",
            "1 imp fl oz = 28.413 ml",
            "1 US fl oz = 29.574 ml",
        ),
        ("1 gal l", "1 imp gal = 4.546 l", "1 US gal = 3.785 l"),
        ("1 pt ml", "1 imp pt = 568.261 ml", "1 US pt = 473.176 ml"),
        ("1 tbsp. ml", "1 tbsp. = 14.207 ml", "1 tbsp. = 14.787 ml"),
        ("1 tsp. ml", "1 tsp. = 3.552 ml", "1 tsp. = 4.929 ml"),
        ("1 mpg km/l", "1 mpg = 0.354 km/l", "1 mpg = 0.425 km/l"),
    ];
    for (query, expected_imperial, expected_us) in cases {
        let mut engine = UnitEngine::new()?;
        let imperial = legacy_conversion_with_customary_system(query, CustomarySystem::Imperial)
            .ok_or_else(|| anyhow::anyhow!("imperial conversion missing: {query}"))?;
        assert_eq!(
            engine.evaluate_legacy(imperial)?.result,
            expected_imperial,
            "{query}"
        );

        let mut engine = UnitEngine::new()?;
        let us = legacy_conversion_with_customary_system(query, CustomarySystem::UsCustomary)
            .ok_or_else(|| anyhow::anyhow!("US conversion missing: {query}"))?;
        assert_eq!(engine.evaluate_legacy(us)?.result, expected_us, "{query}");
    }
    Ok(())
}

#[test]
fn listings_should_come_from_numbat_and_exclude_money() -> anyhow::Result<()> {
    let engine = UnitEngine::new()?;
    let listings = engine.listings();
    assert!(listings.iter().any(|unit| unit.identifier == "metre"));
    assert!(listings.iter().all(|unit| unit.dimension != "Money"));
    Ok(())
}

#[test]
fn every_convertible_dart_alias_should_resolve_and_evaluate() -> anyhow::Result<()> {
    let mut engine = UnitEngine::new()?;
    for alias in CONVERTIBLE_DART_ALIASES {
        let unit = legacy_unit(alias)
            .ok_or_else(|| anyhow::anyhow!("Dart alias {alias:?} is not mapped"))?;
        engine
            .evaluate_legacy(LegacyConversion {
                amount: "1",
                from: unit,
                to: unit,
            })
            .map_err(|error| {
                anyhow::anyhow!("Dart alias {alias:?} cannot be evaluated: {error}")
            })?;
    }
    Ok(())
}

#[test]
fn historically_non_convertible_numeral_aliases_should_remain_invalid() {
    for alias in [
        "dec",
        "decimal",
        "hex",
        "hexadecimal",
        "oct",
        "octal",
        "bin",
        "binary",
    ] {
        assert!(
            legacy_unit(alias).is_none(),
            "{alias} unexpectedly became a unit"
        );
    }
}
