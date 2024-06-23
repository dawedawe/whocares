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

struct CareWeek {
    week: u32,
    caretaker: String,
    start_date: chrono::NaiveDate,
    end_date: chrono::NaiveDate,
}

fn get_config(path: &str) -> io::Result<Config> {
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

fn get_next_weeks(conf: Config, weeks: u32) -> Vec<CareWeek> {
    let len = conf.caretakers.len();
    let caretaker_idx = get_current_caretaker_idx(conf.startweek, len as u32);
    let mut caretaker_ordered = Vec::new();
    for i in caretaker_idx..(caretaker_idx + weeks as usize) {
        let idx = i % len;
        caretaker_ordered.push(conf.caretakers.get(idx).unwrap());
    }
    let current_week_number = chrono::Local::now().iso_week().week();
    caretaker_ordered
        .iter()
        .zip(current_week_number..)
        .map(|(caretaker, week)| {
            let start = chrono::NaiveDate::from_isoywd_opt(chrono::Local::now().year(), week, chrono::Weekday::Mon).unwrap();
            let end = chrono::NaiveDate::from_isoywd_opt(chrono::Local::now().year(), week, chrono::Weekday::Sun).unwrap();
            CareWeek {
                week: week,
                caretaker: caretaker.to_string(),
                start_date: start,
                end_date: end,
            }
        })
        .collect::<Vec<CareWeek>>()
}

fn main() {
    match get_config(PATH) {
        Ok(conf) => {
            let weeks = get_next_weeks(conf, 8);
            for week in weeks {
                println!("week #{} {} - {}: {}", week.week, week.start_date, week.end_date, week.caretaker);
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
        let result = get_config(PATH);
        assert!(result.is_ok());
    }

    #[test]
    fn get_current_caretaker_works() {
        let schedule = get_config(PATH).unwrap();
        let current_caretaker = get_current_caretaker(schedule.startweek, &schedule.caretakers);
        assert!(schedule.caretakers.contains(&current_caretaker));
    }
}
