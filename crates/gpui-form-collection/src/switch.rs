use gpui::{App, Context, Entity, EventEmitter, IntoElement, RenderOnce, Window};
use gpui_component::switch::Switch as GpuiSwitch;
use gpui_form_component::custom::{
    CustomComponentValueBinding, FormValueChange, OwnedCustomComponentValueBinding,
};

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

gpui_form_derive::custom_component! {
    /// Form component for a value-bound `gpui_component::switch::Switch`.
    pub struct Switch {
        type State = SwitchState;
        new = SwitchState::new;
        component = gpui_form_collection::switch::SwitchField;
        value_binding;
        field_suffix = "switch";
    }
}

impl CustomComponentValueBinding<bool> for Switch {
    type Event = SwitchEvent;

    fn seed_value_binding_state(
        state: &mut Self::State,
        value: Option<&bool>,
        _window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        state.set_checked(value.copied().unwrap_or(false), cx);
    }

    fn form_value_change(_state: &Self::State, event: &Self::Event) -> FormValueChange<bool> {
        match event {
            SwitchEvent::Change(checked) => FormValueChange::Set(*checked),
        }
    }
}

impl OwnedCustomComponentValueBinding<bool> for Switch {}
