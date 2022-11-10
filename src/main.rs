mod util;

use util::*;

use std::collections::HashMap;
use std::env;
use std::os::unix::prelude::MetadataExt;
use std::path::Path;
use std::time::{Duration, SystemTime};
use std::{fs, io};

fn print_help(prog_name: &String) {
    println!("Usage: {} [OPTION]... [TARGET]\n\nOPTIONS:", prog_name);
    println!("    -h, --help         Shows this message");
    println!("    -t, --time         Times execution and calculates files and lines per ms");
    println!("    -c, --colors       Colors rows of the result table");
    println!("        --ignore=a,b   Ignores files with the specified extension. (separated by commas)");
    println!("Due to the shitty way this program parses command line args,\n you can put them in any order you want and put any amount of - in between the short args, \nand put the target file whereever you want to.");
    println!("For example, `{} -c-t--q ~/project/src` does the same as `{} -ctq ~/project/src`", prog_name, prog_name);
}

fn process_file(file: &Path, map: &mut HashMap<String, LocData>, ignore: &Vec<String>) -> bool {
    // get the file extension
    let extension: String = match file.extension() {
        Some(ext) => match ext.to_os_string().into_string() {
            Ok(string) => string,
            Err(_) => String::from("unknown"),
        },
        None => String::from("unknown"),
    };
    if ignore.contains(&extension) {
        return false;
    }

    // get language from extension and check if language is already in the hashmap
    let lang = extension;
    if !map.contains_key(&lang) {
        map.insert(lang.clone(), LocData::of(lang.clone()));
    }

    //read file and process
    let file_content = match fs::read_to_string(&file) {
        Ok(file) => file,
        Err(_) => return false,
    };

    // get a mutable reference to the data for this language/extension
    let obj = match map.get_mut(&lang) {
        Some(obj) => obj,
        None => return false,
    };
    obj.files += 1;
    for line in file_content.lines() {
        if line.trim().is_empty() {
            obj.blank += 1;
        } else if line.trim().starts_with("//") || line.trim().starts_with("#") {
            obj.comment += 1;
        } else {
            obj.code += 1;
        }
    }
    return true;
}

#[allow(unused_assignments)]
#[allow(unused_variables)]
fn scan_dir_recursive(path: &Path, ignore: &Vec<String>) -> io::Result<HashMap<String, LocData>> {
    let mut data: HashMap<String, LocData> = HashMap::new();
    let mut skipped: u32 = 0;

    if path.is_dir() {
        for entry in path.read_dir().expect("failed to read dir") {
            if let Ok(entry) = entry {
                if entry.path().is_dir() {
                    let subdir_data = match scan_dir_recursive(&entry.path(), ignore) {
                        Ok(subdir_data) => subdir_data,
                        Err(_) => HashMap::new(),
                    };
                    for (key, value) in subdir_data {
                        if !data.contains_key(&key) {
                            data.insert(key, value);
                            continue;
                        }
                        if let Some(obj) = data.get_mut(&key) {
                            obj.files += value.files;
                            obj.blank += value.blank;
                            obj.comment += value.comment;
                            obj.code += value.code;
                        }
                    }
                    continue;
                }

                let file = entry.path();

                // skip if file is in binary
                if file.metadata().unwrap().mode() & 0o111 != 0 {
                    skipped += 1;
                    continue;
                }
                if !process_file(&file, &mut data, ignore) {
                    skipped += 1;
                }
            }
        }
    } else {
        if !process_file(&path, &mut data, ignore){
            skipped += 1;
        }
    }
    // eprintln!("skipped {} files either because they were not text files or because there was an error.", skipped);
    Ok(data)
}

fn main() -> () {
    let args = parse_args(env::args().collect());

    if args.has_flag_or_option('h', "help") {
        print_help(&String::from("rs-loc"));
        return;
    }

    let ignored: Vec<String> = match args.get_option_value("ignore") {
        Some(value) => value.split(',').map(|e| String::from(e)).collect(),
        None => Vec::new()
    };

    let file_name = match &args.target {
        Some(file_name) => file_name,
        None => panic!("no file provided"),
    };

    let path = Path::new(&file_name);
    let start = SystemTime::now();
    
    let data = match scan_dir_recursive(&path, &ignored) {
        Ok(subdir_data) => subdir_data,
        Err(_) => panic!("sob"),
    };

    let elapsed = match start.elapsed() {
        Ok(time) => time,
        Err(_) => Duration::from_secs(1),
    };
    let mut sum_files: u32 = 0;
    let mut sum_lines: u32 = 0;
    for (_, v) in &data {
        sum_files += v.files;
        sum_lines += v.code + v.blank + v.comment;
    }
    if args.has_flag_or_option('t', "time") {
        println!(
            "t={:?} ({} files/ms, {} L/ms)",
            elapsed,
            (((sum_files as f64) / elapsed.as_nanos() as f64) * 1e9).round() / 1e3,
            (((sum_lines as f64) / elapsed.as_nanos() as f64) * 1e9).round() / 1e3
        );
    }
    let colors = args.has_flag_or_option('c', "color");
    print_loc_stats(data, colors);
}

fn print_loc_stats(data: HashMap<String, LocData>, colors: bool) {
    let color1 = "\x1b[38;2;203;166;247m";
    let color2 = "\x1b[38;2;255;175;196m";
    let reset = "\x1b[0m";

    let mut sum = LocData::of(String::from("SUM"));
    for (_, v) in &data {
        sum.files += v.files;
        sum.blank += v.blank;
        sum.comment += v.comment;
        sum.code += v.code;
    }

    // ┌ ┐└ ┘ ├ ┤─ │
    println!("┌─<LOC STATS>───────────────────────────────────────────────────────────────────────────┐");
    println!("│ Language               files          blank         comment         code          %   │");
    println!("├───────────────────────────────────────────────────────────────────────────────────────┤");
    let mut i: u32 = 0;
    for (_, v) in &data {
        let color = if colors { if i % 2 == 0 { color1 } else { color2 }} else { "" };
        let ratio = ((v.code as f32/ sum.code as f32) * 1e3).round() / 10f32;
        println!(
            "│{}{: <24}{: <15}{: <14}{: <16}{: <14}{: <4}{}│",
            color, v.lang, v.files, v.blank, v.comment, v.code, ratio, reset
        );
        i += 1;
    }
    println!("├───────────────────────────────────────────────────────────────────────────────────────┤");
    println!(
            "│{: <24}{: <15}{: <14}{: <16}{: <18}│",
            sum.lang, sum.files, sum.blank, sum.comment, sum.code
        );
    println!("└───────────────────────────────────────────────────────────────────────────────────────┘");

}

