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
3. You can add the workflow to a category, then click "Import" to finish importing. You'll now see the workflow listed
   in the left sidebar of your Workflows preferences pane.

## Usage

- `conv <quantity> <from unit> <to unit>` - Perform a conversion
    - When performing a monetary conversion pressing `⌘Y` or `<return>` will open the currency-pair chart
      on [Xe.com](http://www.xe.com).
    - When performing a physical unit conversion pressing `⌘Y` or `<return>` will open up detailed the conversion
      explanation on [WolframAlpha.com](https://www.wolframalpha.com).
- `conv money` - View a list of all the supported currencies
    - When selecting a certain currency and pressing `<return>` that
      currency's [ISO 4217](https://en.wikipedia.org/wiki/ISO_4217) code will get copied to the clipboard.
- `conv units` - View a list of all the supported physical units
    - When selecting a certain unit and pressing `<return>` that unit's symbol will get copied to the clipboard.

### Notes

- All [the reference exchange rates are from the ECB](https://www.ecb.europa.eu/stats/policy_and_exchange_rates/euro_reference_exchange_rates/html/index.en.html).
  The reference rates are usually updated around 16:00 CET on every working day, except
  on [TARGET closing days](https://www.ecb.europa.eu/home/contacts/working-hours/html/index.en.html).

- All non-monetary conversions performed using [ferraridamiano/units_converter](https://github.com/ferraridamiano/units_converter). 

- The displayed emoji images are from [joypixels/emoji-assets](https://github.com/joypixels/emoji-assets).