use crate::unicode_blocks::UnicodeBlocks;
use std::fs::{create_dir, File};
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod unicode_blocks;

const BLOCKS_FILE: &str = "Blocks.txt";
const UNICODE_DATA_FILE: &str = "UnicodeData.txt";
const GENERATED_CODE_DIR: &str = "unicode_types";

#[derive(StructOpt)]
struct Options {
    /// Path to the ucd directory.
    /// This directory must contain the plain text unicode data.
    /// The latest compressed directory may be downloaded from the
    /// unicode consortium's website:
    /// https://www.unicode.org/Public/UCD/latest/ucd/UCD.zip
    #[structopt(long = "ucd_dir", short = "d")]
    ucd_dir: PathBuf,
}

fn generate_unicode_types(blocks: &UnicodeBlocks) -> std::io::Result<()> {
    // TODO:
    // mkdir unicode
    // touch unicode/mod.rs
    // dump the comment in there
    // list the mods in there
    // for each block
    // touch unicode/block.as_snake_case().rs
    // place enums in there
    // Variants will come later
    create_dir(GENERATED_CODE_DIR)?;
    let mod_file = PathBuf::from(GENERATED_CODE_DIR).join("mod.rs");
    let mod_content = blocks
        .comments
        .iter()
        .map(move |line| {
            if line.is_empty() {
                String::from("///\n")
            } else {
                String::from("/// ") + &line + "\n"
            }
        })
        .chain(
            blocks
                .blocks
                .iter()
                .map(|block| String::from("\n") + "mod " + block.as_snake_case().as_str() + ";"),
        )
        .chain(std::iter::once(String::from("\n")))
        .collect::<Vec<_>>();
    let binary_mod_content = mod_content
        .iter()
        .map(|s| s.bytes().collect::<Vec<_>>())
        .flatten()
        .collect::<Vec<_>>();
    let mut file = File::create(mod_file)?;
    file.write_all(&binary_mod_content)
}

fn main() {
    let options = Options::from_args();
    let blocks_file = options.ucd_dir.join(BLOCKS_FILE);
    let blocks = UnicodeBlocks::from_file(blocks_file).expect("Parse error");
    generate_unicode_types(&blocks);
    println!("{:#?}", blocks);
}
