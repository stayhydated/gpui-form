use chrono::NaiveDate;
use gpui::{Context, Window};
use gpui_component::{
    calendar::Date,
    date_picker::{DatePickerEvent, DatePickerState},
};
use gpui_form_runtime::shape::{ComponentValueBinding, FormValueChange};

gpui_form_derive::component_shape! {
    /// Form component for a `gpui_component::date_picker::DatePicker`.
    pub struct DatePicker {
        type State = DatePickerState;
        component = gpui_component::date_picker::DatePicker;
        value = NaiveDate;
        field_suffix = "date_picker";
        value_binding;

        impl ComponentValueBinding<NaiveDate> for DatePicker {
            type Event = DatePickerEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&NaiveDate>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                state.set_date(Date::Single(value.copied()), window, cx);
            }

            fn form_value_change(_state: &Self::State, event: &Self::Event) -> FormValueChange<NaiveDate> {
                match event {
                    DatePickerEvent::Change(Date::Single(Some(date))) => FormValueChange::Set(*date),
                    DatePickerEvent::Change(Date::Single(None)) => FormValueChange::Clear,
                    DatePickerEvent::Change(Date::Range(_, _)) => FormValueChange::Unchanged,
                }
            }
        }
    }
}

gpui_form_derive::component_shape! {
    /// Form component for a range-mode `gpui_component::date_picker::DatePicker`.
    pub struct DateRangePicker {
        type State = DatePickerState;
        new = DatePickerState::range;
        component = gpui_component::date_picker::DatePicker;
        value = (NaiveDate, NaiveDate);
        field_suffix = "date_range_picker";
        value_binding;

        impl ComponentValueBinding<(NaiveDate, NaiveDate)> for DateRangePicker {
            type Event = DatePickerEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&(NaiveDate, NaiveDate)>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                let date = match value {
                    Some((start, end)) => Date::Range(Some(*start), Some(*end)),
                    None => Date::Range(None, None),
                };
                state.set_date(date, window, cx);
            }

            fn form_value_change(
                _state: &Self::State,
                event: &Self::Event,
            ) -> FormValueChange<(NaiveDate, NaiveDate)> {
                match event {
                    DatePickerEvent::Change(Date::Range(Some(start), Some(end))) => {
                        FormValueChange::Set((*start, *end))
                    },
                    DatePickerEvent::Change(Date::Range(_, _)) => FormValueChange::Clear,
                    DatePickerEvent::Change(Date::Single(_)) => FormValueChange::Unchanged,
                }
            }
        }
    }
}
