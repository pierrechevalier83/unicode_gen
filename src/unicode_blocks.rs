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
pub struct UnicodeBlocks {
    /// Each line of comments, stripped from the starting "# "
    pub comments: Vec<String>,
    pub blocks: Vec<UnicodeBlock>,
}

impl UnicodeBlocks {
    /// Parse comment lines from Blocks file
    /// Panic if the syntax isn't what we expect
    fn parse_comments(lines: &[&str]) -> Vec<String> {
        lines
            .iter()
            .filter(|line| !line.contains("EOF"))
            .map(|line| {
                let mut line = line.to_string();
                if line.remove(0) != '#' {
                    panic!("Unrecognized syntax in \"Blocks\" comment");
                }
                if line.is_empty() {
                    "".to_string()
                } else if line.remove(0) != ' ' {
                    panic!("Unrecognized syntax in \"Blocks\" comment");
                } else {
                    line
                }
            })
            .collect()
    }
    fn parse_block(line: &&str) -> UnicodeBlock {
        let tokens = line.split(';').collect::<Vec<_>>();
        if !tokens.len() == 2 {
            panic!("Unrecognized syntax in \"Blocks\" block line");
        }
        let range = tokens[0].split("..").collect::<Vec<_>>();
        if !range.len() == 2 {
            panic!("Unrecognized syntax in \"Blocks\" block line");
        }
        UnicodeBlock {
            range: Range {
                begin: u32::from_str_radix(range[0], 16).expect("Fail"),
                end: u32::from_str_radix(range[1], 16).expect("Fail"),
            },
            name: tokens[1].trim().to_string(),
        }
    }
    /// Parse block lines from Blocks file
    fn parse_blocks(lines: &[&str]) -> Vec<UnicodeBlock> {
        lines.into_iter().map(Self::parse_block).collect()
    }
    /// Parse the unicode blocks file into Self
    pub fn from_file(path: PathBuf) -> Result<Self, std::io::Error> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let (comments, blocks): (Vec<_>, Vec<_>) = contents
            .lines()
            .filter(|line| !line.is_empty())
            .partition(|line| line.starts_with('#'));
        Ok(UnicodeBlocks {
            comments: Self::parse_comments(&comments),
            blocks: Self::parse_blocks(&blocks),
        })
    }
}
