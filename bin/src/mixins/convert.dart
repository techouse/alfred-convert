import 'dart:io' show File;

import 'package:alfred_workflow/alfred_workflow.dart';
import 'package:collection/collection.dart';
import 'package:decimal/decimal.dart';
import 'package:decimal/intl.dart';
import 'package:intl/intl.dart';
import 'package:units_converter/models/double_property.dart';
import 'package:units_converter/units_converter.dart';

import '../constants/units.dart';
import '../extensions/num_helper.dart';
import '../models/currency.dart';
import '../models/exchange_rate.dart';
import '../models/exchange_rates.dart';
import '../services/ecb_exchange_rates.dart';
import '../services/emoji_downloader.dart';

class Convert {
  Convert._();

  static final NumberFormat numberFormat = NumberFormat();

  static Future<AlfredItem?> convertCurrency(
    Decimal value,
    String fromUnitSymbol,
    String toUnitSymbol,
  ) async {
    final Currency? fromCurrency =
        Currency.byNameOrNull(fromUnitSymbol.toUpperCase());

    if (fromCurrency != null) {
      final ExchangeRates? rates = await EcbExchangeRates().getLatest();

      if (rates != null) {
        final Currency? toCurrency =
            Currency.byNameOrNull(toUnitSymbol.toUpperCase());

        if (toCurrency != null) {
          final ExchangeRate rate = rates.convert(fromCurrency, toCurrency);

          final DecimalIntl convertedValue = DecimalIntl(value * rate.rate);
          final DecimalIntl invertedValue =
              DecimalIntl(value * rate.invertedRate);

          final File? image = await EmojiDownloader(
            '${toCurrency.flag.runes.map((int cp) => cp.toRadixString(16)).join('-')}.png',
          ).downloadImage();

          final Uri xeUrl = Uri.https('www.xe.com', 'currencycharts', {
            'from': fromCurrency.name,
            'to': toCurrency.name,
          });

          return AlfredItem(
            title: '${numberFormat.format(DecimalIntl(value))}'
                ' ${fromCurrency.name} ${fromCurrency.flag} ≃'
                ' ${numberFormat.format(convertedValue)}'
                ' ${toCurrency.name} ${toCurrency.flag}',
            subtitle: 'Based on ECB exchange rates from '
                    '${DateFormat.yMMMd().format(rates.date)} '
                    '${rates.date.toUtc().hour > 0 && rates.date.toUtc().minute > 0 ? "${DateFormat.Hm().format(rates.date.toUtc())} UTC" : ''}'
                .trim(),
            arg: xeUrl.toString(),
            quickLookUrl: xeUrl.toString(),
            icon: AlfredItemIcon(
              path: image != null ? image.absolute.path : 'icon.png',
            ),
            valid: true,
            mods: {
              {AlfredItemModKey.alt}: AlfredItemMod(
                subtitle: '${numberFormat.format(DecimalIntl(value))} '
                    '${toCurrency.name} ${toCurrency.flag} ≃'
                    ' ${numberFormat.format(invertedValue)}'
                    ' ${fromCurrency.name} ${fromCurrency.flag}',
                valid: true,
              ),
              {AlfredItemModKey.cmd}: AlfredItemMod(
                subtitle: 'Copy ${numberFormat.format(convertedValue)}'
                    ' ${toCurrency.name} ${toCurrency.flag} to clipboard',
                arg: '${numberFormat.format(convertedValue)}'
                    ' ${toCurrency.name}',
                valid: true,
              ),
            },
          );
        } else {
          return _invalidFormat(
            'Can not convert ${fromCurrency.fullName} to "$toUnitSymbol"',
          );
        }
      }
    }

    return null;
  }

  static Future<AlfredItem> convertUnit(
    num value,
    String fromUnitSymbol,
    String toUnitSymbol,
  ) async {
    final Enum? from = _identifyUnit(fromUnitSymbol);
    final Enum? to = _identifyUnit(toUnitSymbol);

    if (from != null && to != null) {
      final DoubleProperty? fromProperty = getProperty(from)
        ?..convert(from, value.toDouble());
      final Unit? fromUnit = fromProperty?.getUnit(from);

      try {
        final String? emoji = propertyEmojis[from.runtimeType];
        final File? image = emoji != null
            ? await EmojiDownloader(
                '${emoji.runes.first.toRadixString(16)}.png',
              ).downloadImage()
            : null;

        final Unit? toUnit = fromProperty?.getUnit(to);

        final Decimal? convertedQuantity = toUnit?.value?.toDecimal();
        final DecimalIntl? convertedQuantityIntl =
            convertedQuantity != null ? DecimalIntl(convertedQuantity) : null;

        final Decimal? singleConvertedQuantity =
            1.convertFromTo(from, to)?.toDecimal();
        final DecimalIntl? singleConvertedQuantityIntl =
            singleConvertedQuantity != null
                ? DecimalIntl(singleConvertedQuantity)
                : null;

        final Uri wolframAlphaUrl = Uri.https('www.wolframalpha.com', 'input', {
          'i': '$value ${fromUnit?.symbol ?? fromUnitSymbol} to '
              '${toUnit?.symbol ?? toUnitSymbol}',
        });

        return AlfredItem(
          title:
              '${numberFormat.format(DecimalIntl(value.toDecimal()))} ${fromUnit?.symbol} ='
              ' ${numberFormat.format(convertedQuantityIntl)}'
              ' ${toUnit?.symbol}',
          subtitle: 'Based on the fact that 1 ${fromUnit?.symbol} ='
              ' ${numberFormat.format(singleConvertedQuantityIntl)}'
              ' ${toUnit?.symbol}',
          arg: wolframAlphaUrl.toString(),
          quickLookUrl: wolframAlphaUrl.toString(),
          icon: AlfredItemIcon(
            path: image != null ? image.absolute.path : 'icon.png',
          ),
          valid: true,
        );
      } on TypeError {
        return _invalidFormat(
          'Can not convert ${fromUnit?.symbol ?? fromUnitSymbol} to "$toUnitSymbol"',
        );
      }
    }

    return _invalidFormat();
  }

  static Enum? _identifyUnit(String unitSymbol) => properties.firstWhereOrNull(
        (Map<String, Enum> property) => property.containsKey(unitSymbol),
      )?[unitSymbol];

  static DoubleProperty? getProperty(Enum unit) {
    if (unit is ANGLE) return Angle();
    if (unit is AREA) return Area();
    if (unit is DIGITAL_DATA) return DigitalData();
    if (unit is ENERGY) return Energy();
    if (unit is FORCE) return Force();
    if (unit is FUEL_CONSUMPTION) return FuelConsumption();
    if (unit is LENGTH) return Length();
    if (unit is MASS) return Mass();
    if (unit is POWER) return Power();
    if (unit is PRESSURE) return Pressure();
    if (unit is SPEED) return Speed();
    if (unit is TEMPERATURE) return Temperature();
    if (unit is TIME) return Time();
    if (unit is TORQUE) return Torque();
    if (unit is VOLUME) return Volume();

    return null;
  }

  static AlfredItem _invalidFormat([String? message]) => AlfredItem(
        title: message ?? 'Invalid format.',
        subtitle: 'Usage: conv 123.45 gbp usd',
        icon: AlfredItemIcon(path: 'icon.png'),
        valid: false,
      );
}
