# Alfred Convert Workflow

![GitHub release](https://img.shields.io/github/release/techouse/alfred-convert.svg)
![GitHub All Releases](https://img.shields.io/github/downloads/techouse/alfred-convert/total.svg)
![GitHub](https://img.shields.io/github/license/techouse/alfred-convert.svg)

Convert between different units in Alfred.

Heavily inspired by [deanishe/alfred-convert](https://github.com/deanishe/alfred-convert) 😊

![demo](demo.gif)

## Installation

1. [Download the latest version](https://github.com/techouse/alfred-convert/releases/latest)
2. Install the workflow by double-clicking the `.alfredworkflow` file
3. You can add the workflow to a category, then click "Import" to finish importing. You'll now see the workflow listed in the left sidebar of your Workflows preferences pane.

## Usage

- `conv <quantity> <from unit> <to unit>` - Perform a conversion
    - For monetary conversions, <kbd>return</kbd> (↵) performs the configured Default monetary action: open the currency-pair chart on [Xe.com](http://www.xe.com) or copy the converted value.
      - <kbd>cmd+return</kbd> (⌘↵) performs the alternate action.
      - <kbd>option+return</kbd> (⌥↵) shows the inverse conversion and copies its value when Copy to clipboard is the default.
      - Quick Look (`⌘Y`) always opens the Xe chart.
    - For physical unit conversions and other Numbat evaluations, <kbd>return</kbd> (↵) performs the configured Default non-monetary action: open the result on [WolframAlpha.com](https://www.wolframalpha.com) or copy the converted value.
      - <kbd>cmd+return</kbd> (⌘↵) performs the alternate action.
      - Quick Look (`⌘Y`) always opens the WolframAlpha result.
- `conv money` - View a list of all the supported currencies
    - Rate-backed rows use the same configured Default monetary action and <kbd>cmd+return</kbd> (⌘↵) alternate action.
      - <kbd>option+return</kbd> (⌥↵) shows the inverse rate and copies it when Copy to clipboard is the default.
      - Quick Look (`⌘Y`) always opens the Xe chart.
- `conv units` - View a list of all the supported physical units
    - When selecting a certain unit and pressing <kbd>return</kbd> (↵) that unit's symbol will get copied to the clipboard.

### Default currency

In order to set a default currency, you can set it in the Workflow Configuration.

![default_currency](default_currency.png)

Valid values are the [ISO 4217](https://en.wikipedia.org/wiki/ISO_4217) currency codes currently published by the ECB: AUD, BRL, CAD, CHF, CNY, CZK, DKK, EUR, GBP, HKD, HUF, IDR, ILS, INR, ISK, JPY, KRW, MXN, MYR, NOK, NZD, PHP, PLN, RON, SEK, SGD, THB, TRY, USD, ZAR.

### Default customary units

![default_customary_units](default_customary_units.png)

The Workflow Configuration defaults ambiguous historical shorthand to Imperial (UK) units. Choose US customary to use US definitions for bare `gal`, `pt`, `fl.oz`/`floz`, `tbsp.`, `tsp.`, and `mpg`. Explicit `us.gal`, `us.pt`, `us.fl.oz`, `us.floz`, and `us.mpg` aliases always use US customary units, while native Numbat names such as `gallon`, `tablespoon`, and `fluidounce` remain explicit. `cup` is always US customary.

To convert between systems explicitly, use the lowercase `uk_`/`us_` aliases in the three- or four-token form. Short and long spellings are available as `uk_gal`/`uk_gallon`, `uk_qt`/`uk_quart`, `uk_pt`/`uk_pint`, `uk_gi`/`uk_gill`, `uk_floz`/`uk_fluid_ounce`, `uk_fldr`/`uk_fluid_drachm`, `uk_tbsp`/`uk_tablespoon`, `uk_tsp`/`uk_teaspoon`, and `uk_mpg`/`uk_miles_per_gallon`, with corresponding `us_` forms (the US long fluid-dram spelling is `us_fluid_dram`). For example, `conv 1 uk_floz to us_floz` returns `1 UK fl oz = 0.961 US fl oz`. Explicit aliases ignore the customary-unit preference in both source and target positions. `uk_cup`, bushel, and hogshead aliases are not defined because this workflow has no like-for-like pair for them.

### Default actions

| monetary | non-monetary |
| -- | -- |
| ![default_monetary_action](default_monetary_action.png) | ![default_non_monetary_action](default_non_monetary_action.png) |

The Workflow Configuration provides separate Default monetary action and Default non-monetary action preferences. `Open website` remains the default for both; choose `Copy to clipboard` to copy only the converted value instead. <kbd>cmd+return</kbd> always performs the alternate action.

### Notes

- All [the reference exchange rates are from the ECB](https://www.ecb.europa.eu/stats/policy_and_exchange_rates/euro_reference_exchange_rates/html/index.en.html). The reference rates are usually updated around 16:00 CET on every working day, except on [TARGET closing days](https://www.ecb.europa.eu/home/contacts/working-hours/html/index.en.html).

- Non-monetary conversions are evaluated by [Numbat](https://numbat.dev/). Native Numbat expressions such as `2in to cm`, arithmetic, constants, and compound units are supported alongside the workflow's historical shorthand.

- The displayed emoji images are from [joypixels/emoji-assets](https://github.com/joypixels/emoji-assets).

## Development

The workflow is an unpublished Rust 2024 crate and requires Rust 1.88 or newer.
Install the locked license generator before running the full local checks:

```shell
cargo install cargo-about --locked --features cli
make ci
```

`make build-release` creates a native development build. `make package` builds both Apple Silicon and Intel slices and creates a universal `.alfredworkflow` archive under `build/`.
