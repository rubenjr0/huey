[![Stand With Ukraine](https://raw.githubusercontent.com/vshymanskyy/StandWithUkraine/main/banner2-direct.svg)](https://stand-with-ukraine.pp.ua)

# Huey

[![Crates.io](https://img.shields.io/crates/v/huey?style=for-the-badge)](https://crates.io/crates/huey)
[![GitHub Repo stars](https://img.shields.io/github/stars/rubenjr0/huey?style=for-the-badge)](https://github.com/rubenjr0/huey)
[![Crates.io](https://img.shields.io/crates/l/huey?style=for-the-badge)](https://www.apache.org/licenses/LICENSE-2.0)

Huey is a CLI tool for colorizing images with a specified palette. Instead of using specialized tools to colorize images with a specific palette (e.g. [Catppuccin](https://github.com/catppuccin/catppuccin), [Srcery](https://srcery.sh/), etc.) you can just use `huey path-to-image path-to-palete`!

## Installation

You can install `huey` by using cargo:

```bash
$ cargo install huey
```

## Usage

```bash
$ huey <IMAGE_PATH> <PALETTE_PATH> [OUTPUT_PATH] [OPTIONS]
```

Check the program's help for more information.
```bash
$ huey --help
```

### Options

The options available are `-o`, `-i` and `-r`.

You can specify an output path with `-o`, the default is "colorized.png".

The `-i` flag lets you specify an interpolation mode from `mix` and `interpolate`. The `interpolate` mode takes the 2 closest colors to a pixel and simply takes a color in between them in the specified color space, while the `mix` mode does the same but staying closer to the closest color. In the `mix` mode the mixing factor is calculated with `d1 / (d1 + d2)`, where `d1` is the distance to the closest color and `d2` is the distance to the second closest color.

You can use the `-r` flag if you want to use the RGB color space to calculate the closest colors, `huey` uses the OKLAB color space by default.

### Palettes
Palette files are simply text files containing the palette's colors in RGB hex format (i.e. `#77FF00`, although the pad symbol is optional).

An example palette could be:

```
#000000 #121212
#0066FF #00FF66
#77FF00
```

You can store your palettes wherever you want.

# License

This tool is made available under the [Apache License, Version 2](https://www.apache.org/licenses/LICENSE-2.0).
