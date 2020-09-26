/*
Partially borrowed from https://github.com/fdehau/tui-rs/blob/master/examples/util/mod.rs under MIT
May have some modifications for: App, TabsState, StatefulList
*/

use ansi_term::Colour::Red;
use csv::StringRecord;
use regex::Regex;
use serde::Deserialize;
use std::{
    error::Error,
    fs::{read_dir, File},
    io,
    io::prelude::*,
    path::Path,
    process::Command,
    result::Result,
};
use tui::{
    style::{Color, Style},
    text::{Span, Spans},
    widgets::ListState,
};
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

fn human_readable(value: u64, bytes: bool) -> String {
    let mut readable_value = value as f64;
    let sizes = ["K", "M", "G", "T", "Y", "Z", "E"]; // This lacks support for 128 bit systems ^__^
    let mut counter = 0_usize;

    while readable_value > 1024.0 && counter < sizes.len() {
        readable_value /= 1024.0;
        counter += 1_usize;
    }

    let suffix = if bytes { "B" } else { "" };
    if value < 1024 && suffix == "" {
        return format!("{}", value);
    } else if value < 1024 {
        return format!("{} {}", value, suffix);
    } else if readable_value.fract() == 0.0 {
        return format!("{:.0} {}{}", readable_value, sizes[counter], suffix);
    } else {
        return format!("{:.2} {}{}", readable_value, sizes[counter], suffix);
    }
}

pub fn check_user_quotas<'a>(record: &'a QuotaRecord, group: &'a str) -> Vec<Spans<'a>> {
    let mut results: Vec<Spans> = Vec::new();

    if record.block_hard < record.block_usage {
        results.push(Spans(vec![
            Span::styled("Warning: ", Style::default().fg(Color::Red)),
            Span::from(format!(
                "Hard block limit ({}) for {} by {} exceeded. Grace Period: {}",
                human_readable(record.block_hard, true),
                record.filesystem,
                group,
                record.block_grace
            )),
        ]));
    } else if record.block_soft < record.block_usage {
        results.push(Spans(vec![
            Span::styled("Warning: ", Style::default().fg(Color::Yellow)),
            Span::from(format!(
                "Soft block limit ({}) for {} by {} exceeded. Grace Period: {}",
                human_readable(record.block_soft, true),
                record.filesystem,
                group,
                record.block_grace
            )),
        ]));
    }

    if record.inode_hard < record.inode_usage {
        results.push(Spans(vec![
            Span::styled("Warning: ", Style::default().fg(Color::Red)),
            Span::from(format!(
                "Hard inode limit ({}) for {} by {} exceeded. Grace Period: {}",
                human_readable(record.inode_hard, false),
                record.filesystem,
                group,
                record.inode_grace
            )),
        ]));
    } else if record.inode_soft < record.inode_usage {
        results.push(Spans(vec![
            Span::styled("Warning: ", Style::default().fg(Color::Yellow)),
            Span::from(format!(
                "Soft inode limit ({}) for {} by {} exceeded. Grace Period: {}",
                human_readable(record.inode_soft, false),
                record.filesystem,
                group,
                record.inode_grace
            )),
        ]));
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
) -> Result<(String, Vec<Vec<String>>, Vec<QuotaRecord>), Box<dyn Error>> {
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
    let mut records: Vec<QuotaRecord> = Vec::new();

    // deserialize csv
    for result in rdr.deserialize() {
        match result {
            Ok(result) => {
                // this is used to ensure the structure is valid
                let record: QuotaRecord = result;
                records.push(record.clone());

                // this is hardcoded though still secure due to serde based deserializing
                // there's at the moment no way of just iterating over struct fields
                quotas.push(vec![
                    record.filesystem,
                    human_readable(record.block_usage, true),
                    human_readable(record.block_soft, true),
                    human_readable(record.block_hard, true),
                    record.block_grace,
                    human_readable(record.inode_usage, false),
                    human_readable(record.inode_soft, false),
                    human_readable(record.inode_hard, false),
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
    return Ok((timestamp, quotas, records));
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
