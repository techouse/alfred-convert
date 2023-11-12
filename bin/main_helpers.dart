// ignore_for_file: long-method

part of 'main.dart';

final Logger log = Logger('alfred_convert_workflow');

void _logListener(LogRecord record) {
  if (record.error != null && record.stackTrace != null) {
    stdout.write(
      '${record.level.name}: ${record.time}: ${record.message}\n${record.error}\n${record.stackTrace}'
          .trim(),
    );
  } else if (record.error != null) {
    stdout.write(
      '${record.level.name}: ${record.time}: ${record.message}\n${record.error}'
          .trim(),
    );
  } else {
    stdout.write(
      '${record.level.name}: ${record.time}: ${record.message}'.trim(),
    );
  }
}

final NumberFormat numberFormat = NumberFormat();

final AlfredWorkflow _workflow = AlfredWorkflow();

final AlfredUpdater _updater = AlfredUpdater(
  githubRepositoryUrl: Uri.parse(Env.githubRepositoryUrl),
  currentVersion: Env.appVersion,
  updateInterval: Duration(days: 7),
);

const updateItem = AlfredItem(
  title: 'Auto-Update available!',
  subtitle: 'Press <enter> to auto-update to a new version of this workflow.',
  arg: 'update:workflow',
  match:
      'Auto-Update available! Press <enter> to auto-update to a new version of this workflow.',
  icon: AlfredItemIcon(path: 'alfredhatcog.png'),
  valid: true,
);

void _showPlaceholder() {
  _workflow.addItem(
    const AlfredItem(
      title: 'Convert from ... to ...',
      icon: AlfredItemIcon(path: 'icon.png'),
    ),
  );
}

void _invalidFormat([String? message]) {
  _workflow.addItem(
    AlfredItem(
      title: message ?? 'Invalid format.',
      subtitle: 'Usage: conv 123.45 gbp usd',
      icon: AlfredItemIcon(path: 'icon.png'),
      valid: false,
    ),
  );
}

Future<void> _listCurrencies({Currency homeCurrency = Currency.USD}) async {
  final ExchangeRates? rates = await EcbExchangeRates().getLatest();

  final AlfredItems items = AlfredItems(
    await Future.wait(Currency.values.map((currency) async {
      final File? image = await EmojiDownloader(
        '${currency.flag.runes.map((int cp) => cp.toRadixString(16)).join('-')}.png',
      ).downloadImage();

      if (currency != homeCurrency) {
        try {
          final ExchangeRate? rate = rates?.convert(currency, homeCurrency);

          final Uri xeUrl = Uri.https('www.xe.com', 'currencycharts', {
            'from': currency.name,
            'to': homeCurrency.name,
          });

          if (rate != null) {
            final DecimalIntl convertedValue = DecimalIntl(rate.rate);
            final DecimalIntl invertedValue = DecimalIntl(rate.invertedRate);

            return AlfredItem(
              title: '${currency.fullName} (${currency.name})',
              subtitle: '1 ${currency.name} ≃'
                  ' ${numberFormat.format(convertedValue)}'
                  ' ${homeCurrency.name}',
              arg: xeUrl.toString(),
              quickLookUrl: xeUrl.toString(),
              match: '${currency.fullName} (${currency.name})',
              text: AlfredItemText(
                copy: currency.name,
                largeType: currency.name,
              ),
              icon: AlfredItemIcon(
                path: image != null ? image.absolute.path : 'icon.png',
              ),
              valid: true,
              mods: {
                {AlfredItemModKey.alt}: AlfredItemMod(
                  subtitle: '1 ${homeCurrency.name} ≃'
                      ' ${numberFormat.format(invertedValue)}'
                      ' ${currency.name}',
                  valid: true,
                ),
                {AlfredItemModKey.cmd}: AlfredItemMod(
                  subtitle: 'Copy ${numberFormat.format(convertedValue)}'
                      ' ${homeCurrency.name} ${homeCurrency.flag} to clipboard',
                  arg: '${numberFormat.format(convertedValue)} '
                      '${homeCurrency.name}',
                  valid: true,
                ),
              },
            );
          }
        } catch (error, stackTrace) {
          if (_verbose) {
            log.warning(
              'Error getting exchange rate for ${currency.name}',
              error,
              stackTrace,
            );
          }
        }
      }

      final Uri oandaUrl = Uri.https(
        'www.oanda.com',
        'currency-converter/en/currencies/majors/${currency.name.toLowerCase()}/',
      );

      return AlfredItem(
        title: '${currency.fullName} (${currency.name})',
        subtitle: 'Open currency fact sheet',
        arg: oandaUrl.toString(),
        quickLookUrl: oandaUrl.toString(),
        match: '${currency.fullName} (${currency.name})',
        text: AlfredItemText(
          copy: currency.name,
          largeType: currency.name,
        ),
        icon: AlfredItemIcon(
          path: image != null ? image.absolute.path : 'icon.png',
        ),
        valid: true,
        mods: {
          {AlfredItemModKey.cmd}: AlfredItemMod(
            subtitle: 'Copy ${homeCurrency.fullName} (${homeCurrency.name}) '
                '${homeCurrency.flag} to clipboard',
            arg: '${homeCurrency.fullName} ${homeCurrency.name}',
            valid: true,
          ),
        },
      );
    }).toList()),
  );
  _workflow.addItems(items.items);
}

Future<void> _listUnits() async {
  final Map<String, AlfredItem> items = {};

  for (final Map<String, Enum> property in properties) {
    for (final MapEntry<String, Enum> entry in property.entries) {
      final Unit? unit = Convert.getProperty(entry.value)?.getUnit(entry.value);
      final String? emoji = propertyEmojis[entry.value.runtimeType];
      final File? image = emoji != null
          ? await EmojiDownloader(
              '${emoji.runes.first.toRadixString(16)}.png',
            ).downloadImage()
          : null;

      if (!items.containsKey(entry.value.name)) {
        items[entry.value.name] = AlfredItem(
          uid: entry.value.name,
          title: entry.value.name.sentenceCase,
          subtitle: unit?.symbol ?? entry.value.name,
          arg: entry.key,
          match:
              '${entry.value.name.sentenceCase} [${unit?.symbol ?? entry.value.name}]',
          text: AlfredItemText(
            copy: entry.key,
            largeType: entry.key,
          ),
          icon: AlfredItemIcon(
            path: image != null ? image.absolute.path : 'icon.png',
          ),
          valid: true,
        );
      }
    }
  }

  _workflow.addItems(items.values.toList());
}

Future<void> _convert(
  String query, {
  Currency homeCurrency = Currency.USD,
}) async {
  final List<String> parts = query.split(' ');

  if (parts.length < 2) return _invalidFormat();

  final Decimal? value = Decimal.tryParse(parts[0]);
  if (value == null) return _invalidFormat();

  final String fromUnitSymbol = parts[1].trim();

  try {
    /// First try to convert a currency
    late final String toUnitSymbol;

    switch (parts.length) {
      case 3:
        toUnitSymbol = parts[2].trim().toLowerCase() == 'to'
            ? homeCurrency.name
            : parts[2].trim();
        break;
      case 4:
        if (parts[2].trim().toLowerCase() != 'to') {
          return _invalidFormat();
        }
        toUnitSymbol = parts[3].trim();
        break;
      default:
        toUnitSymbol = homeCurrency.name;
    }

    final AlfredItem? currencyItem = await Convert.convertCurrency(
      value,
      fromUnitSymbol,
      toUnitSymbol,
    );
    if (currencyItem != null) {
      _workflow.addItem(currencyItem);
    } else {
      if (parts.length < 3 || parts.length > 4) return _invalidFormat();

      /// Then try the others
      _workflow.addItem(
        await Convert.convertUnit(
          value.toDouble(),
          fromUnitSymbol,
          parts.length == 4 && parts[2].trim().toLowerCase() == 'to'
              ? parts[3].trim()
              : parts[2].trim(),
        ),
      );
    }
  } on ArgumentError catch (error, stackTrace) {
    if (_verbose) {
      log.severe(error.message, error, stackTrace);
    }
    _invalidFormat('${error.message}: "${error.invalidValue}"');
  } catch (error, stackTrace) {
    if (_verbose) {
      log.severe('Error calling _convert', error, stackTrace);
    }
    _invalidFormat(error.toString());
  }
}
