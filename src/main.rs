use crate::unicode_blocks::{UnicodeBlock, UnicodeBlocks};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod unicode_blocks;

const BLOCKS_FILE: &str = "Blocks.txt";
const UNICODE_DATA_FILE: &str = "UnicodeData.txt";
const GENERATED_CODE_DIR: &str = "unicode_types/src";

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

fn generate_mod_rs(blocks: &UnicodeBlocks) -> std::io::Result<()> {
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

fn generate_block_files(blocks: &Vec<UnicodeBlock>) -> std::io::Result<()> {
    for block in blocks {
        let filename = block.as_snake_case() + ".rs";
        let file = PathBuf::from(GENERATED_CODE_DIR).join(filename);
        let content = String::from("enum ") + block.as_upper_camel_case().as_str() + " {\n}";
        let mut file = File::create(file)?;
        file.write_all(&content.bytes().collect::<Vec<_>>())?;
    }
    Ok(())
}

fn generate_unicode_types(blocks: &UnicodeBlocks) -> std::io::Result<()> {
    create_dir_all(GENERATED_CODE_DIR)?;
    generate_mod_rs(blocks)?;
    generate_block_files(&blocks.blocks)
    // TODO:
    // const
    // Variants for enums
}

fn main() {
    let options = Options::from_args();
    let blocks_file = options.ucd_dir.join(BLOCKS_FILE);
    let blocks = UnicodeBlocks::from_file(blocks_file).expect("Parse error");
    generate_unicode_types(&blocks);
    println!("{:#?}", blocks);
}
