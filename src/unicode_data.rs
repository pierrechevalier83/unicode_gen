use std::collections::HashMap;
use std::convert::TryFrom;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct UnicodeCharacter {
    pub character: char,
    name: String,
}

impl UnicodeCharacter {
    // We assume a space-separated, all uppercase name
    pub fn as_upper_snake_case(&self) -> String {
        self.name.replace(' ', "_").replace('-', "_")
    }
    pub fn as_upper_camel_case(&self) -> String {
        let words = self.name.replace('-', " ");
        words
            .split(' ')
            .map(|word| {
                let mut word = word.to_string();
                if word.len() > 1 {
                    let (_initial, rest) = word.split_at_mut(1);
                    rest.make_ascii_lowercase();
                }
                word.chars().collect::<Vec<_>>()
            })
            .flatten()
            .collect()
    }
}

#[derive(Debug)]
pub struct UnicodeData(pub HashMap<u32, UnicodeCharacter>);

impl UnicodeData {
    pub fn from_file(path: PathBuf) -> Result<Self, std::io::Error> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(UnicodeData(
            contents
                .lines()
                .flat_map(|line| {
                    let tokens = line.split(';').collect::<Vec<_>>();
                    if !tokens.len() == 15 {
                        panic!("Expected 15 fields per character");
                    }
                    let index = u32::from_str_radix(tokens[0], 16).expect("Fail");
                    if let Ok(character) = char::try_from(index) {
                        let name = tokens[1].to_string();
                        Some((index, UnicodeCharacter { character, name }))
                    } else {
                        None
                    }
                })
                .collect(),
        ))
    }
}
