use chrono::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{self};
use whocares::date_serializer;

const PATH: &str = "./config.json";

#[derive(Deserialize)]
struct Config {
    #[serde(with = "date_serializer")]
    startdate: chrono::NaiveDate,
    caretakers: Vec<String>,
    reschedule: HashMap<String, String>,
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

    start_of_current_week
        .iter_weeks()
        .zip(caretaker_idx..(caretaker_idx + weeks as usize))
        .map(|(d, i)| {
            let week_number: u32 = d.iso_week().week();
            let start_of_week = d;
            let end_of_week = start_of_week
                .checked_add_days(chrono::Days::new(6))
                .unwrap();

            let caretaker =
                match &conf
                    .reschedule
                    .get(&format!("{}-{}", d.year_ce().1, week_number))
                {
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
    let weeks_to_preview = if env::args().len() == 2 {
        let arg: Vec<String> = env::args().into_iter().collect();

        match arg[1].parse::<u32>() {
            Ok(n) => n,
            Err(e) => panic!("{e}"),
        }
    } else {
        4
    };

    match get_config(PATH) {
        Ok(conf) => {
            let weeks = get_next_weeks(&conf, weeks_to_preview);
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
    use std::str::FromStr;

    #[test]
    fn deserialization_works() {
        let result = get_config(PATH);
        assert!(result.is_ok());
        let config = result.unwrap();
        assert!(config.caretakers.len() == 4);
        assert!(config.startdate == NaiveDate::from_str("2024-05-27").unwrap());
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

    #[test]
    fn reschedule_works() {
        let current_week = chrono::Local::now().date_naive().iso_week().week();
        let current_year = chrono::Local::now().date_naive().year_ce().1;
        let config = Config {
            caretakers: vec!["A".to_string(), "B".to_string(), "C".to_string()],
            startdate: NaiveDate::from_str("2024-01-01").unwrap(),
            reschedule: HashMap::from([
                (
                    format!("{}-{}", current_year, current_week),
                    "C".to_string(),
                ),
                (
                    format!("{}-{}", current_year, current_week + 1),
                    "B".to_string(),
                ),
                (
                    format!("{}-{}", current_year, current_week + 2),
                    "A".to_string(),
                ),
            ]),
        };

        let weeks = get_next_weeks(&config, 3);
        assert!(weeks.len() == 3);
        assert!(weeks[0].caretaker == "C");
        assert!(weeks[1].caretaker == "B");
        assert!(weeks[2].caretaker == "A");
    }
}
