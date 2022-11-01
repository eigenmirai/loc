// module `util`
use json;
use std::collections::HashMap;

#[allow(non_upper_case_globals)]
const json_file: &str = include_str!("lang.json");

pub struct LocData {
    pub lang: String,
    pub files: u32,
    pub blank: u32,
    pub comment: u32,
    pub code: u32
}

impl LocData {
    pub fn of(lang: String) -> LocData {
        LocData {
            lang,
            files: 0,
            blank: 0,
            comment: 0,
            code: 0
        }
    }
}

pub struct Args {
    pub flags: Vec<String>,
    // map contains the option and its value, if present
    pub long_options: HashMap<String, Option<String>>, // TwT
    pub target: Option<String>,
}

impl Args {
    pub fn has_flag(&self, flag: char) -> bool{
        self.flags.contains(&String::from(flag))
    }

    pub fn has_option(&self, option: &str) -> bool {
        self.long_options.contains_key(option)
    }

    pub fn has_flag_or_option(&self, flag: char, option: &str) -> bool{
        self.has_flag(flag) || self.has_option(option)
    }

    pub fn get_option_value(&self, option: &str) -> Option<String> {
        if !self.has_option(option) {
            return None;
        }
        self.long_options.get(option).unwrap().clone()
    }
}

// this is definitely the worst part of this program
// i am sorry
pub fn parse_args(raw_args: Vec<String>) -> Args {
    let mut flags: Vec<String> = Vec::new();
    let mut long_options: HashMap<String, Option<String>> = HashMap::new();
    let mut target: Option<String> = None;

    for arg in raw_args {
        if arg.starts_with("--") {
            let (opt, val) = match arg.split_once('=') {
                Some((a, b)) => (String::from(a), Some(String::from(b))),
                None => (arg, None)
            };
            long_options.insert(opt.replace("--", ""), val);
        } else if arg.starts_with("-") {
            for ch in arg.chars() {
                if ch == '-' {
                    continue;
                }
                flags.push(String::from(ch));
            }
        } else {
            target = Some(arg);
        }
    }
    
    Args {
        flags,
        long_options,
        target
    }
}

pub fn resolve_extension(ext: String, quick: &bool) -> String {
    if *quick {
        return ext;
    }
    let json = json::parse(json_file).unwrap();
    let value = &json[ext];

    if value.is_null() {
        String::from("Unknown")
    } else {
        value.to_string()
    }
}

