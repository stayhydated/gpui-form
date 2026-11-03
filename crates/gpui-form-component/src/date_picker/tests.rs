use chrono::NaiveDate;
use gpui_kit::AppContext as _;
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

struct KeyboardHarness {
    single: gpui_kit::Entity<super::DatePickerState>,
    range: gpui_kit::Entity<super::DateRangePickerState>,
    single_events: Vec<Option<jiff::civil::Date>>,
    range_events: Vec<(Option<jiff::civil::Date>, Option<jiff::civil::Date>)>,
    disabled: bool,
    cleanable: bool,
    escaped: usize,
    _subscriptions: Vec<gpui_kit::Subscription>,
}

impl KeyboardHarness {
    fn new(window: &mut gpui_kit::Window, cx: &mut gpui_kit::Context<Self>) -> Self {
        use gpui_kit::AppContext as _;
        let single = cx.new(|cx| {
            let mut state = super::DatePickerState::new(window, cx);
            state.set_date(date(2026, 11, 26), window, cx);
            state
        });
        let range = cx.new(|cx| {
            let mut state = super::DateRangePickerState::new(window, cx);
            state.set_range(date(2026, 1, 31), date(2026, 2, 3), window, cx);
            state
        });
        let subscriptions = vec![
            cx.subscribe_in(&single, window, |this, _, event, _, _| {
                let super::DatePickerEvent::Change(value) = event;
                this.single_events.push(*value);
            }),
            cx.subscribe_in(&range, window, |this, _, event, _, _| {
                let super::DateRangePickerEvent::Change(start, end) = event;
                this.range_events.push((*start, *end));
            }),
        ];
        Self {
            single,
            range,
            single_events: vec![],
            range_events: vec![],
            disabled: false,
            cleanable: false,
            escaped: 0,
            _subscriptions: subscriptions,
        }
    }
}

impl gpui_kit::Render for KeyboardHarness {
    fn render(
        &mut self,
        _window: &mut gpui_kit::Window,
        cx: &mut gpui_kit::Context<Self>,
    ) -> impl gpui_kit::IntoElement {
        use gpui_kit::component::Disableable as _;
        use gpui_kit::{InteractiveElement as _, ParentElement as _, Styled as _};
        gpui_kit::div()
            .size_full()
            .on_key_down(cx.listener(|this, event: &gpui_kit::KeyDownEvent, _, _| {
                if event.keystroke.key == "escape" {
                    this.escaped += 1;
                }
            }))
            .child(
                super::DatePicker::new(&self.single)
                    .disabled(self.disabled)
                    .cleanable(self.cleanable),
            )
            .child(
                super::DateRangePicker::new(&self.range)
                    .disabled(self.disabled)
                    .cleanable(self.cleanable),
            )
    }
}

fn keyboard_harness(
    cx: &mut gpui_kit::TestAppContext,
) -> (
    gpui_kit::WindowHandle<gpui_kit::component::Root>,
    gpui_kit::Entity<KeyboardHarness>,
) {
    cx.update(gpui_kit::init);
    cx.update(|cx| gpui_es_fluent::init_with_language(cx, unic_langid::langid!("en")).unwrap());
    let mut harness = None;
    let handle = cx.open_window(
        gpui_kit::size(gpui_kit::px(800.), gpui_kit::px(700.)),
        |window, cx| {
            let view = cx.new(|cx| KeyboardHarness::new(window, cx));
            harness = Some(view.clone());
            gpui_kit::component::Root::new(view, window, cx)
        },
    );
    cx.run_until_parked();
    (handle, harness.unwrap())
}

#[gpui_kit::test]
fn keyboard_date_picker_previews_commits_and_cancels_without_early_events(
    cx: &mut gpui_kit::TestAppContext,
) {
    use gpui_kit::{Focusable as _, test::TestWindowExt as _};
    let (handle, view) = keyboard_harness(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).single.clone();
        state.read(cx).focus_handle(cx).focus(window, cx);
        window.render_frame(cx);
        window.press("enter", cx);
        assert!(
            state.read(cx).open,
            "Enter must open the focused date picker"
        );
        assert_eq!(state.read(cx).date(), Some(date(2026, 11, 26)));
        window.press("right", cx);
        window.press("right", cx);
        window.press("right", cx);
        window.press("pagedown", cx);
        window.press("home", cx);
        assert_eq!(state.read(cx).date(), Some(date(2026, 11, 26)));
        assert!(view.read(cx).single_events.is_empty());
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).single.clone();
        assert_eq!(state.read(cx).date(), Some(date(2026, 12, 27)));
        assert!(!state.read(cx).open);
        assert_eq!(view.read(cx).single_events, [Some(date(2026, 12, 27))]);
        window.render_frame(cx);
        window.press("space", cx);
        assert!(state.read(cx).open);
        window.press("left", cx);
        window.press("escape", cx);
        assert!(!state.read(cx).open);
        assert_eq!(
            view.read(cx).escaped,
            0,
            "Popup Escape must not dismiss its parent"
        );
        assert_eq!(state.read(cx).date(), Some(date(2026, 12, 27)));
        window.press("escape", cx);
        assert_eq!(
            view.read(cx).escaped,
            1,
            "Closed picker must propagate Escape"
        );
        window.press("enter", cx);
        window.press("tab", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, _, cx| {
        assert!(
            !view.read(cx).single.read(cx).open,
            "Tab dismisses without committing a preview"
        );
        assert_eq!(view.read(cx).single_events, [Some(date(2026, 12, 27))]);
    })
    .unwrap();
}

#[gpui_kit::test]
fn keyboard_range_picker_requires_complete_selection_and_cancel_restores_anchor(
    cx: &mut gpui_kit::TestAppContext,
) {
    use gpui_kit::{Focusable as _, test::TestWindowExt as _};
    let (handle, view) = keyboard_harness(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).range.clone();
        state.read(cx).focus_handle(cx).focus(window, cx);
        window.render_frame(cx);
        window.press("space", cx);
        assert!(state.read(cx).open);
        window.press("left", cx);
        window.press("left", cx);
        window.press("enter", cx);
        assert!(
            view.read(cx).range_events.is_empty(),
            "A range start alone is not a changed range"
        );
        assert!(state.read(cx).open);
        window.press("right", cx);
        window.press("right", cx);
        window.press("right", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).range.clone();
        assert_eq!(
            state.read(cx).range(),
            (Some(date(2026, 2, 1)), Some(date(2026, 2, 4)))
        );
        assert_eq!(view.read(cx).range_events.len(), 1);
        assert!(!state.read(cx).open);
        window.render_frame(cx);
        window.press("enter", cx);
        window.press("left", cx);
        window.press("enter", cx);
        window.press("escape", cx);
        window.press("enter", cx);
        window.press("right", cx);
        window.press("enter", cx);
        window.press("right", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, _, cx| {
        assert_eq!(
            view.read(cx).range_events,
            [
                (Some(date(2026, 2, 1)), Some(date(2026, 2, 4))),
                (Some(date(2026, 2, 5)), Some(date(2026, 2, 6))),
            ]
        );
    })
    .unwrap();
}

#[gpui_kit::test]
fn keyboard_date_picker_respects_disabled_cleanable_and_modifier_gates(
    cx: &mut gpui_kit::TestAppContext,
) {
    use gpui_kit::{Focusable as _, test::TestWindowExt as _};
    let (handle, view) = keyboard_harness(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).single.clone();
        state.read(cx).focus_handle(cx).focus(window, cx);
        window.render_frame(cx);
        window.press("ctrl-enter", cx);
        assert!(!state.read(cx).open);
        window.press("delete", cx);
        assert_eq!(state.read(cx).date(), Some(date(2026, 11, 26)));
        view.update(cx, |h, cx| {
            h.disabled = true;
            h.cleanable = true;
            cx.notify();
        });
        window.render_frame(cx);
        window.press("enter", cx);
        window.press("backspace", cx);
        assert!(!state.read(cx).open);
        assert_eq!(state.read(cx).date(), Some(date(2026, 11, 26)));
        assert!(view.read(cx).single_events.is_empty());
        view.update(cx, |h, cx| {
            h.disabled = false;
            cx.notify();
        });
        state.read(cx).focus_handle(cx).focus(window, cx);
        window.render_frame(cx);
        window.press("delete", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).single.clone();
        assert_eq!(state.read(cx).date(), None);
        assert_eq!(view.read(cx).single_events, [None]);
        state.update(cx, |state, cx| {
            state.set_date(date(2028, 2, 29), window, cx)
        });
        window.render_frame(cx);
        window.press("enter", cx);
        window.press("pageup", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, _, cx| {
        assert_eq!(
            view.read(cx).single.read(cx).date(),
            Some(date(2028, 1, 29))
        );
        assert_eq!(view.read(cx).single_events, [None, Some(date(2028, 1, 29))]);
    })
    .unwrap();
}

#[gpui_kit::test]
fn keyboard_calendar_obeys_locale_month_clamping_and_civil_bounds(
    cx: &mut gpui_kit::TestAppContext,
) {
    use gpui_kit::{Focusable as _, test::TestWindowExt as _};
    let (handle, view) = keyboard_harness(cx);
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).single.clone();
        state.update(cx, |state, cx| {
            state.display_locale = Some(locale!("fr-FR"));
            state.set_date(date(2026, 11, 26), window, cx);
        });
        state.read(cx).focus_handle(cx).focus(window, cx);
        window.press("enter", cx);
        window.press("home", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).single.clone();
        assert_eq!(state.read(cx).date(), Some(date(2026, 11, 23)));
        state.update(cx, |state, cx| {
            state.set_date(date(2028, 2, 29), window, cx)
        });
        window.press("enter", cx);
        window.press("shift-pageup", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).single.clone();
        assert_eq!(state.read(cx).date(), Some(date(2027, 2, 28)));
        state.update(cx, |state, cx| {
            state.set_date(date(9999, 12, 31), window, cx)
        });
        window.press("enter", cx);
        window.press("right", cx);
        window.press("pagedown", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, window, cx| {
        let state = view.read(cx).single.clone();
        assert_eq!(state.read(cx).date(), Some(date(9999, 12, 31)));
        state.update(cx, |state, cx| {
            state.set_date(date(-9999, 1, 1), window, cx)
        });
        window.press("enter", cx);
        window.press("left", cx);
        window.press("pageup", cx);
        window.press("enter", cx);
    })
    .unwrap();
    cx.run_until_parked();
    cx.update_window(handle.into(), |_, _, cx| {
        assert_eq!(
            view.read(cx).single.read(cx).date(),
            Some(date(-9999, 1, 1))
        );
        assert_eq!(
            view.read(cx).single_events.len(),
            4,
            "Only the four confirmations emit"
        );
    })
    .unwrap();
}
