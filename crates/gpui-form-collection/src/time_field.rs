use chrono::NaiveTime;
use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui_kit::component::time_field::{TimeFieldEvent, TimeFieldState};
use gpui_kit::{Context, Window};

fn time_field_value_change(event: &TimeFieldEvent) -> ValueChange<NaiveTime> {
    match event {
        TimeFieldEvent::Change(time) => ValueChange::Set(*time),
    }
}

component_shape! {
    /// Form component for a `gpui_kit::component::time_field::TimeField`.
    pub struct TimeField {
        state = TimeFieldState;
        component = gpui_kit::component::time_field::TimeField;
        value = NaiveTime;
        field_suffix = "time_field";
        value_binding;

        impl GpuiComponentValueBinding<NaiveTime> for TimeField {
            type Event = TimeFieldEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&NaiveTime>,
                window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                state.set_time(value.copied().unwrap_or(NaiveTime::MIN), window, cx);
            }

            fn value_change(_state: &Self::State, event: &Self::Event) -> ValueChange<NaiveTime> {
                time_field_value_change(event)
            }
        }
    }
}

impl_form_component_shape!(TimeField, gpui_form_runtime::shape::RequiredValueStorage);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_field_events_set_the_edited_time() {
        let time = NaiveTime::from_hms_opt(9, 30, 15).unwrap();

        assert_eq!(
            time_field_value_change(&TimeFieldEvent::Change(time)),
            ValueChange::Set(time)
        );
    }
}
