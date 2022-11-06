// ignore_for_file: long-method

part of 'main.dart';

final Logger log = Logger('alfred_convert_workflow');

void _logListener(LogRecord record) {
  if (record.error != null && record.stackTrace != null) {
    print(
      '${record.level.name}: ${record.time}: ${record.message}\n${record.error}\n${record.stackTrace}'
          .trim(),
    );
  } else if (record.error != null) {
    print(
      '${record.level.name}: ${record.time}: ${record.message}\n${record.error}'
          .trim(),
    );
  } else {
    print(
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

Future<void> _listCurrencies() async {
  final ExchangeRates? rates = await EcbExchangeRates().getLatest();

  final AlfredItems items = AlfredItems(
    await Future.wait(Currency.values.map((currency) async {
      final File? image = await EmojiDownloader(
        '${currency.flag.runes.map((int cp) => cp.toRadixString(16)).join('-')}.png',
      ).downloadImage();

      if (currency != Currency.USD) {
        try {
          final ExchangeRate? rate = rates?.convert(currency, Currency.USD);
          final DecimalIntl convertedValue = DecimalIntl(rate!.rate);

          return AlfredItem(
            uid: currency.name,
            title: '${currency.fullName} (${currency.name})',
            subtitle: '1 ${currency.name} ≃'
                ' ${numberFormat.format(convertedValue)}'
                ' ${Currency.USD.name}',
            arg: currency.name,
            match: '${currency.fullName} (${currency.name})',
            text: AlfredItemText(
              copy: currency.name,
              largeType: currency.name,
            ),
            icon: AlfredItemIcon(
              path: image != null ? image.absolute.path : 'icon.png',
            ),
            valid: true,
          );
        } on ArgumentError {
          return AlfredItem(
            uid: currency.name,
            title: '${currency.fullName} (${currency.name})',
            arg: currency.name,
            match: '${currency.fullName} (${currency.name})',
            text: AlfredItemText(
              copy: currency.name,
              largeType: currency.name,
            ),
            icon: AlfredItemIcon(
              path: image != null ? image.absolute.path : 'icon.png',
            ),
            valid: true,
          );
        }
      } else {
        return AlfredItem(
          uid: currency.name,
          title: '${currency.fullName} (${currency.name})',
          arg: currency.name,
          match: '${currency.fullName} (${currency.name})',
          text: AlfredItemText(
            copy: currency.name,
            largeType: currency.name,
          ),
          icon: AlfredItemIcon(
            path: image != null ? image.absolute.path : 'icon.png',
          ),
          valid: true,
        );
      }
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

Future<void> _convert(String query) async {
  final List<String> parts = query.split(' ');

  if (parts.length < 3 || parts.length > 4) return _invalidFormat();

  final Decimal? value = Decimal.tryParse(parts[0]);
  if (value == null) return _invalidFormat();

  final String fromUnitSymbol = parts[1].trim();
  final String toUnitSymbol =
      parts.length == 4 && parts[2].trim().toLowerCase() == 'to'
          ? parts[3].trim()
          : parts[2].trim();

  try {
    /// First try to convert a currency
    final AlfredItem? currencyItem = await Convert.convertCurrency(
      value,
      fromUnitSymbol,
      toUnitSymbol,
    );
    if (currencyItem != null) {
      _workflow.addItem(currencyItem);
    } else {
      /// Then try the others
      _workflow.addItem(
        await Convert.convertUnit(
          value.toDouble(),
          fromUnitSymbol,
          toUnitSymbol,
        ),
      );
    }
  } on ArgumentError catch (error, stackTrace) {
    if (_verbose) {
      log.severe(
        error.message,
        error,
        stackTrace,
      );
    }
    _invalidFormat('${error.message}: "${error.invalidValue}"');
  } catch (error, stackTrace) {
    if (_verbose) {
      log.severe(
        'Error calling _convert',
        error,
        stackTrace,
      );
    }
    _invalidFormat(error.toString());
  }
}
