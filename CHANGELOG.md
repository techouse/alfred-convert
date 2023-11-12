## 1.1.0

- [FEAT] modify setting the `default_currency` via the Workflow Configuration

## 1.0.7

- [FEAT] modify the copy to clipboard behaviour for currency related conversions
- [CHORE] update [alfred_workflow](https://pub.dev/packages/alfred_workflow) to 0.4.2 (was 0.4.1)
- [CHORE] update [build_daemon](https://pub.dev/packages/build_daemon) to 4.0.1 (was 4.0.0)
- [CHORE] update [build_resolvers](https://pub.dev/packages/build_resolvers) to 2.4.1 (was 2.2.1)
- [CHORE] update [build_runner_core](https://pub.dev/packages/build_runner_core) to 7.2.11 (was 7.2.10)
- [CHORE] update [built_value](https://pub.dev/packages/built_value) to 8.7.0 (was 8.6.1)
- [CHORE] update [code_builder](https://pub.dev/packages/code_builder) to 4.7.0 (was 4.5.0)
- [CHORE] update [envied](https://pub.dev/packages/envied) to 0.5.1 (was 0.3.0+3)
- [CHORE] update [envied_generator](https://pub.dev/packages/envied_generator) to 0.5.1 (was 0.3.0+3)
- [CHORE] update [lints](https://pub.dev/packages/lints) to 3.0.0 (was 2.1.1)
- [CHORE] update [meta](https://pub.dev/packages/meta) to 1.11.0 (was 1.9.1)
- [CHORE] update [petitparser](https://pub.dev/packages/petitparser) to 6.0.1 (was 6.0.0)
- [CHORE] update [stash](https://pub.dev/packages/stash) to 5.0.2 (was 5.0.0)
- [CHORE] update [stash_file](https://pub.dev/packages/stash_file) to 5.0.2 (was 5.0.0)
- [CHORE] update [uuid](https://pub.dev/packages/uuid) to 4.2.1 (was 3.0.7)
- [CHORE] update [xml](https://pub.dev/packages/xml) to 6.4.2 (was 6.4.0)

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
