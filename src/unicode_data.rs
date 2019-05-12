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
    fn as_upper_snake_case(&self) -> String {
        self.name.replace(' ', "_").replace('-', "_DASH_")
    }
    // We assume a space-separated, all uppercase name
    pub fn as_const_name(&self, mod_name: &str) -> String {
        // Don't repeat the mod name in the const name
        self.as_upper_snake_case()
            .replace((mod_name.to_uppercase() + "_").as_str(), "")
    }
    fn as_upper_camel_case(&self) -> String {
        let words = self.name.replace('-', " Dash ");
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
    pub fn as_enum_variant(&self, enum_name: &str) -> String {
        // Don't repeat the enum name in the variant
        self.as_upper_camel_case().replace(enum_name, "")
    }
    pub fn as_pretty_name(&self) -> String {
        self.name.to_lowercase()
    }
    pub fn printable_character(&self) -> String {
        match self.character {
            '\t' => "\\t".to_string(),
            '\n' => "\\n".to_string(),
            '\r' => "\\r".to_string(),
            '\'' => "\\'".to_string(),
            '\\' => "\\\\".to_string(),
            _ => self.character.to_string(),
        }
    }
}

#[derive(Debug)]
pub struct UnicodeData(pub HashMap<u32, UnicodeCharacter>);

impl UnicodeData {
    fn from_line(line: &str) -> Option<(u32, UnicodeCharacter)> {
        let tokens = line.split(';').collect::<Vec<_>>();
        if !tokens.len() == 15 {
            panic!("Expected 15 fields per character");
        }
        let index = u32::from_str_radix(tokens[0], 16).expect("Fail");
        if let Ok(character) = char::try_from(index) {
            let mut name = tokens[1].to_string();
            if name == "<control>" {
                name = tokens[11].to_string();
                if name == "" {
                    name = String::from("CONTROL ") + tokens[0];
                }
            } else if name.starts_with("<") {
                name = name.replace('_', " ");
                name.retain(|c| c != ',' && c != '<' && c != '>');
                name = name.to_uppercase()
            } else if name.starts_with("BOX DRAWINGS") {
                // The raw data is inconsitent as the block is called
                // Box Drawing, but the characters are called BOX DRAWINGS...
                // Go for consistency over perfect accuracy to statisfy my OCD
                name = name.replace("BOX DRAWINGS", "BOX DRAWING");
            }
            Some((index, UnicodeCharacter { character, name }))
        } else {
            None
        }
    }
    pub fn from_file(path: PathBuf) -> Result<Self, std::io::Error> {
        let mut file = File::open(&path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(UnicodeData(
            contents
                .lines()
                .flat_map(|line| Self::from_line(line))
                .collect(),
        ))
    }
}
