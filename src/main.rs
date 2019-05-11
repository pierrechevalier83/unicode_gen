use crate::unicode_blocks::{Range, UnicodeBlock, UnicodeBlocks};
use crate::unicode_data::{UnicodeCharacter, UnicodeData};
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod unicode_blocks;
mod unicode_data;

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

fn characters_in_range(range: &Range, data: &UnicodeData) -> Vec<UnicodeCharacter> {
    (range.begin..range.end)
        .flat_map(|index| data.0.get(&index))
        .cloned()
        .collect()
}

fn generate_block_files(blocks: &Vec<UnicodeBlock>, data: &UnicodeData) -> std::io::Result<()> {
    for block in blocks {
        let filename = block.as_snake_case() + ".rs";
        let file = PathBuf::from(GENERATED_CODE_DIR).join(filename);
        let characters = characters_in_range(&block.range, data);
        let mut content = String::new();
        content += "mod constants {\n";
        for c in &characters {
            content = content
                + "    const "
                + c.as_upper_snake_case().as_str()
                + ": char = '"
                + c.character.to_string().as_str()
                + "';\n";
        }
        content += "}";
        content = content + "\npub enum " + block.as_upper_camel_case().as_str() + " {\n";

        for c in characters {
            content = content
                + "    /// '"
                + c.character.to_string().as_str()
                + "'\n"
                + "    "
                + c.as_upper_camel_case().as_str()
                + ","
                + "\n";
        }
        content += "}\n";

        let mut file = File::create(file)?;
        file.write_all(&content.bytes().collect::<Vec<_>>())?;
    }
    Ok(())
}

fn generate_unicode_types(blocks: &UnicodeBlocks, data: &UnicodeData) -> std::io::Result<()> {
    create_dir_all(GENERATED_CODE_DIR)?;
    generate_mod_rs(blocks)?;
    generate_block_files(&blocks.blocks, data)
}

fn main() {
    let options = Options::from_args();
    let blocks_file = options.ucd_dir.join(BLOCKS_FILE);
    let blocks = UnicodeBlocks::from_file(blocks_file).expect("Parse Error");
    let data_file = options.ucd_dir.join(UNICODE_DATA_FILE);
    let data = UnicodeData::from_file(data_file).expect("Parse Error");
    generate_unicode_types(&blocks, &data).expect("Error generating code");
}
