// ignore_for_file: long-method

import 'dart:io' show File, exitCode, stdout;

import 'package:alfred_workflow/alfred_workflow.dart';
import 'package:args/args.dart';
import 'package:cli_script/cli_script.dart';
import 'package:decimal/decimal.dart';
import 'package:decimal/intl.dart';
import 'package:intl/intl.dart';
import 'package:logging/logging.dart';
import 'package:recase/recase.dart';
import 'package:units_converter/models/unit.dart';

import 'src/constants/units.dart';
import 'src/env/env.dart';
import 'src/mixins/convert.dart';
import 'src/models/currency.dart';
import 'src/models/exchange_rate.dart';
import 'src/models/exchange_rates.dart';
import 'src/services/ecb_exchange_rates.dart';
import 'src/services/emoji_downloader.dart';

part 'main_helpers.dart';

bool _verbose = false;
bool _update = false;

void main(List<String> arguments) {
  wrapMain(() async {
    Logger.root.level = Level.ALL;

    try {
      exitCode = 0;

      _workflow.clearItems();

      final ArgParser parser = ArgParser()
        ..addOption('query', abbr: 'q', defaultsTo: '')
        ..addFlag('currencies', abbr: 'C', defaultsTo: false)
        ..addFlag('units', abbr: 'U', defaultsTo: false)
        ..addFlag('verbose', abbr: 'v', defaultsTo: false)
        ..addFlag('update', abbr: 'u', defaultsTo: false);
      final ArgResults args = parser.parse(arguments);

      _update = args['update'];
      if (_update) {
        stdout.writeln('Updating workflow...');

        return await _updater.update();
      }

      _verbose = args['verbose'];

      if (_verbose) Logger.root.onRecord.listen(_logListener);

      if (args['currencies']) {
        await _listCurrencies();
      } else if (args['units']) {
        await _listUnits();
      } else {
        final String query =
            args['query'].replaceAll(RegExp(r'\s+'), ' ').trim();

        if (_verbose) log.info('Query: "$query"');

        if (query.isEmpty) {
          _showPlaceholder();
        } else {
          final AlfredItems? items = await _workflow.getItems();
          if (items == null || items.items.isEmpty) {
            await _convert(query);
          }
        }
      }
    } on FormatException catch (err) {
      exitCode = 2;
      _workflow.addItem(AlfredItem(title: err.toString()));
    } catch (err) {
      exitCode = 1;
      _workflow.addItem(AlfredItem(title: err.toString()));
      if (_verbose) rethrow;
    } finally {
      if (!_update) {
        if (await _updater.updateAvailable()) {
          _workflow.run(addToBeginning: updateItem);
        } else {
          _workflow.run();
        }
      }
    }
  });
}
