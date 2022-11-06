import 'package:emojis/emojis.dart';
import 'package:units_converter/units_converter.dart';

const Set<Map<String, Enum>> properties = {
  _angle,
  _area,
  _digitalData,
  _energy,
  _force,
  _fuelConsumption,
  _length,
  _mass,
  _numeralSystems,
  _power,
  _pressure,
  _speed,
  _temperature,
  _time,
  _torque,
  _volume,
};

const Map<Type, String> propertyEmojis = {
  ANGLE: Emojis.triangularRuler,
  AREA: Emojis.house,
  DIGITAL_DATA: Emojis.computerDisk,
  ENERGY: Emojis.sun,
  FORCE: Emojis.elephant,
  FUEL_CONSUMPTION: Emojis.fuelPump,
  LENGTH: Emojis.straightRuler,
  MASS: Emojis.balanceScale,
  NUMERAL_SYSTEMS: Emojis.inputNumbers,
  POWER: Emojis.highVoltage,
  PRESSURE: Emojis.fireExtinguisher,
  SPEED: Emojis.racingCar,
  TEMPERATURE: Emojis.thermometer,
  TIME: Emojis.stopwatch,
  TORQUE: Emojis.locomotive,
  VOLUME: Emojis.testTube,
};

const Map<String, ANGLE> _angle = {
  '°': ANGLE.degree,
  'deg': ANGLE.degree,
  "'": ANGLE.minutes,
  "''": ANGLE.seconds,
  '"': ANGLE.seconds,
  'rad': ANGLE.radians,
};

const Map<String, AREA> _area = {
  'm2': AREA.squareMeters,
  'cm2': AREA.squareCentimeters,
  'in2': AREA.squareInches,
  'ft2': AREA.squareFeet,
  'mi2': AREA.squareMiles,
  'yd2': AREA.squareYard,
  'mm2': AREA.squareMillimeters,
  'km2': AREA.squareKilometers,
  'ha': AREA.hectares,
  'ac': AREA.acres,
  'a': AREA.are,
};

const Map<String, DIGITAL_DATA> _digitalData = {
  'b': DIGITAL_DATA.bit,
  'kb': DIGITAL_DATA.kilobit,
  'Mb': DIGITAL_DATA.megabit,
  'Gb': DIGITAL_DATA.gigabit,
  'Tb': DIGITAL_DATA.terabit,
  'Pb': DIGITAL_DATA.petabit,
  'Eb': DIGITAL_DATA.exabit,
  'Kibit': DIGITAL_DATA.kibibit,
  'Mibit': DIGITAL_DATA.mebibit,
  'Gibit': DIGITAL_DATA.gibibit,
  'Tibit': DIGITAL_DATA.tebibit,
  'Pibit': DIGITAL_DATA.pebibit,
  'Eibit': DIGITAL_DATA.exbibit,
  'B': DIGITAL_DATA.byte,
  'kB': DIGITAL_DATA.kilobyte,
  'MB': DIGITAL_DATA.megabyte,
  'GB': DIGITAL_DATA.gigabyte,
  'TB': DIGITAL_DATA.terabyte,
  'PB': DIGITAL_DATA.petabyte,
  'EB': DIGITAL_DATA.exabyte,
  'KiB': DIGITAL_DATA.kibibyte,
  'MiB': DIGITAL_DATA.mebibyte,
  'GiB': DIGITAL_DATA.gibibyte,
  'TiB': DIGITAL_DATA.tebibyte,
  'PiB': DIGITAL_DATA.pebibyte,
  'EiB': DIGITAL_DATA.exbibyte,
};

const Map<String, ENERGY> _energy = {
  'J': ENERGY.joules,
  'j': ENERGY.joules,
  'cal': ENERGY.calories,
  'kcal': ENERGY.kilocalories,
  'kwh': ENERGY.kilowattHours,
  'eV': ENERGY.electronvolts,
  'ev': ENERGY.electronvolts,
  'ft⋅lbf': ENERGY.energyFootPound,
  'ftlbf': ENERGY.energyFootPound,
};

const Map<String, FORCE> _force = {
  'N': FORCE.newton,
  'n': FORCE.newton,
  'dyn': FORCE.dyne,
  'lbf': FORCE.poundForce,
  'kgf': FORCE.kilogramForce,
  'pdl': FORCE.poundal,
};

const Map<String, FUEL_CONSUMPTION> _fuelConsumption = {
  'km/l': FUEL_CONSUMPTION.kilometersPerLiter,
  'l/100km': FUEL_CONSUMPTION.litersPer100km,
  'us.mpg': FUEL_CONSUMPTION.milesPerUsGallon,
  'mpg': FUEL_CONSUMPTION.milesPerImperialGallon,
};

const Map<String, LENGTH> _length = {
  'm': LENGTH.meters,
  'cm': LENGTH.centimeters,
  'in': LENGTH.inches,
  'ft': LENGTH.feet,
  'M': LENGTH.nauticalMiles,
  'yd': LENGTH.yards,
  'mi': LENGTH.miles,
  'mm': LENGTH.millimeters,
  'µm': LENGTH.micrometers,
  'nm': LENGTH.nanometers,
  'Å': LENGTH.angstroms,
  'å': LENGTH.angstroms,
  'pm': LENGTH.picometers,
  'km': LENGTH.kilometers,
  'au': LENGTH.astronomicalUnits,
  'ly': LENGTH.lightYears,
  'pc': LENGTH.parsec,
};

const Map<String, MASS> _mass = {
  'g': MASS.grams,
  'hg': MASS.ettograms,
  'kg': MASS.kilograms,
  'lb': MASS.pounds,
  'oz': MASS.ounces,
  't': MASS.tons,
  'mg': MASS.milligrams,
  'u': MASS.uma,
  'ct': MASS.carats,
  'cg': MASS.centigrams,
  'dwt': MASS.pennyweights,
  'ozt': MASS.troyOunces,
  'st': MASS.stones,
};

const Map<String, NUMERAL_SYSTEMS> _numeralSystems = {
  'dec': NUMERAL_SYSTEMS.decimal,
  'decimal': NUMERAL_SYSTEMS.decimal,
  'hex': NUMERAL_SYSTEMS.hexadecimal,
  'hexadecimal': NUMERAL_SYSTEMS.hexadecimal,
  'oct': NUMERAL_SYSTEMS.octal,
  'octal': NUMERAL_SYSTEMS.octal,
  'bin': NUMERAL_SYSTEMS.binary,
  'binary': NUMERAL_SYSTEMS.binary,
};

const Map<String, POWER> _power = {
  'W': POWER.watt,
  'mW': POWER.milliwatt,
  'kW': POWER.kilowatt,
  'MW': POWER.megawatt,
  'GW': POWER.gigawatt,
  'eu.hp': POWER.europeanHorsePower,
  'hp': POWER.imperialHorsePower,
};

const Map<String, PRESSURE> _pressure = {
  'pa': PRESSURE.pascal,
  'atm': PRESSURE.atmosphere,
  'bar': PRESSURE.bar,
  'mbar': PRESSURE.millibar,
  'psi': PRESSURE.psi,
  'mmhg': PRESSURE.torr,
  'torr': PRESSURE.torr,
  'hpa': PRESSURE.hectoPascal,
  'inhg': PRESSURE.inchOfMercury,
};

const Map<String, SPEED> _speed = {
  'm/s': SPEED.metersPerSecond,
  'km/h': SPEED.kilometersPerHour,
  'kph': SPEED.kilometersPerHour,
  'mi/h': SPEED.milesPerHour,
  'mph': SPEED.milesPerHour,
  'kts': SPEED.knots,
  'ft/s': SPEED.feetsPerSecond,
  'min/km': SPEED.minutesPerKilometer,
};

const Map<String, TEMPERATURE> _temperature = {
  '°F': TEMPERATURE.fahrenheit,
  'F': TEMPERATURE.fahrenheit,
  '°C': TEMPERATURE.celsius,
  'C': TEMPERATURE.celsius,
  'K': TEMPERATURE.kelvin,
  '°Re': TEMPERATURE.reamur,
  'Re': TEMPERATURE.reamur,
  '°Rø': TEMPERATURE.romer,
  'Rø': TEMPERATURE.romer,
  '°De': TEMPERATURE.delisle,
  'De': TEMPERATURE.delisle,
  '°R': TEMPERATURE.rankine,
  'R': TEMPERATURE.rankine,
};

const Map<String, TIME> _time = {
  's': TIME.seconds,
  'ds': TIME.deciseconds,
  'cs': TIME.centiseconds,
  'ms': TIME.milliseconds,
  'µs': TIME.microseconds,
  'ns': TIME.nanoseconds,
  'min': TIME.minutes,
  'h': TIME.hours,
  'd': TIME.days,
  'a': TIME.years365,
  'c.': TIME.centuries,
};

const Map<String, TORQUE> _torque = {
  'N·m': TORQUE.newtonMeter,
  'Nm': TORQUE.newtonMeter,
  'dyn·m': TORQUE.dyneMeter,
  'dynm': TORQUE.dyneMeter,
  'lbf·ft': TORQUE.poundForceFeet,
  'lbfft': TORQUE.poundForceFeet,
  'kgf·m': TORQUE.kilogramForceMeter,
  'kgfm': TORQUE.kilogramForceMeter,
  'pdl·m': TORQUE.poundalMeter,
  'pdlm': TORQUE.poundalMeter,
};

const Map<String, VOLUME> _volume = {
  'm3': VOLUME.cubicMeters,
  'l': VOLUME.liters,
  'L': VOLUME.liters,
  'gal': VOLUME.imperialGallons,
  'us.gal': VOLUME.usGallons,
  'pt': VOLUME.imperialPints,
  'us.pt': VOLUME.usPints,
  'ml': VOLUME.milliliters,
  'tbsp.': VOLUME.tablespoonsUs,
  'cup': VOLUME.cups,
  'cm3': VOLUME.cubicCentimeters,
  'ft3': VOLUME.cubicFeet,
  'in3': VOLUME.cubicInches,
  'mm3': VOLUME.cubicMillimeters,
  'fl.oz': VOLUME.imperialFluidOunces,
  'floz': VOLUME.imperialFluidOunces,
  'us.fl.oz': VOLUME.usFluidOunces,
  'us.floz': VOLUME.usFluidOunces,
};
