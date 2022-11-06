// ignore_for_file: constant_identifier_names

import 'package:collection/collection.dart';
import 'package:emojis/emojis.dart';
import 'package:json_annotation/json_annotation.dart';

enum Currency {
  @JsonValue('AUD')
  AUD('Australian dollar', Emojis.flagAustralia),
  @JsonValue('BGN')
  BGN('Bulgarian lev', Emojis.flagBulgaria),
  @JsonValue('BRL')
  BRL('Brazilian real', Emojis.flagBrazil),
  @JsonValue('CAD')
  CAD('Canadian dollar', Emojis.flagCanada),
  @JsonValue('CHF')
  CHF('Swiss franc', Emojis.flagSwitzerland),
  @JsonValue('CNY')
  CNY('Chinese yuan renminbi', Emojis.flagChina),
  @JsonValue('CZK')
  CZK('Czech koruna', Emojis.flagCzechia),
  @JsonValue('DKK')
  DKK('Danish krone', Emojis.flagDenmark),
  @JsonValue('EUR')
  EUR('Euro', Emojis.flagEuropeanUnion),
  @JsonValue('GBP')
  GBP('Pound sterling', Emojis.flagUnitedKingdom),
  @JsonValue('HKD')
  HKD('Hong Kong dollar', Emojis.flagHongKongSarChina),
  @JsonValue('HRK')
  HRK('Croatian kuna', Emojis.flagCroatia),
  @JsonValue('HUF')
  HUF('Hungarian forint', Emojis.flagHungary),
  @JsonValue('IDR')
  IDR('Indonesian rupiah', Emojis.flagIndonesia),
  @JsonValue('ILS')
  ILS('Israeli shekel', Emojis.flagIsrael),
  @JsonValue('INR')
  INR('Indian rupee', Emojis.flagIndia),
  @JsonValue('ISK')
  ISK('Icelandic krona', Emojis.flagIceland),
  @JsonValue('JPY')
  JPY('Japanese yen', Emojis.flagJapan),
  @JsonValue('KRW')
  KRW('South Korean won', Emojis.flagSouthKorea),
  @JsonValue('MXN')
  MXN('Mexican peso', Emojis.flagMexico),
  @JsonValue('MYR')
  MYR('Malaysian ringgit', Emojis.flagMalaysia),
  @JsonValue('NOK')
  NOK('Norwegian krone', Emojis.flagNorway),
  @JsonValue('NZD')
  NZD('New Zealand dollar', Emojis.flagNewZealand),
  @JsonValue('PHP')
  PHP('Philippine peso', Emojis.flagPhilippines),
  @JsonValue('PLN')
  PLN('Polish zloty', Emojis.flagPoland),
  @JsonValue('RON')
  RON('Romanian leu', Emojis.flagRomania),
  @JsonValue('RUB')
  RUB('Russian rouble', Emojis.flagRussia),
  @JsonValue('SEK')
  SEK('Swedish krona', Emojis.flagSweden),
  @JsonValue('SGD')
  SGD('Singapore dollar', Emojis.flagSingapore),
  @JsonValue('THB')
  THB('Thai baht', Emojis.flagThailand),
  @JsonValue('TRY')
  TRY('Turkish lira', Emojis.flagTurkey),
  @JsonValue('USD')
  USD('US dollar', Emojis.flagUnitedStates),
  @JsonValue('ZAR')
  ZAR('South African rand', Emojis.flagSouthAfrica);

  const Currency(this.fullName, this.flag);

  final String fullName;
  final String flag;

  @override
  String toString() => '$name ($fullName $flag)';

  static Currency? byNameOrNull(String name) =>
      values.firstWhereOrNull((Currency value) => value.name == name);
}
