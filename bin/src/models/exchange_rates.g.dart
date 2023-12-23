// GENERATED CODE - DO NOT MODIFY BY HAND

part of 'exchange_rates.dart';

// **************************************************************************
// AutoequalGenerator
// **************************************************************************

mixin _$ExchangeRatesAutoequalMixin on EquatableMixin {
  @override
  List<Object?> get props =>
      _$ExchangeRatesAutoequal((this as ExchangeRates))._$props;
}

extension _$ExchangeRatesAutoequal on ExchangeRates {
  List<Object?> get _autoequalProps => _$props;
  List<Object?> get _$props => [
        date,
        rates,
      ];
}

// **************************************************************************
// JsonSerializableGenerator
// **************************************************************************

ExchangeRates _$ExchangeRatesFromJson(Map<String, dynamic> json) =>
    ExchangeRates(
      date: UtcDateTimeConverter.instance.fromJson(json['date'] as String),
      rates: ExchangeRates._itemsFromJson(json['rates'] as List),
    );

Map<String, dynamic> _$ExchangeRatesToJson(ExchangeRates instance) =>
    <String, dynamic>{
      'date': UtcDateTimeConverter.instance.toJson(instance.date),
      'rates': instance.rates.map((e) => e.toJson()).toList(),
    };
