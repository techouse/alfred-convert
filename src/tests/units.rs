use super::normalize::{FuelUnit, SpecialUnit};
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

const CANONICAL_LEGACY_ALIASES: &[&str] = &[
    "kWh", "Pa", "kPa", "hPa", "inHg", "m²", "cm²", "in²", "ft²", "mi²", "yd²", "mm²", "km²", "m³",
    "cm³", "ft³", "in³", "mm³",
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
    assert_eq!(result.result, "1 imp gal = 4.546 L");
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
        "3.78541 L"
    );
    assert_eq!(
        engine.evaluate_native("1 imperial_gallon to liter")?.result,
        "4.54609 L"
    );
    assert_eq!(engine.evaluate_native("1 cup to mL")?.result, "236.588 mL");
    Ok(())
}

#[test]
fn legacy_liter_aliases_should_use_uppercase_si_symbols() {
    for (alias, expected_symbol) in [
        ("l", "L"),
        ("L", "L"),
        ("ml", "mL"),
        ("mL", "mL"),
        ("fl", "fL"),
        ("fL", "fL"),
        ("pl", "pL"),
        ("pL", "pL"),
        ("nl", "nL"),
        ("nL", "nL"),
        ("µl", "µL"),
        ("µL", "µL"),
        ("dl", "dL"),
        ("dL", "dL"),
        ("cl", "cL"),
        ("cL", "cL"),
        ("km/l", "km/L"),
        ("km/L", "km/L"),
        ("l/100km", "L/100km"),
        ("L/100km", "L/100km"),
    ] {
        assert_eq!(
            legacy_unit(alias).map(|unit| unit.symbol),
            Some(expected_symbol),
            "{alias}"
        );
    }
}

#[test]
fn canonical_symbols_should_resolve_as_legacy_aliases() -> anyhow::Result<()> {
    for (historical, canonical) in [
        ("kwh", "kWh"),
        ("pa", "Pa"),
        ("kpa", "kPa"),
        ("hpa", "hPa"),
        ("inhg", "inHg"),
        ("m2", "m²"),
        ("cm2", "cm²"),
        ("in2", "in²"),
        ("ft2", "ft²"),
        ("mi2", "mi²"),
        ("yd2", "yd²"),
        ("mm2", "mm²"),
        ("km2", "km²"),
        ("m3", "m³"),
        ("cm3", "cm³"),
        ("ft3", "ft³"),
        ("in3", "in³"),
        ("mm3", "mm³"),
    ] {
        let historical_unit =
            legacy_unit(historical).ok_or_else(|| anyhow::anyhow!("{historical} missing"))?;
        let canonical_unit =
            legacy_unit(canonical).ok_or_else(|| anyhow::anyhow!("{canonical} missing"))?;
        assert_eq!(canonical_unit, historical_unit, "{canonical}");
    }

    let mut engine = UnitEngine::new()?;
    for alias in CANONICAL_LEGACY_ALIASES {
        let unit = legacy_unit(alias).ok_or_else(|| anyhow::anyhow!("{alias} missing"))?;
        engine
            .evaluate_legacy(LegacyConversion {
                amount: "1",
                from: unit,
                to: unit,
            })
            .map_err(|error| anyhow::anyhow!("{alias} cannot be evaluated: {error}"))?;
    }
    Ok(())
}

#[test]
fn canonical_symbols_should_keep_legacy_conversion_routing() -> anyhow::Result<()> {
    let cases = [
        ("1 kwh to kWh", "1 kWh = 1 kWh"),
        ("1 kpa to Pa", "1 kPa = 1,000 Pa"),
        ("1 inhg to Pa", "1 inHg = 3,386.388 Pa"),
        ("1 m2 to m²", "1 m² = 1 m²"),
        ("1 m3 to m³", "1 m³ = 1 m³"),
    ];
    let mut engine = UnitEngine::new()?;
    for (query, expected) in cases {
        let conversion =
            legacy_conversion(query).ok_or_else(|| anyhow::anyhow!("{query} should parse"))?;
        assert_eq!(
            engine.evaluate_legacy(conversion)?.result,
            expected,
            "{query}"
        );
    }
    Ok(())
}

#[test]
fn customary_system_should_match_legacy_volume_and_mpg_coefficients() -> anyhow::Result<()> {
    let cases = [
        (
            "1 floz ml",
            "1 imp fl oz = 28.413 mL",
            "1 US fl oz = 29.574 mL",
        ),
        ("1 gal l", "1 imp gal = 4.546 L", "1 US gal = 3.785 L"),
        ("1 pt ml", "1 imp pt = 568.261 mL", "1 US pt = 473.176 mL"),
        ("1 tbsp. ml", "1 tbsp. = 14.207 mL", "1 tbsp. = 14.787 mL"),
        ("1 tsp. ml", "1 tsp. = 3.552 mL", "1 tsp. = 4.929 mL"),
        ("1 mpg km/l", "1 mpg = 0.354 km/L", "1 mpg = 0.425 km/L"),
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

fn assert_explicit_ordinary_aliases(
    aliases: &[(&[&str], &str, &str)],
    customary_system: CustomarySystem,
) -> anyhow::Result<()> {
    for (alias_names, expression, symbol) in aliases {
        for alias in *alias_names {
            let unit = legacy_unit_with_customary_system(alias, customary_system)
                .ok_or_else(|| anyhow::anyhow!("alias {alias} is missing"))?;
            assert_eq!(
                (unit.expression, unit.symbol),
                (*expression, *symbol),
                "{alias}"
            );

            let source = format!("1 {alias} ml");
            let source_conversion =
                legacy_conversion_with_customary_system(&source, customary_system)
                    .ok_or_else(|| anyhow::anyhow!("source alias {alias} is missing"))?;
            assert_eq!(source_conversion.from, unit, "source alias {alias}");

            let target = format!("1 ml {alias}");
            let target_conversion =
                legacy_conversion_with_customary_system(&target, customary_system)
                    .ok_or_else(|| anyhow::anyhow!("target alias {alias} is missing"))?;
            assert_eq!(target_conversion.to, unit, "target alias {alias}");
        }
    }
    Ok(())
}

fn assert_explicit_fuel_aliases(customary_system: CustomarySystem) -> anyhow::Result<()> {
    for (alias, unit) in [
        ("uk_mpg", FuelUnit::MilesPerImperialGallon),
        ("uk_miles_per_gallon", FuelUnit::MilesPerImperialGallon),
        ("us_mpg", FuelUnit::MilesPerUsGallon),
        ("us_miles_per_gallon", FuelUnit::MilesPerUsGallon),
    ] {
        let resolved = legacy_unit_with_customary_system(alias, customary_system)
            .ok_or_else(|| anyhow::anyhow!("fuel alias {alias} is missing"))?;
        assert_eq!(
            resolved.symbol,
            if alias.starts_with("uk_") {
                "UK mpg"
            } else {
                "US mpg"
            }
        );
        assert_eq!(resolved.special, SpecialUnit::Fuel(unit), "{alias}");

        let source = format!("1 {alias} km/l");
        let source_conversion = legacy_conversion_with_customary_system(&source, customary_system)
            .ok_or_else(|| anyhow::anyhow!("source fuel alias {alias} is missing"))?;
        assert_eq!(source_conversion.from, resolved, "source alias {alias}");

        let target = format!("1 km/l {alias}");
        let target_conversion = legacy_conversion_with_customary_system(&target, customary_system)
            .ok_or_else(|| anyhow::anyhow!("target fuel alias {alias} is missing"))?;
        assert_eq!(target_conversion.to, resolved, "target alias {alias}");
    }
    Ok(())
}

#[test]
fn explicit_customary_aliases_should_resolve_independently_of_preference() -> anyhow::Result<()> {
    let ordinary_aliases = [
        (
            ["uk_gal", "uk_gallon"].as_slice(),
            "imperial_gallon",
            "UK gal",
        ),
        (["us_gal", "us_gallon"].as_slice(), "gallon", "US gal"),
        (["uk_qt", "uk_quart"].as_slice(), "imperial_quart", "UK qt"),
        (["us_qt", "us_quart"].as_slice(), "gallon / 4", "US qt"),
        (["uk_pt", "uk_pint"].as_slice(), "imperial_pint", "UK pt"),
        (["us_pt", "us_pint"].as_slice(), "pint", "US pt"),
        (["uk_gi", "uk_gill"].as_slice(), "imperial_gill", "UK gi"),
        (["us_gi", "us_gill"].as_slice(), "gallon / 32", "US gi"),
        (
            ["uk_floz", "uk_fluid_ounce"].as_slice(),
            "imperial_fluidounce",
            "UK fl oz",
        ),
        (
            ["us_floz", "us_fluid_ounce"].as_slice(),
            "fluidounce",
            "US fl oz",
        ),
        (
            ["uk_fldr", "uk_fluid_drachm"].as_slice(),
            "imperial_fluid_drachm",
            "UK fl dr",
        ),
        (
            ["us_fldr", "us_fluid_dram"].as_slice(),
            "fluidounce / 8",
            "US fl dr",
        ),
        (
            ["uk_tbsp", "uk_tablespoon"].as_slice(),
            "imperial_tablespoon",
            "UK tbsp",
        ),
        (
            ["us_tbsp", "us_tablespoon"].as_slice(),
            "tablespoon",
            "US tbsp",
        ),
        (
            ["uk_tsp", "uk_teaspoon"].as_slice(),
            "imperial_teaspoon",
            "UK tsp",
        ),
        (["us_tsp", "us_teaspoon"].as_slice(), "teaspoon", "US tsp"),
    ];
    for customary_system in [CustomarySystem::Imperial, CustomarySystem::UsCustomary] {
        assert_explicit_ordinary_aliases(&ordinary_aliases, customary_system)?;
        assert_explicit_fuel_aliases(customary_system)?;
    }
    assert!(
        legacy_conversion_with_customary_system("1 uk_cup us_cup", CustomarySystem::Imperial)
            .is_none()
    );
    Ok(())
}

#[test]
fn explicit_customary_pairs_should_render_the_same_under_both_preferences() -> anyhow::Result<()> {
    let cases = [
        (
            "1 uk_gal to us_gal",
            "1 UK gal = 1.201 US gal",
            "1 us_gal to uk_gal",
            "1 US gal = 0.833 UK gal",
        ),
        (
            "1 uk_qt to us_qt",
            "1 UK qt = 1.201 US qt",
            "1 us_qt to uk_qt",
            "1 US qt = 0.833 UK qt",
        ),
        (
            "1 uk_pt to us_pt",
            "1 UK pt = 1.201 US pt",
            "1 us_pt to uk_pt",
            "1 US pt = 0.833 UK pt",
        ),
        (
            "1 uk_gi to us_gi",
            "1 UK gi = 1.201 US gi",
            "1 us_gi to uk_gi",
            "1 US gi = 0.833 UK gi",
        ),
        (
            "1 uk_floz to us_floz",
            "1 UK fl oz = 0.961 US fl oz",
            "1 us_floz to uk_floz",
            "1 US fl oz = 1.041 UK fl oz",
        ),
        (
            "1 uk_fldr to us_fldr",
            "1 UK fl dr = 0.961 US fl dr",
            "1 us_fldr to uk_fldr",
            "1 US fl dr = 1.041 UK fl dr",
        ),
        (
            "1 uk_tbsp to us_tbsp",
            "1 UK tbsp = 0.961 US tbsp",
            "1 us_tbsp to uk_tbsp",
            "1 US tbsp = 1.041 UK tbsp",
        ),
        (
            "1 uk_tsp to us_tsp",
            "1 UK tsp = 0.721 US tsp",
            "1 us_tsp to uk_tsp",
            "1 US tsp = 1.388 UK tsp",
        ),
        (
            "1 uk_mpg to us_mpg",
            "1 UK mpg = 0.833 US mpg",
            "1 us_mpg to uk_mpg",
            "1 US mpg = 1.201 UK mpg",
        ),
    ];
    for customary_system in [CustomarySystem::Imperial, CustomarySystem::UsCustomary] {
        let mut engine = UnitEngine::new()?;
        for (forward, expected_forward, reverse, expected_reverse) in cases {
            let forward_conversion =
                legacy_conversion_with_customary_system(forward, customary_system)
                    .ok_or_else(|| anyhow::anyhow!("pair should parse: {forward}"))?;
            assert_eq!(
                engine.evaluate_legacy(forward_conversion)?.result,
                expected_forward,
                "{forward}"
            );

            let reverse_conversion =
                legacy_conversion_with_customary_system(reverse, customary_system)
                    .ok_or_else(|| anyhow::anyhow!("pair should parse: {reverse}"))?;
            assert_eq!(
                engine.evaluate_legacy(reverse_conversion)?.result,
                expected_reverse,
                "{reverse}"
            );
        }
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
