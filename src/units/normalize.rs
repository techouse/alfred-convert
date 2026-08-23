use crate::format::parse_decimal;

/// One historical alias and the Numbat identifier used to evaluate it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LegacyUnit {
    pub expression: &'static str,
    pub symbol: &'static str,
    pub dimension: &'static str,
    pub emoji: &'static str,
    pub special: SpecialUnit,
}

/// Legacy conversions that need an affine or reciprocal Numbat function.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SpecialUnit {
    None,
    Fuel(FuelUnit),
    Pace,
    Temperature(TemperatureUnit),
}

/// Historical fuel-consumption representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FuelUnit {
    KilometersPerLiter,
    LitersPer100Kilometers,
    MilesPerUsGallon,
    MilesPerImperialGallon,
}

/// Historical temperature representation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TemperatureUnit {
    Fahrenheit,
    Celsius,
    Kelvin,
    Reamur,
    Romer,
    Delisle,
    Rankine,
}

/// A parsed historical Alfred conversion.
#[derive(Clone, Copy, Debug)]
pub struct LegacyConversion<'a> {
    pub amount: &'a str,
    pub from: LegacyUnit,
    pub to: LegacyUnit,
}

/// Recognizes only the Dart workflow's exact three/four-token shorthand.
#[must_use]
pub fn legacy_conversion(query: &str) -> Option<LegacyConversion<'_>> {
    let parts = query.split(' ').collect::<Vec<_>>();
    let (amount, from, to) = match parts.as_slice() {
        [amount, from, to] => (*amount, *from, *to),
        [amount, from, separator, to] if separator.eq_ignore_ascii_case("to") => {
            (*amount, *from, *to)
        }
        _ => return None,
    };
    parse_decimal(amount)?;
    Some(LegacyConversion {
        amount,
        from: legacy_unit(from)?,
        to: legacy_unit(to)?,
    })
}

/// Finds metadata for a historical physical-unit alias.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn legacy_unit(alias: &str) -> Option<LegacyUnit> {
    let ordinary = |expression, symbol, dimension, emoji| LegacyUnit {
        expression,
        symbol,
        dimension,
        emoji,
        special: SpecialUnit::None,
    };
    let fuel = |symbol, unit| LegacyUnit {
        expression: "",
        symbol,
        dimension: "Fuel consumption",
        emoji: "⛽",
        special: SpecialUnit::Fuel(unit),
    };
    let pace = |expression, symbol| LegacyUnit {
        expression,
        symbol,
        dimension: "Speed",
        emoji: "🏎️",
        special: SpecialUnit::Pace,
    };
    let temperature = |expression, symbol, unit| LegacyUnit {
        expression,
        symbol,
        dimension: "Temperature",
        emoji: "🌡️",
        special: SpecialUnit::Temperature(unit),
    };

    Some(match alias {
        "°" | "deg" => ordinary("degree", "°", "Angle", "📐"),
        "'" => ordinary("arcminute", "'", "Angle", "📐"),
        "''" | "\"" => ordinary("arcsecond", "''", "Angle", "📐"),
        "rad" => ordinary("legacy_radian", "rad", "Angle", "📐"),

        "m2" => ordinary("m^2", "m²", "Area", "🏠"),
        "cm2" => ordinary("cm^2", "cm²", "Area", "🏠"),
        "in2" => ordinary("in^2", "in²", "Area", "🏠"),
        "ft2" => ordinary("ft^2", "ft²", "Area", "🏠"),
        "mi2" => ordinary("mi^2", "mi²", "Area", "🏠"),
        "yd2" => ordinary("yd^2", "yd²", "Area", "🏠"),
        "mm2" => ordinary("mm^2", "mm²", "Area", "🏠"),
        "km2" => ordinary("km^2", "km²", "Area", "🏠"),
        "ha" => ordinary("hectare", "ha", "Area", "🏠"),
        "ac" => ordinary("acre", "ac", "Area", "🏠"),
        "a" => ordinary("are", "a", "Area", "🏠"),

        "b" => ordinary("bit", "b", "Digital information", "💾"),
        "kb" => ordinary("kilobit", "kb", "Digital information", "💾"),
        "Mb" => ordinary("megabit", "Mb", "Digital information", "💾"),
        "Gb" => ordinary("gigabit", "Gb", "Digital information", "💾"),
        "Tb" => ordinary("terabit", "Tb", "Digital information", "💾"),
        "Pb" => ordinary("petabit", "Pb", "Digital information", "💾"),
        "Eb" => ordinary("exabit", "Eb", "Digital information", "💾"),
        "Kibit" => ordinary("kibibit", "Kibit", "Digital information", "💾"),
        "Mibit" => ordinary("mebibit", "Mibit", "Digital information", "💾"),
        "Gibit" => ordinary("gibibit", "Gibit", "Digital information", "💾"),
        "Tibit" => ordinary("tebibit", "Tibit", "Digital information", "💾"),
        "Pibit" => ordinary("pebibit", "Pibit", "Digital information", "💾"),
        "Eibit" => ordinary("exbibit", "Eibit", "Digital information", "💾"),
        "B" => ordinary("byte", "B", "Digital information", "💾"),
        "kB" => ordinary("kilobyte", "kB", "Digital information", "💾"),
        "MB" => ordinary("megabyte", "MB", "Digital information", "💾"),
        "GB" => ordinary("gigabyte", "GB", "Digital information", "💾"),
        "TB" => ordinary("terabyte", "TB", "Digital information", "💾"),
        "PB" => ordinary("petabyte", "PB", "Digital information", "💾"),
        "EB" => ordinary("exabyte", "EB", "Digital information", "💾"),
        "KiB" => ordinary("kibibyte", "KiB", "Digital information", "💾"),
        "MiB" => ordinary("mebibyte", "MiB", "Digital information", "💾"),
        "GiB" => ordinary("gibibyte", "GiB", "Digital information", "💾"),
        "TiB" => ordinary("tebibyte", "TiB", "Digital information", "💾"),
        "PiB" => ordinary("pebibyte", "PiB", "Digital information", "💾"),
        "EiB" => ordinary("exbibyte", "EiB", "Digital information", "💾"),

        "J" | "j" => ordinary("J", "J", "Energy", "☀️"),
        "kJ" => ordinary("kJ", "kJ", "Energy", "☀️"),
        "cal" => ordinary("legacy_calorie", "cal", "Energy", "☀️"),
        "kcal" => ordinary("legacy_kilocalorie", "kcal", "Energy", "☀️"),
        "kwh" => ordinary("kWh", "kWh", "Energy", "☀️"),
        "eV" | "ev" => ordinary("legacy_electronvolt", "eV", "Energy", "☀️"),
        "ft⋅lbf" | "ftlbf" => ordinary("legacy_energy_foot_pound", "ft⋅lbf", "Energy", "☀️"),
        "Wh" => ordinary("Wh", "Wh", "Energy", "☀️"),
        "BTU" => ordinary("legacy_btu", "BTU", "Energy", "☀️"),

        "N" | "n" => ordinary("N", "N", "Force", "🐘"),
        "dyn" => ordinary("dyne", "dyn", "Force", "🐘"),
        "lbf" => ordinary("legacy_pound_force", "lbf", "Force", "🐘"),
        "kgf" => ordinary("kilogram_force", "kgf", "Force", "🐘"),
        "pdl" => ordinary("legacy_poundal", "pdl", "Force", "🐘"),

        "km/l" => fuel("km/l", FuelUnit::KilometersPerLiter),
        "l/100km" => fuel("l/100km", FuelUnit::LitersPer100Kilometers),
        "us.mpg" => fuel("mpg", FuelUnit::MilesPerUsGallon),
        "mpg" => fuel("mpg", FuelUnit::MilesPerImperialGallon),

        "lx" => ordinary("lux", "lx", "Illuminance", "💡"),
        "fc" => ordinary("footcandle", "fc", "Illuminance", "💡"),

        "m" => ordinary("m", "m", "Length", "📏"),
        "cm" => ordinary("cm", "cm", "Length", "📏"),
        "in" => ordinary("in", "in", "Length", "📏"),
        "ft" => ordinary("ft", "ft", "Length", "📏"),
        "M" => ordinary("nautical_mile", "M", "Length", "📏"),
        "yd" => ordinary("yd", "yd", "Length", "📏"),
        "mi" => ordinary("mi", "mi", "Length", "📏"),
        "mm" => ordinary("mm", "mm", "Length", "📏"),
        "µm" => ordinary("micrometer", "µm", "Length", "📏"),
        "nm" => ordinary("nm", "nm", "Length", "📏"),
        "Å" | "å" => ordinary("angstrom", "Å", "Length", "📏"),
        "pm" => ordinary("pm", "pm", "Length", "📏"),
        "km" => ordinary("km", "km", "Length", "📏"),
        "au" => ordinary("astronomicalunit", "au", "Length", "📏"),
        "ly" => ordinary("legacy_lightyear", "ly", "Length", "📏"),
        "pc" => ordinary("legacy_parsec", "pc", "Length", "📏"),
        "th" => ordinary("thou", "th", "Length", "📏"),

        "g" => ordinary("g", "g", "Mass", "⚖️"),
        "hg" => ordinary("hectogram", "hg", "Mass", "⚖️"),
        "kg" => ordinary("kg", "kg", "Mass", "⚖️"),
        "lb" => ordinary("lb", "lb", "Mass", "⚖️"),
        "oz" => ordinary("oz", "oz", "Mass", "⚖️"),
        "t" => ordinary("tonne", "t", "Mass", "⚖️"),
        "mg" => ordinary("mg", "mg", "Mass", "⚖️"),
        "u" => ordinary("legacy_atomic_mass_unit", "u", "Mass", "⚖️"),
        "ct" => ordinary("legacy_carat", "ct", "Mass", "⚖️"),
        "cg" => ordinary("cg", "cg", "Mass", "⚖️"),
        "dwt" => ordinary("pennyweight", "dwt", "Mass", "⚖️"),
        "ozt" => ordinary("troy_ounce", "oz t", "Mass", "⚖️"),
        "st" => ordinary("stone", "st.", "Mass", "⚖️"),

        "W" => ordinary("W", "W", "Power", "⚡"),
        "mW" => ordinary("mW", "mW", "Power", "⚡"),
        "kW" => ordinary("kW", "kW", "Power", "⚡"),
        "MW" => ordinary("MW", "MW", "Power", "⚡"),
        "GW" => ordinary("GW", "GW", "Power", "⚡"),
        "eu.hp" => ordinary("horsepower", "hp(M)", "Power", "⚡"),
        "hp" => ordinary("legacy_imperial_horsepower", "hp(I)", "Power", "⚡"),

        "pa" => ordinary("Pa", "Pa", "Pressure", "🧯"),
        "atm" => ordinary("atm", "atm", "Pressure", "🧯"),
        "bar" => ordinary("bar", "bar", "Pressure", "🧯"),
        "mbar" => ordinary("mbar", "mbar", "Pressure", "🧯"),
        "psi" => ordinary("legacy_psi", "psi", "Pressure", "🧯"),
        "mmhg" | "torr" => ordinary("legacy_torr", "torr", "Pressure", "🧯"),
        "kpa" => ordinary("kPa", "kPa", "Pressure", "🧯"),
        "hpa" => ordinary("hPa", "hPa", "Pressure", "🧯"),
        "inhg" => ordinary("legacy_inch_of_mercury", "inHg", "Pressure", "🧯"),
        "ksi" => ordinary("legacy_ksi", "ksi", "Pressure", "🧯"),
        "MPa" => ordinary("MPa", "MPa", "Pressure", "🧯"),
        "GPa" => ordinary("GPa", "GPa", "Pressure", "🧯"),

        "m/s" => ordinary("m/s", "m/s", "Speed", "🏎️"),
        "km/h" | "kph" => ordinary("km/h", "km/h", "Speed", "🏎️"),
        "mi/h" | "mph" => ordinary("mi/h", "mi/h", "Speed", "🏎️"),
        "kts" => ordinary("knot", "kts", "Speed", "🏎️"),
        "ft/s" => ordinary("ft/s", "ft/s", "Speed", "🏎️"),
        "min/km" => pace("min/km", "min/km"),
        "min/mi" => pace("min/mi", "min/mi"),
        "c" => ordinary("speed_of_light", "c", "Speed", "🏎️"),

        "°F" | "F" => temperature("degF", "°F", TemperatureUnit::Fahrenheit),
        "°C" | "C" => temperature("degC", "°C", TemperatureUnit::Celsius),
        "K" => temperature("K", "K", TemperatureUnit::Kelvin),
        "°Re" | "Re" => temperature("", "°Re", TemperatureUnit::Reamur),
        "°Rø" | "Rø" => temperature("", "°Rø", TemperatureUnit::Romer),
        "°De" | "De" => temperature("", "°De", TemperatureUnit::Delisle),
        "°R" | "R" => temperature("degR", "°R", TemperatureUnit::Rankine),

        "s" => ordinary("s", "s", "Time", "⏱️"),
        "ds" => ordinary("decisecond", "ds", "Time", "⏱️"),
        "cs" => ordinary("centisecond", "cs", "Time", "⏱️"),
        "ms" => ordinary("ms", "ms", "Time", "⏱️"),
        "µs" => ordinary("microsecond", "µs", "Time", "⏱️"),
        "ns" => ordinary("ns", "ns", "Time", "⏱️"),
        "min" => ordinary("min", "min", "Time", "⏱️"),
        "h" => ordinary("h", "h", "Time", "⏱️"),
        "d" => ordinary("day", "d", "Time", "⏱️"),
        "c." => ordinary("100 legacy_year365", "c.", "Time", "⏱️"),

        "N·m" | "Nm" => ordinary("N * m", "N·m", "Torque", "🚂"),
        "dyn·m" | "dynm" => ordinary("dyne * m", "dyn·m", "Torque", "🚂"),
        "lbf·ft" | "lbfft" => ordinary("legacy_torque_pound_force_foot", "lbf·ft", "Torque", "🚂"),
        "kgf·m" | "kgfm" => ordinary("legacy_kilogram_force_meter", "kgf·m", "Torque", "🚂"),
        "pdl·m" | "pdlm" => ordinary("legacy_poundal * m", "pdl·m", "Torque", "🚂"),
        "lbf·in" => ordinary("legacy_torque_pound_force_inch", "lbf·in", "Torque", "🚂"),

        "m3" => ordinary("m^3", "m³", "Volume", "🧪"),
        "l" | "L" => ordinary("L", "l", "Volume", "🧪"),
        "gal" => ordinary("imperial_gallon", "imp gal", "Volume", "🧪"),
        "us.gal" => ordinary("gallon", "US gal", "Volume", "🧪"),
        "pt" => ordinary("imperial_pint", "imp pt", "Volume", "🧪"),
        "us.pt" => ordinary("pint", "US pt", "Volume", "🧪"),
        "ml" => ordinary("mL", "ml", "Volume", "🧪"),
        "tbsp." => ordinary("tablespoon", "tbsp.", "Volume", "🧪"),
        "cup" => ordinary("cup", "cup", "Volume", "🧪"),
        "cm3" => ordinary("cm^3", "cm³", "Volume", "🧪"),
        "ft3" => ordinary("ft^3", "ft³", "Volume", "🧪"),
        "in3" => ordinary("in^3", "in³", "Volume", "🧪"),
        "mm3" => ordinary("mm^3", "mm³", "Volume", "🧪"),
        "fl.oz" | "floz" => ordinary("imperial_fluidounce", "imp fl oz", "Volume", "🧪"),
        "us.fl.oz" | "us.floz" => ordinary("fluidounce", "US fl oz", "Volume", "🧪"),
        "US. liq. gi" => ordinary("gallon / 32", "US. liq. gi", "Volume", "🧪"),
        "US. liq. qt" => ordinary("gallon / 4", "US. liq. qt", "Volume", "🧪"),
        "fl" => ordinary("femtoliter", "fl", "Volume", "🧪"),
        "pl" => ordinary("picoliter", "pl", "Volume", "🧪"),
        "nl" => ordinary("nanoliter", "nl", "Volume", "🧪"),
        "µl" => ordinary("microliter", "µl", "Volume", "🧪"),
        "dl" => ordinary("deciliter", "dl", "Volume", "🧪"),
        "cl" => ordinary("centiliter", "cl", "Volume", "🧪"),
        "tsp." => ordinary("legacy_metric_teaspoon", "tsp.", "Volume", "🧪"),
        _ => return None,
    })
}
