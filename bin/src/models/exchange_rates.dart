import 'package:autoequal/autoequal.dart';
import 'package:collection/collection.dart';
import 'package:decimal/decimal.dart';
import 'package:equatable/equatable.dart' show EquatableMixin;
import 'package:json_annotation/json_annotation.dart';
import 'package:timezone/data/latest.dart' as tz;
import 'package:timezone/timezone.dart' as tz;
import 'package:xml/xml.dart';

import '../converters/utc_date_time_converter.dart';
import 'currency.dart';
import 'exchange_rate.dart';

part 'exchange_rates.g.dart';

@autoequalMixin
@JsonSerializable(explicitToJson: true)
@UtcDateTimeConverter.instance
class ExchangeRates with EquatableMixin, _$ExchangeRatesAutoequalMixin {
  const ExchangeRates({
    required this.date,
    required this.rates,
  });

  final DateTime date;

  @JsonKey(fromJson: _itemsFromJson)
  final List<ExchangeRate> rates;

  ExchangeRate convert(Currency from, Currency to) {
    final ExchangeRate? fromRate =
        rates.firstWhereOrNull((rate) => rate.currency == from);
    if (fromRate == null) {
      throw ArgumentError.value(from, 'from', 'Invalid from currency pair.');
    }

    final ExchangeRate? toRate =
        rates.firstWhereOrNull((rate) => rate.currency == to);
    if (toRate == null) {
      throw ArgumentError.value(to, 'to', 'Invalid to currency pair.');
    }

    return ExchangeRate(
      currency: to,
      rate: (toRate.rate / fromRate.rate).toDecimal(),
    );
  }

  factory ExchangeRates.fromXml(XmlDocument doc, [DateTime? updatedAt]) {
    if (updatedAt == null) {
      tz.initializeTimeZones();
      final tz.Location brusselsTimeZone = tz.getLocation('Europe/Brussels');

      final DateTime now = DateTime.now().toUtc();

      final String? time = doc.descendantElements
          .firstWhereOrNull((XmlElement el) => el.getAttribute('time') != null)
          ?.getAttribute('time');

      updatedAt = time != null
          ? tz.TZDateTime.from(
              DateTime.parse('${time}T00:00:00Z'),
              brusselsTimeZone,
            )
          : tz.TZDateTime.from(
              DateTime.utc(now.year, now.month, now.day),
              brusselsTimeZone,
            );
    }

    return ExchangeRates(
      date: updatedAt,
      rates: [
        ExchangeRate(currency: Currency.EUR, rate: Decimal.one),
        ...doc.descendantElements.where((XmlElement el) {
          if (el.getAttribute('currency') != null &&
              el.getAttribute('rate') != null) {
            try {
              Currency.values.byName(el.getAttribute('currency')!);

              return true;
            } catch (_) {}
          }

          return false;
        }).map(
          (XmlElement el) => ExchangeRate(
            currency: Currency.values.byName(el.getAttribute('currency')!),
            rate: Decimal.parse(el.getAttribute('rate')!),
          ),
        ),
      ],
    );
  }

  static List<ExchangeRate> _itemsFromJson(List<dynamic> items) => items
      .map((e) => ExchangeRate.fromJson(Map<String, dynamic>.from(e)))
      .toList();

  factory ExchangeRates.fromJson(Map<String, dynamic> json) =>
      _$ExchangeRatesFromJson(json);

  Map<String, dynamic> toJson() => _$ExchangeRatesToJson(this);
}
