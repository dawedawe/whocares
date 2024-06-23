use chrono::Datelike;
use serde::Deserialize;
use serde_json;
use std::fs::File;
use std::io::{self};

const PATH: &str = "./config.json";

#[derive(Deserialize)]
struct Config {
    startweek: u32,
    caretakers: Vec<String>,
}

fn get_schedule(path: &str) -> io::Result<Config> {
    if let Ok(file) = File::open(path) {
        let reader = io::BufReader::new(file);
        let schedule: Config = serde_json::from_reader(reader).unwrap();
        Ok(schedule)
    } else {
        Err(io::Error::new(io::ErrorKind::Other, "Failed to open file"))
    }
}

fn get_current_caretaker_idx(schedule_start_week: u32, caretaker_count: u32) -> usize {
    let week_number = chrono::Local::now().iso_week().week();
    let caretaker_idx = (week_number - schedule_start_week) % caretaker_count;
    caretaker_idx as usize
}

fn get_current_caretaker(schedule_start_week: u32, caretakers: &Vec<String>) -> String {
    let caretaker_idx = get_current_caretaker_idx(schedule_start_week, caretakers.len() as u32);
    caretakers.get(caretaker_idx).unwrap().to_string()
}

fn get_next_weeks_schedule(schedule_start_week: u32, caretakers: &Vec<String>) -> Vec<String> {
    let len = caretakers.len();
    let caretaker_idx = get_current_caretaker_idx(schedule_start_week, len as u32);
    let mut weeks = Vec::new();
    for i in caretaker_idx..(caretaker_idx + len) {
        let idx = i % len;
        weeks.push(caretakers.get(idx).unwrap());
    }
    let current_week_number = chrono::Local::now().iso_week().week();
    weeks
        .iter()
        .zip(current_week_number..)
        .map(|(caretaker, week)| format!("Week {}: {}", week, caretaker))
        .collect::<Vec<String>>()
}

fn main() {
    match get_schedule(PATH) {
        Ok(schedule) => {
            let weeks = get_next_weeks_schedule(schedule.startweek, &schedule.caretakers);
            for week in weeks {
                println!("{}", week);
            }
        }
        _ => panic!("Failed to open file"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialization_works() {
        let result = get_schedule(PATH);
        assert!(result.is_ok());
    }

    #[test]
    fn get_current_caretaker_works() {
        let schedule = get_schedule(PATH).unwrap();
        let current_caretaker = get_current_caretaker(schedule.startweek, &schedule.caretakers);
        assert!(schedule.caretakers.contains(&current_caretaker));
    }
}