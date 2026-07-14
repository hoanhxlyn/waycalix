//! Pure date arithmetic and formatting. No GTK, no I/O — all functions here are
//! deterministic and unit-tested (see the `tests` module at the bottom).

use chrono::{Datelike, Days, NaiveDate};

// Date format used when copying the selected day to the clipboard.
// %d/%m/%Y renders 2026-07-10 as 10/07/2026 (Vietnamese localized order).
pub const COPY_FORMAT: &str = "%d/%m/%Y";

pub fn days_in_month(y: i32, m: u32) -> u32 {
    let (ny, nm) = if m == 12 { (y + 1, 1) } else { (y, m + 1) };
    let first = NaiveDate::from_ymd_opt(y, m, 1).unwrap();
    let next = NaiveDate::from_ymd_opt(ny, nm, 1).unwrap();
    next.signed_duration_since(first).num_days() as u32
}

/// Shift the selected date by whole months, clamping the day to the target
/// month's length (e.g. Jan 31 -> Feb 28) so the result is always valid.
pub fn shift_month(date: NaiveDate, delta: i32) -> NaiveDate {
    let total = date.year() * 12 + (date.month() as i32 - 1) + delta;
    let year = total.div_euclid(12);
    let month = total.rem_euclid(12) as u32 + 1;
    let day = date.day().min(days_in_month(year, month));
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

pub fn shift_days(date: NaiveDate, delta: i64) -> NaiveDate {
    if delta >= 0 {
        date.checked_add_days(Days::new(delta as u64)).unwrap_or(date)
    } else {
        date.checked_sub_days(Days::new((-delta) as u64)).unwrap_or(date)
    }
}

pub fn month_name(m: u32) -> &'static str {
    match m {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ymd(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn days_in_month_handles_leap_february() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2026, 2), 28);
        assert_eq!(days_in_month(2026, 12), 31);
    }

    #[test]
    fn shift_month_clamps_day_to_shorter_month() {
        // Jan 31 -> Feb has no 31st, so clamp to the 28th (non-leap year).
        assert_eq!(shift_month(ymd(2026, 1, 31), 1), ymd(2026, 2, 28));
    }

    #[test]
    fn shift_month_crosses_year_boundary() {
        assert_eq!(shift_month(ymd(2026, 12, 15), 1), ymd(2027, 1, 15));
        assert_eq!(shift_month(ymd(2026, 1, 15), -1), ymd(2025, 12, 15));
    }

    #[test]
    fn shift_days_crosses_month_boundary() {
        assert_eq!(shift_days(ymd(2026, 1, 31), 1), ymd(2026, 2, 1));
        assert_eq!(shift_days(ymd(2026, 3, 1), -1), ymd(2026, 2, 28));
        assert_eq!(shift_days(ymd(2026, 7, 10), 7), ymd(2026, 7, 17));
    }
}
