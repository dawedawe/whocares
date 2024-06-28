use chrono::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self};
use whocares::date_serializer;

const PATH: &str = "./config.json";

#[derive(Deserialize)]
struct Config {
    #[serde(with = "date_serializer")]
    startdate: chrono::NaiveDate,
    caretakers: Vec<String>,
    reschedule: Vec<(u32, String)>,
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

fn get_current_caretaker_idx(conf: &Config) -> usize {
    let start = conf.startdate;
    let current_date: chrono::NaiveDate = chrono::Local::now().date_naive();

    let diff = start
        .iter_weeks()
        .take_while(|w| w <= &current_date)
        .count()
        - 1;

    diff % conf.caretakers.len()
}

fn get_current_caretaker(conf: &Config) -> String {
    let caretaker_idx = get_current_caretaker_idx(conf);
    conf.caretakers.get(caretaker_idx).unwrap().to_string()
}

fn get_next_weeks(conf: &Config, weeks: u32) -> Vec<CareWeek> {
    let caretaker_idx = get_current_caretaker_idx(conf);
    let num_caretakers = conf.caretakers.len();
    let start_of_current_week = chrono::Local::now()
        .date_naive()
        .week(Weekday::Mon)
        .first_day();
    let rescheduled: HashMap<u32, String> = conf.reschedule.clone().into_iter().collect();

    start_of_current_week
        .iter_weeks()
        .zip(caretaker_idx..(caretaker_idx + weeks as usize))
        .map(|(d, i)| {
            let week_number: u32 = d.iso_week().week();
            let start_of_week = d;
            let end_of_week = start_of_week
                .checked_add_days(chrono::Days::new(6))
                .unwrap();

            let caretaker = match rescheduled.get(&week_number) {
                Some(rescheduled_caretaker) => rescheduled_caretaker,
                None => {
                    let idx = i % num_caretakers;
                    let regular_caretaker = conf.caretakers.get(idx).unwrap();
                    regular_caretaker
                }
            };

            CareWeek {
                week: week_number,
                caretaker: caretaker.clone(),
                start_date: start_of_week,
                end_date: end_of_week,
            }
        })
        .collect::<Vec<CareWeek>>()
}

fn main() {
    match get_config(PATH) {
        Ok(conf) => {
            let weeks = get_next_weeks(&conf, 4);
            for week in weeks {
                println!(
                    "week #{} {} - {}: {}",
                    week.week, week.start_date, week.end_date, week.caretaker
                );
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
        let config = get_config(PATH).unwrap();
        let current_caretaker = get_current_caretaker(&config);
        assert!(config.caretakers.contains(&current_caretaker));
    }

    #[test]
    fn get_next_weeks_across_years_works() {
        let config = get_config(PATH).unwrap();
        let weeks = get_next_weeks(&config, 100);
        assert!(weeks.len() == 100);
    }
}
