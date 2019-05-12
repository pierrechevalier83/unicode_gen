unicode_gen
===

Generate rust code that maps each block of the Unicode Chart Database to a mod
and an enum, and each character in this block to a variant of this enum.

Example usage:
===
```
cargo run --release -- --ucd_dir "../ucd" --out_dir "../unicode/src/generated"
```

```
cargo run --release -- --help
```

```
unicode_gen 0.1.0
Pierre Chevalier <pierrechevalier83@gmail.com>

USAGE:
    unicode_gen --out_dir <out_dir> --ucd_dir <ucd_dir>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --out_dir <out_dir>    Directory where to generate the code
    -d, --ucd_dir <ucd_dir>    Path to the Unicode Character Database (UCD) directory. This directory must contain the
                               plain text unicode data. The latest compressed directory may be downloaded from the
                               unicode consortium's website: https://www.unicode.org/Public/UCD/latest/ucd/UCD.zip
```
