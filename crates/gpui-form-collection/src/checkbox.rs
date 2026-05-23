use gpui::{App, Context, Entity, EventEmitter, IntoElement, RenderOnce, Window};
use gpui_component::checkbox::Checkbox as GpuiCheckbox;
use gpui_form_component::custom::{CustomComponentValueAdapter, CustomComponentValueChange};

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
pub struct Checkbox {
    state: Entity<CheckboxState>,
}

impl Checkbox {
    pub fn new(state: &Entity<CheckboxState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for Checkbox {
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
    /// Shape for a value-bound `gpui_component::checkbox::Checkbox`.
    pub struct CheckboxShape {
        type State = CheckboxState;
        new = CheckboxState::new;
        component = gpui_form_collection::checkbox::Checkbox;
        value_binding;
        field_suffix = "checkbox";
    }
}

impl CustomComponentValueAdapter<bool> for CheckboxShape {
    type Event = CheckboxEvent;

    fn set_state_value(
        state: &mut Self::State,
        value: Option<&bool>,
        _window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        state.set_checked(value.copied().unwrap_or(false), cx);
    }

    fn value_change(_state: &Self::State, event: &Self::Event) -> CustomComponentValueChange<bool> {
        match event {
            CheckboxEvent::Change(checked) => CustomComponentValueChange::Set(*checked),
        }
    }
}
