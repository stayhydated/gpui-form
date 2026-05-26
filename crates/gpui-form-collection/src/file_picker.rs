use std::path::PathBuf;

use gpui::{Context, Window};
use gpui_form_component::file_picker::{FilePickerEvent, FilePickerState};
use gpui_form_component::shape::{ComponentValueBinding, FormValueChange};

// Form component for a `gpui_form_component::file_picker::FilePicker`.
gpui_form_derive::component_shape! {
    pub struct FilePicker {
        type State = FilePickerState;
        new = FilePickerState::new;
        component = gpui_form_component::file_picker::FilePicker;
        value_binding;
        field_suffix = "file_picker";
    }
}

impl ComponentValueBinding<Vec<PathBuf>> for FilePicker {
    type Event = FilePickerEvent;

    fn seed_value_binding_state(
        state: &mut Self::State,
        value: Option<&Vec<PathBuf>>,
        window: &mut Window,
        cx: &mut Context<'_, Self::State>,
    ) {
        match value {
            Some(value) => state.set_paths(value.clone(), window, cx),
            None => state.clear_paths(window, cx),
        }
    }

    fn form_value_change(
        _state: &Self::State,
        event: &Self::Event,
    ) -> FormValueChange<Vec<PathBuf>> {
        match event {
            FilePickerEvent::Change(paths) => FormValueChange::Set(paths.clone()),
            _ => FormValueChange::Unchanged,
        }
    }
}
