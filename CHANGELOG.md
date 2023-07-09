## 1.0.5

- Remove [dart_code_metrics](https://pub.dev/packages/dart_code_metrics) from dev dependencies as it is being discontinued on 16 July 2023.

## 1.0.4

- Update workflow to Dart 3

## 1.0.3

- Remove the Croatian Kuna (HRK) from the list of supported currencies as Croatia starts using the Euro on 1 January 2023.

## 1.0.2

- Disable Alfred smart result ordering for currency results in favor of alphabetical ordering
- Add <kbd>option+return</kbd> (⌥↵) shortcut to get the inverse currency conversion

## 1.0.1

- Added `home_currency` environment variable to enable a user to set a default currency. If unset it defaults to `USD`.
- Call the compiled binary with `arch -x86_64` to make it run on M1 Macs (arm64) as well.

## 1.0.0

- Initial version.
