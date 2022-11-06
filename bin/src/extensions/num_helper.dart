import 'dart:math' show pow;

import 'package:decimal/decimal.dart' show Decimal;
import 'package:rational/rational.dart' show Rational;

extension RoundedNum on num {
  num get rounded => this - toInt() > 0 ? roundDouble(2) : toInt();

  num roundDouble([int places = 0]) {
    if (places > 0) {
      final num mod = pow(10.0, places);

      return (this * mod).roundToDouble() / mod;
    }

    return roundToDouble();
  }
}

extension DecimalNum on num {
  Decimal toDecimal() => Decimal.parse(toString());

  Rational toRational() => Rational.parse(toString());
}
