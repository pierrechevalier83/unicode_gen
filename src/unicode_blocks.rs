use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Range {
    pub begin: u32,
    pub end: u32,
}

#[derive(Debug)]
pub struct UnicodeBlock {
    pub range: Range,
    name: String,
}

impl UnicodeBlock {
    /// Making the assumption that the initial string is composed of upper camel case words only
    /// separated by spaces and dashes.
    pub fn as_upper_camel_case(&self) -> String {
        let mut upper_camel_case = self.name.clone();
        upper_camel_case.retain(|c| c != ' ' && c != '-');
        upper_camel_case
    }
    pub fn as_snake_case(&self) -> String {
        self.name.replace(' ', "_").replace('-', "_").to_lowercase()
    }
}

#[derive(Debug)]
pub struct UnicodeBlocks(pub Vec<UnicodeBlock>);

impl UnicodeBlocks {
    fn parse_block(line: &&str) -> Option<UnicodeBlock> {
        let tokens = line.split(';').collect::<Vec<_>>();
        if !tokens.len() == 2 {
            panic!("Unrecognized syntax in \"Blocks\" block line");
        }
        let range = tokens[0].split("..").collect::<Vec<_>>();
        if !range.len() == 2 {
            panic!("Unrecognized syntax in \"Blocks\" block line");
        }
        let range = Range {
            begin: u32::from_str_radix(range[0], 16).expect("Fail"),
            end: u32::from_str_radix(range[1], 16).expect("Fail"),
        };
        if char::try_from(range.begin).is_ok() {
            Some(UnicodeBlock {
                range,
                name: tokens[1].trim().to_string(),
            })
        } else {
            None
        }
    }
    /// Parse block lines from Blocks file
    fn parse_blocks(lines: &[&str]) -> Vec<UnicodeBlock> {
        lines.into_iter().flat_map(Self::parse_block).collect()
    }
    /// Parse the unicode blocks file into Self
    pub fn from_file(path: PathBuf) -> Result<Self, std::io::Error> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let blocks = contents
            .lines()
            .filter(|line| !line.is_empty())
            .filter(|line| !line.starts_with('#'))
            .collect::<Vec<_>>();
        Ok(UnicodeBlocks(Self::parse_blocks(&blocks)))
    }
}
