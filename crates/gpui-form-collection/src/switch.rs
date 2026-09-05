use component_shape::ValueChange;
use component_shape_gpui::{GpuiComponentValueBinding, component_shape};
use gpui_kit as gpui;
use gpui_kit::component::switch::Switch as GpuiSwitch;
use gpui_kit::{App, Context, Entity, EventEmitter, IntoElement, RenderOnce, Window};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwitchEvent {
    Change(bool),
}

#[derive(Debug, Default)]
pub struct SwitchState {
    checked: bool,
}

impl SwitchState {
    pub fn new(_window: &mut Window, _cx: &mut Context<'_, Self>) -> Self {
        Self::default()
    }

    pub const fn checked(&self) -> bool {
        self.checked
    }

    pub fn set_checked(&mut self, checked: bool, cx: &mut Context<'_, Self>) {
        if self.checked == checked {
            return;
        }

        self.checked = checked;
        cx.emit(SwitchEvent::Change(checked));
        cx.notify();
    }
}

impl EventEmitter<SwitchEvent> for SwitchState {}

#[derive(IntoElement)]
pub struct SwitchField {
    state: Entity<SwitchState>,
}

impl SwitchField {
    pub fn new(state: &Entity<SwitchState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for SwitchField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.state.read(cx).checked();
        let state = self.state.clone();

        GpuiSwitch::new(("switch", self.state.entity_id()))
            .checked(checked)
            .on_click(move |checked, _window, cx| {
                state.update(cx, |state, cx| {
                    state.set_checked(*checked, cx);
                });
            })
    }
}

component_shape! {
    /// Form component for a value-bound `gpui_kit::component::switch::Switch`.
    pub struct Switch {
        state = SwitchState;
        component = gpui_form_collection::switch::SwitchField;
        value = bool;
        field_suffix = "switch";
        value_binding;

        impl GpuiComponentValueBinding<bool> for Switch {
            type Event = SwitchEvent;

            fn seed_value_binding_state(
                state: &mut Self::State,
                value: Option<&bool>,
                _window: &mut Window,
                cx: &mut Context<'_, Self::State>,
            ) {
                state.set_checked(value.copied().unwrap_or(false), cx);
            }

            fn value_change(_state: &Self::State, event: &Self::Event) -> ValueChange<bool> {
                match event {
                    SwitchEvent::Change(checked) => ValueChange::Set(*checked),
                }
            }
        }
    }
}

impl_form_component_shape!(Switch, gpui_form_runtime::shape::DirectValueStorage);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_events_map_to_form_values() {
        let state = SwitchState::default();
        assert!(!state.checked());
        assert_eq!(
            <Switch as GpuiComponentValueBinding<bool>>::value_change(
                &state,
                &SwitchEvent::Change(true),
            ),
            ValueChange::Set(true)
        );
    }
}
