//! Dates as CloudNode shows them, without a date library.

use std::time::{SystemTime, UNIX_EPOCH};

const SECONDS_PER_DAY: u64 = 86_400;

/// `time` as an ISO 8601 date in UTC (`2024-01-20`); `None` before 1970.
pub fn iso_date(time: SystemTime) -> Option<String> {
    let days = time.duration_since(UNIX_EPOCH).ok()?.as_secs() / SECONDS_PER_DAY;
    let (year, month, day) = civil_from_days(i64::try_from(days).ok()?);
    Some(format!("{year:04}-{month:02}-{day:02}"))
}

/// Days since 1970-01-01 to (year, month, day) in the proleptic Gregorian calendar: Howard
/// Hinnant's `civil_from_days`, exact for every day an `i64` of days can name within a year `i64`.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days.saturating_add(719_468);
    let era = z.div_euclid(146_097);
    let day_of_era = z.rem_euclid(146_097); // 0..=146_096
    let year_of_era = (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365; // 0..=399
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100); // 0..=365
    let month_index = (5 * day_of_year + 2) / 153; // 0..=11, March first
    let day = day_of_year - (153 * month_index + 2) / 5 + 1; // 1..=31
    let month = if month_index < 10 { month_index + 3 } else { month_index - 9 }; // 1..=12
    let year = year_of_era.saturating_add(era.saturating_mul(400)) + i64::from(month <= 2);
    (year, u32::try_from(month).unwrap_or(1), u32::try_from(day).unwrap_or(1))
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    fn day(days: u64) -> SystemTime {
        UNIX_EPOCH + Duration::from_secs(days * SECONDS_PER_DAY)
    }

    #[test]
    fn known_dates() {
        assert_eq!(iso_date(day(0)).as_deref(), Some("1970-01-01"));
        assert_eq!(iso_date(day(11_016)).as_deref(), Some("2000-02-29"));
        assert_eq!(iso_date(day(11_017)).as_deref(), Some("2000-03-01"));
        assert_eq!(iso_date(day(19_723)).as_deref(), Some("2024-01-01"));
        assert_eq!(iso_date(day(19_742)).as_deref(), Some("2024-01-20"));
    }

    #[test]
    fn the_time_of_day_does_not_matter() {
        let late = day(19_742) + Duration::from_secs(SECONDS_PER_DAY - 1);
        assert_eq!(iso_date(late).as_deref(), Some("2024-01-20"));
    }

    #[test]
    fn before_1970_has_no_date() {
        assert_eq!(iso_date(UNIX_EPOCH - Duration::from_secs(1)), None);
    }
}
