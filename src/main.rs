use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use structopt::StructOpt;

const BLOCKS_FILE: &str = "Blocks.txt";
const UNICODE_DATA_FILE: &str = "UnicodeData.txt";

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

#[derive(Debug)]
struct Range {
    begin: u32,
    end: u32,
}

#[derive(Debug)]
struct UnicodeBlock {
    range: Range,
    name: String,
}

#[derive(Debug)]
struct UnicodeBlocks {
    /// Each line of comments, stripped from the starting "# "
    comments: Vec<String>,
    blocks: Vec<UnicodeBlock>,
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
                println!("line: \"{}\"", line);
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
    fn from_file(path: PathBuf) -> Result<Self, std::io::Error> {
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

fn main() {
    let options = Options::from_args();
    let blocks_file = options.ucd_dir.join(BLOCKS_FILE);
    let blocks = UnicodeBlocks::from_file(blocks_file);
    println!("{:#?}", blocks);
}
