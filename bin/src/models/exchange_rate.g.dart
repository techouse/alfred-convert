// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'exchange_rate.dart';

// **************************************************************************
// AutoequalGenerator
// **************************************************************************

mixin _$ExchangeRateAutoequalMixin on EquatableMixin {
  @override
  List<Object?> get props =>
      _$ExchangeRateAutoequal(this as ExchangeRate)._$props;
}

extension _$ExchangeRateAutoequal on ExchangeRate {
  List<Object?> get _$props => [currency, rate];
}

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ExchangeRate _$ExchangeRateFromJson(Map<String, dynamic> json) => ExchangeRate(
      currency: $enumDecode(_$CurrencyEnumMap, json['currency']),
      rate: ExchangeRate._rateFromJson(json['rate'] as String),
    );

Map<String, dynamic> _$ExchangeRateToJson(ExchangeRate instance) =>
    <String, dynamic>{
      'currency': _$CurrencyEnumMap[instance.currency]!,
      'rate': ExchangeRate._rateToJson(instance.rate),
    };

const _$CurrencyEnumMap = {
  Currency.AUD: 'AUD',
  Currency.BGN: 'BGN',
  Currency.BRL: 'BRL',
  Currency.CAD: 'CAD',
  Currency.CHF: 'CHF',
  Currency.CNY: 'CNY',
  Currency.CZK: 'CZK',
  Currency.DKK: 'DKK',
  Currency.EUR: 'EUR',
  Currency.GBP: 'GBP',
  Currency.HKD: 'HKD',
  Currency.HRK: 'HRK',
  Currency.HUF: 'HUF',
  Currency.IDR: 'IDR',
  Currency.ILS: 'ILS',
  Currency.INR: 'INR',
  Currency.ISK: 'ISK',
  Currency.JPY: 'JPY',
  Currency.KRW: 'KRW',
  Currency.MXN: 'MXN',
  Currency.MYR: 'MYR',
  Currency.NOK: 'NOK',
  Currency.NZD: 'NZD',
  Currency.PHP: 'PHP',
  Currency.PLN: 'PLN',
  Currency.RON: 'RON',
  Currency.RUB: 'RUB',
  Currency.SEK: 'SEK',
  Currency.SGD: 'SGD',
  Currency.THB: 'THB',
  Currency.TRY: 'TRY',
  Currency.USD: 'USD',
  Currency.ZAR: 'ZAR',
};
