use chrono::NaiveDate;
use icu_locale_core::locale;
use jiff::civil::date;

use super::{DateDisplayStyle, format_display_date, format_display_range, parse_form_date};

#[test]
fn formats_dates_with_icu4x() {
    let formatted = format_display_date(
        date(2025, 1, 15),
        &locale!("en-US"),
        DateDisplayStyle::Medium,
    )
    .expect("formatted date");

    assert_eq!(formatted.to_string(), "Jan 15, 2025");
}

#[test]
fn parses_dates_through_jiff_display() {
    let parsed = parse_form_date::<NaiveDate>(date(2025, 1, 15)).expect("parsed date");

    assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
}

#[test]
fn formats_date_ranges_with_icu4x() {
    let formatted = format_display_range(
        Some(date(2025, 1, 15)),
        Some(date(2025, 1, 20)),
        &locale!("en-US"),
        DateDisplayStyle::Medium,
    )
    .expect("formatted date range");

    assert_eq!(formatted.to_string(), "Jan 15, 2025 - Jan 20, 2025");
}

#[test]
fn formats_partial_ranges_and_all_display_widths() {
    let value = date(2025, 1, 15);
    assert_eq!(
        format_display_range(
            Some(value),
            None,
            &locale!("en-US"),
            DateDisplayStyle::Short,
        )
        .unwrap()
        .to_string(),
        "1/15/25 - ..."
    );
    assert_eq!(
        format_display_range(None, Some(value), &locale!("en-US"), DateDisplayStyle::Long,)
            .unwrap()
            .to_string(),
        "... - January 15, 2025"
    );
    assert!(
        format_display_range(None, None, &locale!("en-US"), DateDisplayStyle::Medium).is_none()
    );
}
