import 'package:autoequal/autoequal.dart';
import 'package:decimal/decimal.dart';
import 'package:equatable/equatable.dart' show EquatableMixin;
import 'package:json_annotation/json_annotation.dart';

import 'currency.dart';

part 'exchange_rate.g.dart';

@autoequalMixin
@JsonSerializable()
class ExchangeRate with EquatableMixin, _$ExchangeRateAutoequalMixin {
  const ExchangeRate({
    required this.currency,
    required this.rate,
  });

  final Currency currency;
  @JsonKey(fromJson: _rateFromJson, toJson: _rateToJson)
  final Decimal rate;

  static _rateFromJson(String rate) => Decimal.fromJson(rate);

  static _rateToJson(Decimal rate) => rate.toJson();

  factory ExchangeRate.fromJson(Map<String, dynamic> json) =>
      _$ExchangeRateFromJson(json);

  Map<String, dynamic> toJson() => _$ExchangeRateToJson(this);
}
