use gpui_form_derive::component_value_binding;

struct State;

impl<Event: 'static> gpui::EventEmitter<Event> for State {}

#[component_value_binding(option)]
impl gpui_form_runtime::shape::ComponentValueBinding<String> for State {
    type Event = ();

    fn form_value_change(
        _state: &Self::State,
        _event: &Self::Event,
    ) -> gpui_form_runtime::shape::FormValueChange<String> {
        gpui_form_runtime::shape::FormValueChange::Unchanged
    }
}

#[component_value_binding]
impl State {}

#[component_value_binding]
impl std::fmt::Debug for State {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(())
    }
}

#[component_value_binding]
impl gpui_form_runtime::shape::ComponentValueBinding for State {
    type Event = ();

    fn form_value_change(
        _state: &Self::State,
        _event: &Self::Event,
    ) -> gpui_form_runtime::shape::FormValueChange<String> {
        gpui_form_runtime::shape::FormValueChange::Unchanged
    }
}

fn main() {}
