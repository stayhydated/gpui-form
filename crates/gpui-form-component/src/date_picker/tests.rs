use icu_locale_core::locale;
use jiff::civil::Date as CivilDate;
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
    let parsed = parse_form_date::<CivilDate>(date(2025, 1, 15)).expect("parsed date");

    assert_eq!(parsed, CivilDate::new(2025, 1, 15).unwrap());
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

#[cfg(feature = "component-shape")]
#[test]
fn date_picker_publishes_date_mcp_input() {
    use component_shape::{ComponentShapeMetadata, McpInputShape, McpPrimitiveKind};

    use super::DatePicker;

    assert_eq!(
        <DatePicker as ComponentShapeMetadata>::MCP_INPUT.input_shape(),
        McpInputShape::Scalar(McpPrimitiveKind::Date)
    );
}

#[gpui_kit::test]
fn calendar_events_keep_jiff_dates_and_partial_ranges(cx: &mut gpui_kit::TestAppContext) {
    use gpui_kit::{AppContext as _, Empty, VisualTestContext};

    use super::{CalendarDate, CalendarEvent, DatePickerState, DateRangePickerState};

    cx.update(gpui_kit::component::init);
    let window = cx.add_window(|_, _| Empty);
    let mut visual = VisualTestContext::from_window(window.into(), cx);
    let (single, range) = visual.update(|window, cx| {
        (
            cx.new(|cx| DatePickerState::new(window, cx)),
            cx.new(|cx| DateRangePickerState::new(window, cx)),
        )
    });
    let start = date(2024, 2, 29);
    let end = date(2024, 3, 1);

    visual.update(|window, cx| {
        single.update(cx, |state, cx| state.set_date(start, window, cx));
        range.update(cx, |state, cx| state.set_range(start, None, window, cx));
        assert_eq!(single.read(cx).date(), Some(start));
        assert_eq!(range.read(cx).range(), (Some(start), None));
    });
    visual.update(|_, cx| {
        let single_calendar = single.read(cx).calendar.clone();
        single_calendar.update(cx, |_, cx| {
            cx.emit(CalendarEvent::Selected(CalendarDate::Single(Some(end))));
        });
        let range_calendar = range.read(cx).calendar.clone();
        range_calendar.update(cx, |_, cx| {
            cx.emit(CalendarEvent::Selected(CalendarDate::Range(
                Some(start),
                Some(end),
            )));
        });
    });
    visual.update(|_, cx| {
        assert_eq!(single.read(cx).date(), Some(end));
        assert_eq!(range.read(cx).range(), (Some(start), Some(end)));
    });
}
