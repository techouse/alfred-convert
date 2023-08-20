## 1.0.6

- [CHORE] update [alfred_workflow](https://pub.dev/packages/alfred_workflow) to 0.4.1 (was 0.4.0)
- [CHORE] update [collection](https://pub.dev/packages/collection) to 1.18.0 (was 1.17.2)
- [CHORE] update [copy_with_extension](https://pub.dev/packages/copy_with_extension) to 5.0.4 (was 5.0.3)
- [CHORE] update [petitparser](https://pub.dev/packages/petitparser) to 6.0.0 (was 5.4.0)
- [CHORE] update [units_converter](https://pub.dev/packages/units_converter) to 2.1.1 (was 2.1.0)
- [CHORE] update [xml](https://pub.dev/packages/xml) to 6.4.0 (was 6.3.0)

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
