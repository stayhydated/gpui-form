use gpui::{App, Context, Entity, EventEmitter, IntoElement, RenderOnce, Window};
use gpui_component::switch::Switch as GpuiSwitch;
use gpui_form_component::custom::{CustomComponentValueAdapter, CustomComponentValueChange};

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
pub struct Switch {
    state: Entity<SwitchState>,
}

impl Switch {
    pub fn new(state: &Entity<SwitchState>) -> Self {
        Self {
            state: state.clone(),
        }
    }
}

impl RenderOnce for Switch {
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
    /// Shape for a value-bound `gpui_component::switch::Switch`.
    pub struct SwitchShape {
        type State = SwitchState;
        new = SwitchState::new;
        component = gpui_form_collection::switch::Switch;
        value_binding;
        field_suffix = "switch";
    }
}

impl CustomComponentValueAdapter<bool> for SwitchShape {
    type Event = SwitchEvent;

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
            SwitchEvent::Change(checked) => CustomComponentValueChange::Set(*checked),
        }
    }
}
