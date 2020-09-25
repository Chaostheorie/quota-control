/*
Partially borrowed from https://github.com/fdehau/tui-rs/blob/master/examples/util/mod.rs under MIT
May have some modifications for: App, TabsState, StatefulList
*/

use ansi_term::Colour::{Red, Yellow};
use csv::StringRecord;
use regex::Regex;
use serde::Deserialize;
use std::fs::{read_dir, File};
use std::io::prelude::*;
use std::{error::Error, io, path::Path, process::Command, result::Result};
use tui::widgets::ListState;
use users::{get_current_gid, get_current_username, get_user_groups};

// Structures
pub struct App {
    pub items: StatefulList<String>,
    pub tabs: TabsState,
    pub action: ActionState,
}

pub struct TabsState {
    pub titles: Vec<&'static str>,
    pub index: usize,
}

pub struct ActionState {
    pub state: TabsState,
    pub is_visible: bool,
}

pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QuotaRecord {
    filesystem: String,
    block_usage: u64, // Sometimes appears as 0 and breaks i64
    block_soft: u64,
    block_hard: u64,
    block_grace: String, // grace is sometimes saved as 'none'
    inode_usage: u64,    // Sometimes appears as 0 and breaks i64
    inode_soft: u64,
    inode_hard: u64,
    inode_grace: String, // same here
}

// Implementations
impl ActionState {
    pub fn new(titles: Vec<&'static str>) -> ActionState {
        return ActionState {
            state: TabsState {
                titles,
                index: 0_usize,
            },
            is_visible: false,
        };
    }
}

impl<T> StatefulList<T> {
    pub fn new(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn select(&mut self, state: usize) {
        self.state.select(Some(state));
    }
}

impl TabsState {
    pub fn new(titles: Vec<&'static str>) -> TabsState {
        TabsState { titles, index: 0 }
    }
    pub fn next(&mut self) {
        self.index = (self.index + 1) % self.titles.len();
    }
}

// functions
pub fn exit(code: i32) {
    Command::new("clear").output().expect("Error");
    std::process::exit(code);
}

fn check_user_quotas(record: QuotaRecord, group: &str) -> Vec<String> {
    let prefix = Yellow.paint("Warning:");
    let mut results: Vec<String> = Vec::new();

    if record.block_hard < record.block_usage {
        results.push(format!(
            "{} Hard block limit ({}) for {} by {} exceeded. Grace Period: {}",
            &prefix, &record.block_hard, &record.filesystem, group, &record.block_grace
        ));
    } else if record.block_soft < record.block_usage {
        results.push(format!(
            "{} Soft block limit ({}) for {} by {} exceeded. Grace Period: {}",
            &prefix, &record.block_soft, &record.filesystem, group, &record.block_grace
        ));
    }

    if record.inode_hard < record.inode_usage {
        results.push(format!(
            "{} Hard inode limit ({}) for {} by {} exceeded. Grace Period: {}",
            &prefix, &record.inode_hard, &record.filesystem, group, &record.inode_grace
        ));
    } else if record.inode_soft < record.inode_usage {
        results.push(format!(
            "{} Soft inode limit ({}) for {} by {} exceeded. Grace Period: {}",
            &prefix, &record.inode_soft, &record.filesystem, group, &record.inode_grace
        ));
    }

    return results;
}

pub fn verify_privileges() -> bool {
    let admin = Regex::new(r"^(root|[bghz]z.*)").unwrap();
    for group in get_user_groups(
        &get_current_username().unwrap().to_owned(),
        get_current_gid(),
    )
    .expect("Error looking up groups")
    {
        if admin.is_match(&group.name().to_str().unwrap().to_string()) {
            return true;
        }
    }
    return false;
}

pub fn load_record(
    headers: &Vec<&str>,
    file: &str,
) -> Result<(String, Vec<Vec<String>>), Box<dyn Error>> {
    // assembling file path
    let file_path = format!("/home/quotas/{}.quota", file);

    // assembling reader
    let file = File::open(&file_path).expect("Something went wrong reading the quota file");
    let mut reader = io::BufReader::new(file);
    let mut timestamp = String::new(); // timestamps are not valid csv
    let mut _formats = String::new(); // for some reason the headers are also not properly formatted
    let _ = reader.read_line(&mut timestamp)?; // dispose of lengths
    let _ = reader.read_line(&mut _formats)?; // dispose of lengths
    let mut rdr = csv::ReaderBuilder::new().from_reader(reader);
    rdr.set_headers(StringRecord::from(headers.clone()));
    let mut quotas: Vec<Vec<String>> = Vec::new();

    // deserialize csv
    for result in rdr.deserialize() {
        match result {
            Ok(result) => {
                // this is used to ensure the structure is valid
                let record: QuotaRecord = result;

                // this is hardcoded though still secure due to serde based deserializing
                // there's at the moment no way of just iterating over struct fields
                quotas.push(vec![
                    record.filesystem,
                    record.block_usage.to_string(),
                    record.block_soft.to_string(),
                    record.block_hard.to_string(),
                    record.block_grace,
                    record.inode_usage.to_string(),
                    record.inode_soft.to_string(),
                    record.inode_hard.to_string(),
                    record.inode_grace,
                ]);
            }
            Err(err) => {
                println!(
                    "{} reading CSV from {}: {}",
                    Red.paint("Error:"),
                    &file_path,
                    err
                );
            }
        }
    }

    // return values
    return Ok((timestamp, quotas));
}

pub fn get_groups() -> Result<Vec<String>, Box<dyn Error>> {
    let dir = Path::new("/home/quotas");
    let re = Regex::new(r".*\.quota").unwrap();
    let ex = Regex::new(r"\.quota$").unwrap();
    let mut groups: Vec<String> = Vec::new();
    if dir.exists() && dir.is_dir() {
        for entry in read_dir(dir)? {
            let entry = entry?;
            let entry_path = entry
                .file_name()
                .into_string()
                .expect("Path not convertible");
            if re.is_match(&entry_path) {
                groups.push(ex.replace(&entry_path, "").into_owned());
            }
        }
    } else {
        println!(
            "{} /home/quotas doesn't exist or is not a valid folder",
            Red.paint("Error:")
        );
        exit(3);
    }
    Ok(groups)
}
