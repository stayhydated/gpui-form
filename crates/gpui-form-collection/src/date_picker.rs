use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui_kit::component::{
    calendar::Date,
    date_picker::{DatePickerEvent, DatePickerState},
};
use gpui_kit::{Context, Window};
use jiff::civil::Date as CivilDate;

fn date_picker_value_change(event: &DatePickerEvent) -> ValueChange<CivilDate> {
    match event {
        DatePickerEvent::Change(value) => match value.date() {
            Date::Single(Some(date)) => ValueChange::Set(date),
            Date::Single(None) => ValueChange::Clear,
            Date::Range(_, _) => ValueChange::Unchanged,
        },
    }
}

fn date_range_picker_value_change(event: &DatePickerEvent) -> ValueChange<(CivilDate, CivilDate)> {
    match event {
        DatePickerEvent::Change(value) => match value.date() {
            Date::Range(Some(start), Some(end)) => ValueChange::Set((start, end)),
            Date::Range(_, _) => ValueChange::Clear,
            Date::Single(_) => ValueChange::Unchanged,
        },
    }
}

component_shape! {
    /// Form component for a `gpui_kit::component::date_picker::DatePicker`.
    pub struct DatePicker {
        state = DatePickerState;
        component = gpui_kit::component::date_picker::DatePicker;
        value = jiff::civil::Date;
        field_suffix = "date_picker";
        value_binding;

        impl GpuiComponentValueBinding<CivilDate> for DatePicker {
            type Event = DatePickerEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&CivilDate>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                state.set_date(Date::Single(value.copied()), window, cx);
            }

            fn value_change(_state: &Self::State, event: &Self::Event) -> ValueChange<CivilDate> {
                date_picker_value_change(event)
            }
        }
    }
}

impl_form_component_shape!(DatePicker, gpui_form_runtime::shape::RequiredValueStorage);

component_shape! {
    /// Form component for a range-mode `gpui_kit::component::date_picker::DatePicker`.
    pub struct DateRangePicker {
        state = DatePickerState;
        new = DatePickerState::range;
        component = gpui_kit::component::date_picker::DatePicker;
        value = (jiff::civil::Date, jiff::civil::Date);
        field_suffix = "date_range_picker";
        value_binding;

        impl GpuiComponentValueBinding<(CivilDate, CivilDate)> for DateRangePicker {
            type Event = DatePickerEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&(CivilDate, CivilDate)>,
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
            ) -> ValueChange<(CivilDate, CivilDate)> {
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
    use gpui_kit::component::date_picker::DateTime;

    fn at(date: CivilDate) -> jiff::civil::DateTime {
        date.at(0, 0, 0, 0)
    }

    #[test]
    fn date_picker_events_preserve_single_and_range_modes() {
        let start = CivilDate::new(2026, 7, 11).unwrap();
        let end = CivilDate::new(2026, 7, 18).unwrap();

        assert_eq!(
            date_picker_value_change(&DatePickerEvent::Change(DateTime::Single(Some(at(start))))),
            ValueChange::Set(start)
        );
        assert_eq!(
            date_picker_value_change(&DatePickerEvent::Change(DateTime::Single(None))),
            ValueChange::Clear
        );
        assert_eq!(
            date_picker_value_change(&DatePickerEvent::Change(DateTime::Range(
                Some(at(start)),
                Some(at(end))
            ))),
            ValueChange::Unchanged
        );

        assert_eq!(
            date_range_picker_value_change(&DatePickerEvent::Change(DateTime::Range(
                Some(at(start)),
                Some(at(end)),
            ))),
            ValueChange::Set((start, end))
        );
        assert_eq!(
            date_range_picker_value_change(&DatePickerEvent::Change(DateTime::Range(
                Some(at(start)),
                None
            ))),
            ValueChange::Clear
        );
        assert_eq!(
            date_range_picker_value_change(&DatePickerEvent::Change(DateTime::Single(Some(at(
                start
            ))))),
            ValueChange::Unchanged
        );
    }
}
