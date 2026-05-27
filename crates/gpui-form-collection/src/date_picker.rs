use chrono::NaiveDate;
use chrono::Datelike;
use gpui::{Context, Window};
use gpui_form_component::date_picker::{
    parse_form_date, DatePickerEvent, DatePickerState, DateRangePickerEvent, DateRangePickerState,
};
use gpui_form_component::shape::{ComponentValueBinding, FormValueChange};

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_form_component::date_picker::DatePicker`.
    pub struct DatePicker {
        type State = DatePickerState;
        component = gpui_form_component::date_picker::DatePicker;
        value_binding;
        field_suffix = "date_picker";
    }
}

impl ComponentValueBinding<NaiveDate> for DatePicker {
    type Event = DatePickerEvent;

    fn seed_value_binding_state(
        state: &mut Self::State,
        value: Option<&NaiveDate>,
        window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        state.set_date(value.and_then(|value| jiff_date_from_chrono(*value)), window, cx);
    }

    fn form_value_change(_state: &Self::State, event: &Self::Event) -> FormValueChange<NaiveDate> {
        match event {
            DatePickerEvent::Change(Some(date)) => {
                date.to_string().parse::<NaiveDate>()
                    .map_or(FormValueChange::Unchanged, FormValueChange::Set)
            }
            DatePickerEvent::Change(None) => FormValueChange::Clear,
        }
    }
}

fn jiff_date_from_chrono(date: NaiveDate) -> Option<jiff::civil::Date> {
    let year = i16::try_from(date.year()).ok()?;
    let month = i8::try_from(date.month()).ok()?;
    let day = i8::try_from(date.day()).ok()?;
    jiff::civil::Date::new(year, month, day).ok()
}

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_form_component::date_picker::DateRangePicker`.
    pub struct DateRangePicker {
        type State = DateRangePickerState;
        component = gpui_form_component::date_picker::DateRangePicker;
        value_binding;
        field_suffix = "date_range_picker";
    }
}

impl ComponentValueBinding<(NaiveDate, NaiveDate)> for DateRangePicker {
    type Event = DateRangePickerEvent;

    fn seed_value_binding_state(
        state: &mut Self::State,
        value: Option<&(NaiveDate, NaiveDate)>,
        window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        state.set_range(
            value.and_then(|(start, _)| jiff_date_from_chrono(*start)),
            value.and_then(|(_, end)| jiff_date_from_chrono(*end)),
            window,
            cx,
        );
    }

    fn form_value_change(
        _state: &Self::State,
        event: &Self::Event,
    ) -> FormValueChange<(NaiveDate, NaiveDate)> {
        match event {
            DateRangePickerEvent::Change(Some(start), Some(end)) => {
                jiff_date_range_from_chrono(start, end).map_or(FormValueChange::Unchanged, |value| {
                    FormValueChange::Set(value)
                })
            }
            DateRangePickerEvent::Change(_, _) => FormValueChange::Clear,
        }
    }
}

fn jiff_date_range_from_chrono(
    start: &jiff::civil::Date,
    end: &jiff::civil::Date,
) -> Option<(NaiveDate, NaiveDate)> {
    let start = parse_form_date(start.clone())?;
    let end = parse_form_date(end.clone())?;
    Some((start, end))
}
