# emojicrunch

Export emojis as PNGs/WEBPs and crunch them in size.

CI currently exports WEBPs and PNGs (without zopfli compression).
The build can be found [here](https://github.com/Nerixyz/emojicrunch/releases/tag/nightly-build).

## Building

Building requires Rust, a C++, and a C compiler:

```
cargo b -r
```

## Usage

```bash
# generate PNGs for all providers
cargo r -r
# generate PNGs for twitter (twemoji) and use zopflipng to compress them
cargo r -r -- twitter --use-zopfli
# generate WEBPs for google (noto-emoji) and apple
cargo r -r -- google apple --webp
```

All outputs will be located in `build/{vendor}/{size}`.
By default, 22x22, 44x44, and 88x88 images are generated.
This can be overwritten with `--sizes` (multiple arguments).

Full help:

```text
Usage: emojicrunch [OPTIONS] [VENDORS]...

Arguments:
  [VENDORS]...  List of vendors to build for [possible values: apple, twitter, google, facebook]

Options:
      --apple-font <PATH>
          Sets the path to the Apple font
  -o <OUTPUT_DIR>
          [default: build]
      --emoji-data-root <EMOJI_DATA_ROOT>
          Root directory of the emoji-data repository [default: emoji-data]
      --use-zopfli
          Use zopfli to compress the PNGs
      --webp
          Produce WEBPs
      --size <SIZE>
          The size(s) to scale/render to. Accepts multiple arguments
```
