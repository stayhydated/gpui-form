use gpui::{App, Context, Entity, EventEmitter, IntoElement, RenderOnce, Window};
use gpui_component::checkbox::Checkbox as GpuiCheckbox;
use gpui_form_component::custom::{
    CustomComponentValueBinding, FormValueChange, OwnedCustomComponentValueBinding,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CheckboxEvent {
    Change(bool),
}

#[derive(Debug, Default)]
pub struct CheckboxState {
    checked: bool,
}

impl CheckboxState {
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
        cx.emit(CheckboxEvent::Change(checked));
        cx.notify();
    }
}

impl EventEmitter<CheckboxEvent> for CheckboxState {}

#[derive(IntoElement)]
pub struct CheckboxField {
    state: Entity<CheckboxState>,
}

impl CheckboxField {
    pub fn new(state: &Entity<CheckboxState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for CheckboxField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let checked = self.state.read(cx).checked();
        let state = self.state.clone();

        GpuiCheckbox::new(("checkbox", self.state.entity_id()))
            .checked(checked)
            .on_click(move |checked, _window, cx| {
                state.update(cx, |state, cx| {
                    state.set_checked(*checked, cx);
                });
            })
    }
}

gpui_form_derive::custom_component! {
    /// Form component for a value-bound `gpui_component::checkbox::Checkbox`.
    pub struct Checkbox {
        type State = CheckboxState;
        new = CheckboxState::new;
        component = gpui_form_collection::checkbox::CheckboxField;
        value_binding;
        field_suffix = "checkbox";
    }
}

impl CustomComponentValueBinding<bool> for Checkbox {
    type Event = CheckboxEvent;

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
            CheckboxEvent::Change(checked) => FormValueChange::Set(*checked),
        }
    }
}

impl OwnedCustomComponentValueBinding<bool> for Checkbox {}
