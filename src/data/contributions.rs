use crate::api::graphql::ContributionDay;
use chrono::NaiveDate;
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct StreakInfo {
    pub current_streak: u32,
    pub current_start: Option<String>,
    pub current_end: Option<String>,
    pub longest_streak: u32,
    pub longest_start: Option<String>,
    pub longest_end: Option<String>,
}

pub fn calculate_streaks(days: &[ContributionDay]) -> StreakInfo {
    if days.is_empty() {
        return StreakInfo {
            current_streak: 0,
            current_start: None,
            current_end: None,
            longest_streak: 0,
            longest_start: None,
            longest_end: None,
        };
    }

    let mut parsed: Vec<(NaiveDate, u32)> = days
        .iter()
        .filter_map(|d| {
            NaiveDate::parse_from_str(&d.date, "%Y-%m-%d")
                .ok()
                .map(|date| (date, d.count))
        })
        .collect();

    parsed.sort_by_key(|(date, _)| *date);

    let today = chrono::Local::now().date_naive();

    let mut longest_streak = 0u32;
    let mut longest_start: Option<NaiveDate> = None;
    let mut longest_end: Option<NaiveDate> = None;

    let mut current_run = 0u32;
    let mut current_run_start: Option<NaiveDate> = None;

    let mut final_streak = 0u32;
    let mut final_start: Option<NaiveDate> = None;
    let mut final_end: Option<NaiveDate> = None;

    for i in 0..parsed.len() {
        let (date, count) = parsed[i];

        if count > 0 {
            if current_run == 0 {
                current_run_start = Some(date);
            }
            current_run += 1;
        } else {
            if current_run > longest_streak {
                longest_streak = current_run;
                longest_start = current_run_start;
                longest_end = Some(parsed[i - 1].0);
            }
            current_run = 0;
            current_run_start = None;
        }
    }

    // Handle final run
    if current_run > 0 {
        let last_date = parsed.last().unwrap().0;
        if current_run > longest_streak {
            longest_streak = current_run;
            longest_start = current_run_start;
            longest_end = Some(last_date);
        }

        // Current streak: the run must include today or yesterday
        let diff = (today - last_date).num_days();
        if diff <= 1 {
            final_streak = current_run;
            final_start = current_run_start;
            final_end = Some(last_date);
        }
    }

    StreakInfo {
        current_streak: final_streak,
        current_start: final_start.map(|d| d.format("%b %-d").to_string()),
        current_end: final_end.map(|d| d.format("%b %-d").to_string()),
        longest_streak,
        longest_start: longest_start.map(|d| d.format("%b %-d").to_string()),
        longest_end: longest_end.map(|d| d.format("%b %-d").to_string()),
    }
}
