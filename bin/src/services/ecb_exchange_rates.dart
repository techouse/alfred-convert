import 'dart:developer' as developer show log;
import 'dart:io';

import 'package:alfred_workflow/alfred_workflow.dart' show AlfredCache;
import 'package:http/http.dart' as http show get, Response;
import 'package:stash/stash_api.dart'
    show Cache, CreatedExpiryPolicy, ExpiryPolicy, FifoEvictionPolicy;
import 'package:timezone/data/latest.dart' as tz;
import 'package:timezone/timezone.dart' as tz;
import 'package:xml/xml.dart';

import '../mixins/rfc_822.dart';
import '../models/exchange_rates.dart';

class EcbExchangeRates with Rfc822 {
  /// https://www.ecb.europa.eu/services/contacts/working-hours/html/index.en.html
  static const List<String> _holidays = [
    '01-01', // New Year's Day
    '05-01', // Labour Day
    '12-25', // Christmas Day
    '12-26', // Christmas Holiday
  ];

  static const List<String> _easters = [
    '2022-04-17',
    '2023-04-09',
    '2024-03-31',
    '2025-04-20',
    '2026-04-05',
    '2027-03-29',
    '2028-04-16',
    '2029-04-01',
    '2030-04-21',
  ];

  final DateTime _now = DateTime.now().toUtc();
  late final DateTime _today = DateTime.utc(
    _now.year,
    _now.month,
    _now.day,
  );

  late final AlfredCache<ExchangeRates> _cache = AlfredCache<ExchangeRates>(
    fromEncodable: (Map<String, dynamic> json) => ExchangeRates.fromJson(json),
    maxEntries: 1,
    name: 'exchange_rates_cache',
    expiryPolicy: const CreatedExpiryPolicy(ExpiryPolicy.eternal),
    evictionPolicy: const FifoEvictionPolicy(),
  );

  Future<ExchangeRates?> getLatest() async {
    final Cache<ExchangeRates> cache = await _cache.cache;
    final ExchangeRates? cached = await cache.get('latest');

    if (cached != null) {
      final DateTime cachedDate = DateTime.utc(
        cached.date.year,
        cached.date.month,
        cached.date.day,
      );

      if (cachedDate.difference(_today).inDays >= 0) {
        return cached;
      } else {
        if (_today.difference(cachedDate).inDays > 2 || await _shouldUpdate()) {
          final ExchangeRates? latest = await _downloadLatest();
          if (latest != null) {
            await cache.put('latest', latest);

            return latest;
          }
        }
      }
    } else {
      final ExchangeRates? latest = await _downloadLatest();

      if (latest != null) {
        await cache.put('latest', latest);

        return latest;
      }
    }

    return cached;
  }

  /// The reference rates are usually updated around 16:00 CET on every working day, except on TARGET closing days.
  Future<bool> _shouldUpdate() async {
    tz.initializeTimeZones();

    final tz.Location brusselsTimeZone = tz.getLocation('Europe/Brussels');
    late final DateTime brusselsNow =
        tz.TZDateTime.from(_now, brusselsTimeZone);

    return brusselsNow.weekday != DateTime.saturday &&
        brusselsNow.weekday != DateTime.sunday &&

        /// check that it's not a public holiday
        !_holidays.any((String date) {
          final DateTime holiday = tz.TZDateTime.from(
            DateTime.parse('${brusselsNow.year}-${date}T00:00:00Z'),
            brusselsTimeZone,
          );

          return brusselsNow.year == holiday.year &&
              brusselsNow.month == holiday.month &&
              brusselsNow.day == holiday.day;
        }) &&

        /// check that it's not Good Friday or Easter Monday
        !_easters
            .where((String easter) =>
                tz.TZDateTime.from(
                  DateTime.parse('${easter}T00:00:00Z'),
                  brusselsTimeZone,
                ).year >=
                brusselsNow.year)
            .any((String easter) => <DateTime>[
                  tz.TZDateTime.from(
                    // Good Friday
                    DateTime.parse('${easter}T00:00:00Z').subtract(
                      Duration(days: 2),
                    ),
                    brusselsTimeZone,
                  ),
                  tz.TZDateTime.from(
                    // Easter Monday
                    DateTime.parse('${easter}T00:00:00Z').add(
                      Duration(days: 1),
                    ),
                    brusselsTimeZone,
                  ),
                ].any((DateTime holiday) =>
                    brusselsNow.year == holiday.year &&
                    brusselsNow.month == holiday.month &&
                    brusselsNow.day == holiday.day)) &&

        /// check if it's after 16:00
        (brusselsNow.hour > 16 ||
            (brusselsNow.hour == 16 && brusselsNow.minute > 0));
  }

  static Future<ExchangeRates?> _downloadLatest() async {
    final http.Response response = await http.get(
      Uri.https(
        'www.ecb.europa.eu',
        '/stats/eurofxref/eurofxref-daily.xml',
      ),
    );

    if (response.statusCode < 400) {
      try {
        return ExchangeRates.fromXml(
          XmlDocument.parse(response.body),
          Rfc822.parse(response.headers[HttpHeaders.lastModifiedHeader] ?? ''),
        );
      } on XmlException catch (error, stackTrace) {
        developer.log(
          'XML document can not be parsed',
          error: error,
          stackTrace: stackTrace,
        );
      } catch (error, stackTrace) {
        developer.log(error.toString(), error: error, stackTrace: stackTrace);
      }
    }

    return null;
  }
}
