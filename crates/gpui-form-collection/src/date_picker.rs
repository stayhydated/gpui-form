use chrono::NaiveDate;
use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui::{Context, Window};
use gpui_component::{
    calendar::Date,
    date_picker::{DatePickerEvent, DatePickerState},
};

fn date_picker_value_change(event: &DatePickerEvent) -> ValueChange<NaiveDate> {
    match event {
        DatePickerEvent::Change(Date::Single(Some(date))) => ValueChange::Set(*date),
        DatePickerEvent::Change(Date::Single(None)) => ValueChange::Clear,
        DatePickerEvent::Change(Date::Range(_, _)) => ValueChange::Unchanged,
    }
}

fn date_range_picker_value_change(event: &DatePickerEvent) -> ValueChange<(NaiveDate, NaiveDate)> {
    match event {
        DatePickerEvent::Change(Date::Range(Some(start), Some(end))) => {
            ValueChange::Set((*start, *end))
        },
        DatePickerEvent::Change(Date::Range(_, _)) => ValueChange::Clear,
        DatePickerEvent::Change(Date::Single(_)) => ValueChange::Unchanged,
    }
}

component_shape! {
    /// Form component for a `gpui_component::date_picker::DatePicker`.
    pub struct DatePicker {
        state = DatePickerState;
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
                date_picker_value_change(event)
            }
        }
    }
}

impl_form_component_shape!(DatePicker, gpui_form_runtime::shape::RequiredValueStorage);

component_shape! {
    /// Form component for a range-mode `gpui_component::date_picker::DatePicker`.
    pub struct DateRangePicker {
        state = DatePickerState;
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
                date_range_picker_value_change(event)
            }
        }
    }
}

impl_form_component_shape!(
    DateRangePicker,
    gpui_form_runtime::shape::RequiredValueStorage
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn date_picker_events_preserve_single_and_range_modes() {
        let start = NaiveDate::from_ymd_opt(2026, 7, 11).unwrap();
        let end = NaiveDate::from_ymd_opt(2026, 7, 18).unwrap();

        assert_eq!(
            date_picker_value_change(&DatePickerEvent::Change(Date::Single(Some(start)))),
            ValueChange::Set(start)
        );
        assert_eq!(
            date_picker_value_change(&DatePickerEvent::Change(Date::Single(None))),
            ValueChange::Clear
        );
        assert_eq!(
            date_picker_value_change(&DatePickerEvent::Change(Date::Range(
                Some(start),
                Some(end)
            ))),
            ValueChange::Unchanged
        );

        assert_eq!(
            date_range_picker_value_change(&DatePickerEvent::Change(Date::Range(
                Some(start),
                Some(end),
            ))),
            ValueChange::Set((start, end))
        );
        assert_eq!(
            date_range_picker_value_change(&DatePickerEvent::Change(Date::Range(
                Some(start),
                None
            ))),
            ValueChange::Clear
        );
        assert_eq!(
            date_range_picker_value_change(&DatePickerEvent::Change(Date::Single(Some(start)))),
            ValueChange::Unchanged
        );
    }
}
