use chrono::NaiveDate;
use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui::{Context, Window};
use gpui_component::{
    calendar::Date,
    date_picker::{DatePickerEvent, DatePickerState},
};

component_shape! {
    /// Form component for a `gpui_component::date_picker::DatePicker`.
    pub struct DatePicker {
        type State = DatePickerState;
        component = gpui_component::date_picker::DatePicker;
        value = NaiveDate;
        field_suffix = "date_picker";
        value_binding;

        impl GpuiComponentValueBinding<NaiveDate> for DatePicker {
            type Event = DatePickerEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&NaiveDate>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                state.set_date(Date::Single(value.copied()), window, cx);
            }

            fn value_change(_state: &Self::State, event: &Self::Event) -> ValueChange<NaiveDate> {
                match event {
                    DatePickerEvent::Change(Date::Single(Some(date))) => ValueChange::Set(*date),
                    DatePickerEvent::Change(Date::Single(None)) => ValueChange::Clear,
                    DatePickerEvent::Change(Date::Range(_, _)) => ValueChange::Unchanged,
                }
            }
        }
    }
}

impl_form_component_shape!(DatePicker, gpui_form_runtime::shape::RequiredValueStorage);

component_shape! {
    /// Form component for a range-mode `gpui_component::date_picker::DatePicker`.
    pub struct DateRangePicker {
        type State = DatePickerState;
        new = DatePickerState::range;
        component = gpui_component::date_picker::DatePicker;
        value = (NaiveDate, NaiveDate);
        field_suffix = "date_range_picker";
        value_binding;

        impl GpuiComponentValueBinding<(NaiveDate, NaiveDate)> for DateRangePicker {
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

            fn value_change(
                _state: &Self::State,
                event: &Self::Event,
            ) -> ValueChange<(NaiveDate, NaiveDate)> {
                match event {
                    DatePickerEvent::Change(Date::Range(Some(start), Some(end))) => {
                        ValueChange::Set((*start, *end))
                    },
                    DatePickerEvent::Change(Date::Range(_, _)) => ValueChange::Clear,
                    DatePickerEvent::Change(Date::Single(_)) => ValueChange::Unchanged,
                }
            }
        }
    }
}

impl_form_component_shape!(
    DateRangePicker,
    gpui_form_runtime::shape::RequiredValueStorage
);
