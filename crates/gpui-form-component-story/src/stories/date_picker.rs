use es_fluent::FluentMessage;
use gpui_kit::component::form::v_form;
use gpui_kit::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div,
};
use jiff::civil::{Date as JiffDate, date};

use gpui_form_component::date_picker::{
    DateDisplayStyle, DatePicker, DatePickerEvent, DatePickerState, DateRangePicker,
    DateRangePickerEvent, DateRangePickerState,
};

use crate::i18n::DatePickerComponentText;

use super::common::{story_field, story_panel};

fn localize(cx: &impl std::borrow::Borrow<App>, message: &impl FluentMessage) -> String {
    crate::i18n::localize_message(cx, message)
}

#[gpui_storybook::story]
#[derive(gpui_storybook::StoryControls)]
pub struct DatePickerStory {
    range_picker: Entity<DateRangePickerState>,
    long_picker: Entity<DatePickerState>,
    compact_picker: Entity<DatePickerState>,
    last_change: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl gpui_storybook::Story for DatePickerStory {
    fn title(_: &gpui_kit::App) -> String {
        "Date Picker".into()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for DatePickerStory {
    fn focus_handle(&self, cx: &App) -> gpui_kit::FocusHandle {
        self.range_picker.read(cx).focus_handle(cx)
    }
}

impl DatePickerStory {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let range_picker = cx.new(|cx| DateRangePickerState::new(window, cx));
        let long_picker = cx.new(|cx| {
            let mut state = DatePickerState::new(window, cx).display_style(DateDisplayStyle::Long);
            state.set_date(Some(date(2026, 4, 20)), window, cx);
            state
        });
        let compact_picker = cx.new(|cx| {
            let mut state = DatePickerState::new(window, cx).display_style(DateDisplayStyle::Short);
            state.set_date(Some(date(2026, 10, 5)), window, cx);
            state
        });

        let subscriptions = vec![
            cx.subscribe_in(&range_picker, window, Self::on_range_picker_change),
            cx.subscribe_in(&long_picker, window, Self::on_picker_change),
            cx.subscribe_in(&compact_picker, window, Self::on_picker_change),
        ];

        Self {
            range_picker,
            long_picker,
            compact_picker,
            last_change: "Interact with a picker to inspect emitted change output.".into(),
            _subscriptions: subscriptions,
        }
    }

    fn picker_label(&self, picker: &Entity<DatePickerState>) -> &'static str {
        if picker == &self.long_picker {
            "Long"
        } else if picker == &self.compact_picker {
            "Compact"
        } else {
            "Unknown"
        }
    }

    fn on_picker_change(
        &mut self,
        picker: &Entity<DatePickerState>,
        event: &DatePickerEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let DatePickerEvent::Change(date) = event;
        self.last_change = format!(
            "{} picker changed to {}",
            self.picker_label(picker),
            describe_date(*date)
        )
        .into();
        cx.notify();
    }

    fn on_range_picker_change(
        &mut self,
        _: &Entity<DateRangePickerState>,
        event: &DateRangePickerEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let DateRangePickerEvent::Change(start_date, end_date) = event;
        self.last_change = format!(
            "Range picker changed to {}",
            describe_range(*start_date, *end_date)
        )
        .into();
        cx.notify();
    }
}

impl Render for DatePickerStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (range_start, range_end) = self.range_picker.read(cx).range();
        let range_value = describe_range(range_start, range_end);
        let long_value = describe_date(self.long_picker.read(cx).date());
        let compact_value = describe_date(self.compact_picker.read(cx).date());

        let form = v_form()
            .child(story_field(
                "Range select",
                "Starts empty, uses the active locale, and selects a start and end date across the default two-month calendar.",
                DateRangePicker::new(&self.range_picker)
                    .placeholder(localize(cx, &DatePickerComponentText::LaunchPlaceholder))
                    .cleanable(true),
            ))
            .child(story_field(
                "Long display",
                "Prefilled with DateDisplayStyle::Long so the input follows the active locale with a full date string.",
                DatePicker::new(&self.long_picker)
                    .cleanable(true)
                    .number_of_months(1),
            ))
            .child(story_field(
                "Compact appearance",
                "Uses a short format, a single calendar month, and the borderless input presentation.",
                DatePicker::new(&self.compact_picker)
                    .cleanable(true)
                    .number_of_months(1)
                    .appearance(false),
            ));

        story_panel(
            "Localized date selection",
            "This exercises the runtime wrapper around the calendar component: locale-aware display text, selection state, and emitted change events.",
            div().flex().flex_col().gap_4().child(form).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .text_sm()
                    .child(format!("Range value: {range_value}"))
                    .child(format!("Long value: {long_value}"))
                    .child(format!("Compact value: {compact_value}"))
                    .child(format!("Last change event: {}", self.last_change)),
            ),
        )
    }
}

fn describe_date(date: Option<JiffDate>) -> String {
    match date {
        Some(date) => date.to_string(),
        None => "None".to_string(),
    }
}

fn describe_range(start_date: Option<JiffDate>, end_date: Option<JiffDate>) -> String {
    format!(
        "{} - {}",
        describe_date(start_date),
        describe_date(end_date)
    )
}
