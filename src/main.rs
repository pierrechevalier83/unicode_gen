use crate::unicode_blocks::{Range, UnicodeBlock, UnicodeBlocks};
use crate::unicode_data::{UnicodeCharacter, UnicodeData};
use std::convert::TryFrom;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

mod unicode_blocks;
mod unicode_data;

const BLOCKS_FILE: &str = "Blocks.txt";
const UNICODE_DATA_FILE: &str = "UnicodeData.txt";

#[derive(StructOpt)]
struct Options {
    /// Path to the Unicode Character Database (UCD) directory.
    /// This directory must contain the plain text unicode data.
    /// The latest compressed directory may be downloaded from the
    /// unicode consortium's website:
    /// https://www.unicode.org/Public/UCD/latest/ucd/UCD.zip
    #[structopt(long = "ucd_dir", short = "d")]
    ucd_dir: PathBuf,
    /// Directory where to generate the code
    #[structopt(long = "out_dir", short = "o")]
    out_dir: PathBuf,
}

fn generate_mod_rs(
    blocks: &UnicodeBlocks,
    data: &UnicodeData,
    out_dir: &PathBuf,
) -> std::io::Result<()> {
    let lib_file = PathBuf::from(out_dir).join("mod.rs");
    let lib_content = blocks
        .0
        .iter()
        .map(|block| {
            let characters = characters_in_range(&block.range, data);
            String::new()
                + generate_block_doc_comment(&block, &characters).as_str()
                + "pub mod "
                + block.as_snake_case().as_str()
                + ";\n\n"
        })
        .collect::<Vec<_>>();
    let binary_lib_content = lib_content
        .iter()
        .map(|s| s.bytes().collect::<Vec<_>>())
        .flatten()
        .collect::<Vec<_>>();
    let mut file = File::create(lib_file)?;
    file.write_all(&binary_lib_content)
}

fn characters_in_range(range: &Range, data: &UnicodeData) -> Vec<UnicodeCharacter> {
    (range.begin..range.end)
        .flat_map(|index| data.0.get(&index))
        .cloned()
        .collect()
}

fn generate_char_doc_comment(c: &UnicodeCharacter) -> String {
    String::from("    /// ")
        + c.character.escape_unicode().to_string().as_str()
        + ": '"
        + c.printable_character().as_str()
        + "'\n"
}

fn generate_block_doc_comment(block: &UnicodeBlock, characters: &[UnicodeCharacter]) -> String {
    let begin = char::try_from(block.range.begin)
        .unwrap()
        .escape_unicode()
        .to_string();
    let end = char::try_from(block.range.end)
        .unwrap()
        .escape_unicode()
        .to_string();
    let mut s = String::from("/// ") + begin.as_str() + " → " + end.as_str() + "\\\n" + "///\\\n";
    for chars in characters.chunks(16) {
        s += "///";
        for c in chars {
            s = s + " " + c.printable_character().as_str();
        }
        s += "\n"
    }
    s
}

fn generate_block_constants(block: &UnicodeBlock, characters: &[UnicodeCharacter]) -> String {
    let mut content = generate_block_doc_comment(&block, &characters);
    content += "pub mod constants {\n";
    for c in characters {
        content = content
            + generate_char_doc_comment(&c).as_str()
            + "    pub const "
            + c.as_upper_snake_case().as_str()
            + ": char = '"
            + c.printable_character().as_str()
            + "';\n";
    }
    content += "}\n";
    content
}

fn generate_block_enum(block: &UnicodeBlock, characters: &[UnicodeCharacter]) -> String {
    let mut content = String::new();
    content = content
        + "\n"
        + generate_block_doc_comment(&block, &characters).as_str()
        + "#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]\n"
        + "pub enum "
        + block.as_upper_camel_case().as_str()
        + " {\n";

    for c in characters {
        content = content
            + generate_char_doc_comment(&c).as_str()
            + "    "
            + c.as_upper_camel_case().as_str()
            + ","
            + "\n";
    }
    content += "}\n";
    content
}

fn generate_block_into_char(block: &UnicodeBlock, characters: &[UnicodeCharacter]) -> String {
    let mut content = String::new();
    content = content
        + "\n"
        + "impl Into<char> for "
        + block.as_upper_camel_case().as_str()
        + " {\n"
        + "    fn into(self) -> char {\n"
        + "        use constants::*;\n"
        + "        match self {\n";
    for c in characters {
        content = content
            + "            "
            + block.as_upper_camel_case().as_str()
            + "::"
            + c.as_upper_camel_case().as_str()
            + " => "
            + c.as_upper_snake_case().as_str()
            + ",\n";
    }
    content = content + "        }\n" + "    }\n" + "}\n";
    content
}

fn generate_block_try_from_char(block: &UnicodeBlock, characters: &[UnicodeCharacter]) -> String {
    let mut content = String::new();
    content = content
        + "\n"
        + "impl std::convert::TryFrom<char> for "
        + block.as_upper_camel_case().as_str()
        + " {\n"
        + "    type Error = ();\n"
        + "    fn try_from(c: char) -> Result<Self, Self::Error> {\n"
        + "        use constants::*;\n"
        + "        match c {\n";
    for c in characters {
        content = content
            + "            "
            + c.as_upper_snake_case().as_str()
            + " => Ok("
            + block.as_upper_camel_case().as_str()
            + "::"
            + c.as_upper_camel_case().as_str()
            + "),\n";
    }
    content = content + "            _ => Err(()),\n" + "        }\n" + "    }\n" + "}\n";
    content
}

fn generate_block_into_u32(block: &UnicodeBlock) -> String {
    let mut content = String::new();
    content = content
        + "\n"
        + "impl Into<u32> for "
        + block.as_upper_camel_case().as_str()
        + " {\n"
        + "    fn into(self) -> u32 {\n"
        + "        let c: char = self.into();\n"
        + "        let hex = c\n"
        + "            .escape_unicode()\n"
        + "            .to_string()\n"
        + "            .replace(\"\\\\u{\", \"\")\n"
        + "            .replace(\"}\", \"\");\n"
        + "        u32::from_str_radix(&hex, 16).unwrap()\n"
        + "    }\n"
        + "}\n";
    content
}

fn generate_block_try_from_u32(block: &UnicodeBlock) -> String {
    let mut content = String::new();
    content = content
        + "\n"
        + "impl std::convert::TryFrom<u32> for "
        + block.as_upper_camel_case().as_str()
        + " {\n"
        + "    type Error = ();\n"
        + "    fn try_from(u: u32) -> Result<Self, Self::Error> {\n"
        + "        if let Ok(c) = char::try_from(u) {\n"
        + "            Self::try_from(c)\n"
        + "        } else {\n"
        + "            Err(())\n"
        + "        }\n"
        + "    }\n"
        + "}\n";
    content
}

fn generate_block_iterator(block: &UnicodeBlock) -> String {
    let mut content = String::new();
    content = content
        + "\n"
        + "impl Iterator for "
        + block.as_upper_camel_case().as_str()
        + " {\n"
        + "    type Item = Self;\n"
        + "    fn next(&mut self) -> Option<Self> {\n"
        + "        let index: u32 = (*self).into();\n"
        + "        use std::convert::TryFrom;\n"
        + "        Self::try_from(index + 1).ok()\n"
        + "    }\n"
        + "}\n";
    content
}

fn generate_block_enum_impl(block: &UnicodeBlock, characters: &[UnicodeCharacter]) -> String {
    let mut content = String::new();
    content = content + "\n" + "impl " + block.as_upper_camel_case().as_str() + " {\n";
    // new
    content = content
        + "    /// The character with the lowest index in this unicode block\n"
        + "    pub fn new() -> Self {\n"
        + "        "
        + block.as_upper_camel_case().as_str()
        + "::"
        + characters[0].as_upper_camel_case().as_str()
        + "\n"
        + "    }\n";
    // name
    content = content
        + "\n"
        + "    /// The character's name, all lowercase and space-separated\n"
        + "    pub fn name(&self) -> &str {\n"
        + "        match self {\n";
    for c in characters {
        content = content
            + "            "
            + block.as_upper_camel_case().as_str()
            + "::"
            + c.as_upper_camel_case().as_str()
            + " => \""
            + c.as_pretty_name().as_str()
            + "\",\n";
    }
    content = content + "        }\n" + "    }\n";
    // end impl
    content = content + "}\n";

    content
}

fn generate_block_files(
    blocks: &Vec<UnicodeBlock>,
    data: &UnicodeData,
    out_dir: &PathBuf,
) -> std::io::Result<()> {
    for block in blocks {
        let filename = block.as_snake_case() + ".rs";
        let file = PathBuf::from(out_dir).join(filename);
        let characters = characters_in_range(&block.range, data);
        let mut content = String::new();
        // constants
        content += generate_block_constants(&block, &characters).as_str();
        // enum
        content += generate_block_enum(&block, &characters).as_str();
        // Into<char>
        content += generate_block_into_char(&block, &characters).as_str();
        // TryFrom<char>
        content += generate_block_try_from_char(&block, &characters).as_str();
        // Into<u32>
        content += generate_block_into_u32(&block).as_str();
        // TryFrom<u32>
        content += generate_block_try_from_u32(&block).as_str();
        // Iterator
        content += generate_block_iterator(&block).as_str();
        // impl
        content += generate_block_enum_impl(&block, &characters).as_str();

        let mut file = File::create(file)?;
        file.write_all(&content.bytes().collect::<Vec<_>>())?;
    }
    Ok(())
}

fn generate_unicode_types(
    blocks: &UnicodeBlocks,
    data: &UnicodeData,
    out_dir: &PathBuf,
) -> std::io::Result<()> {
    create_dir_all(out_dir)?;
    generate_mod_rs(blocks, data, out_dir)?;
    generate_block_files(&blocks.0, data, out_dir)
}

fn main() {
    let options = Options::from_args();
    let blocks_file = options.ucd_dir.join(BLOCKS_FILE);
    let blocks = UnicodeBlocks::from_file(blocks_file).expect("Parse Error");
    let data_file = options.ucd_dir.join(UNICODE_DATA_FILE);
    let data = UnicodeData::from_file(data_file).expect("Parse Error");
    let out_dir = options.out_dir;
    generate_unicode_types(&blocks, &data, &out_dir).expect("Error generating code");
}
