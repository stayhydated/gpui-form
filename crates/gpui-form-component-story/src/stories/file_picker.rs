use std::path::{Path, PathBuf};

use es_fluent::FluentMessage;
use gpui::{
    App, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _, Render,
    SharedString, Styled as _, Subscription, Window, div,
};
use gpui_component::form::v_form;

use gpui_form_component::file_picker::{
    FilePicker, FilePickerEvent, FilePickerMode, FilePickerState,
};

use crate::i18n::FilePickerComponentText;

use super::common::{story_field, story_panel};

fn localize(cx: &impl std::borrow::Borrow<App>, message: &impl FluentMessage) -> String {
    crate::i18n::localize_message(cx, message)
}

#[derive(gpui_storybook::StoryControls)]
#[gpui_storybook::story]
pub struct FilePickerStory {
    file_picker: Entity<FilePickerState>,
    directory_picker: Entity<FilePickerState>,
    multiple_picker: Entity<FilePickerState>,
    last_event: SharedString,
    _subscriptions: Vec<Subscription>,
}

impl gpui_storybook::Story for FilePickerStory {
    fn title(_: &gpui::App) -> String {
        "File Picker".into()
    }

    fn new_view(window: &mut Window, cx: &mut App) -> Entity<Self> {
        cx.new(|cx| Self::new(window, cx))
    }
}

impl Focusable for FilePickerStory {
    fn focus_handle(&self, cx: &App) -> gpui::FocusHandle {
        self.file_picker.read(cx).focus_handle().clone()
    }
}

impl FilePickerStory {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let file_picker = cx.new(|cx| FilePickerState::new(window, cx));
        let directory_picker = cx.new(|cx| {
            let mut state = FilePickerState::new(window, cx);
            state.set_path(PathBuf::from("/tmp/gpui-form"), window, cx);
            state
        });
        let multiple_picker = cx.new(|cx| FilePickerState::new(window, cx));

        let subscriptions = vec![
            cx.subscribe_in(&file_picker, window, Self::on_picker_event),
            cx.subscribe_in(&directory_picker, window, Self::on_picker_event),
            cx.subscribe_in(&multiple_picker, window, Self::on_picker_event),
        ];

        Self {
            file_picker,
            directory_picker,
            multiple_picker,
            last_event: "Interact with a picker to inspect FilePickerEvent output.".into(),
            _subscriptions: subscriptions,
        }
    }

    fn picker_label(&self, picker: &Entity<FilePickerState>) -> &'static str {
        if picker == &self.file_picker {
            "File"
        } else if picker == &self.directory_picker {
            "Directory"
        } else if picker == &self.multiple_picker {
            "Multiple"
        } else {
            "Unknown"
        }
    }

    fn on_picker_event(
        &mut self,
        picker: &Entity<FilePickerState>,
        event: &FilePickerEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.last_event = match event {
            FilePickerEvent::Change(paths) => {
                format!(
                    "{} picker changed to {}",
                    self.picker_label(picker),
                    describe_paths(paths)
                )
            },
            FilePickerEvent::Cancel => {
                format!("{} picker was cancelled", self.picker_label(picker))
            },
            FilePickerEvent::Error(message) => {
                format!("{} picker error: {message}", self.picker_label(picker))
            },
        }
        .into();
        cx.notify();
    }
}

impl Render for FilePickerStory {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let file_value = describe_paths(self.file_picker.read(cx).paths());
        let directory_value = describe_paths(self.directory_picker.read(cx).paths());
        let multiple_value = describe_paths(self.multiple_picker.read(cx).paths());

        let form = v_form()
            .child(story_field(
                "File",
                "Selects one file using GPUI's native PathPromptOptions.",
                FilePicker::new(&self.file_picker)
                    .placeholder(localize(cx, &FilePickerComponentText::SourcePlaceholder))
                    .prompt(localize(cx, &FilePickerComponentText::SourcePlaceholder))
                    .cleanable(true),
            ))
            .child(story_field(
                "Directory",
                "Selects one directory and starts with a programmatic value.",
                FilePicker::new(&self.directory_picker)
                    .mode(FilePickerMode::Directory)
                    .placeholder(localize(cx, &FilePickerComponentText::OutputPlaceholder))
                    .prompt(localize(cx, &FilePickerComponentText::OutputPlaceholder))
                    .cleanable(true),
            ))
            .child(story_field(
                "Multiple",
                "Allows selecting multiple files; the state stores the full PathBuf list.",
                FilePicker::new(&self.multiple_picker)
                    .multiple(true)
                    .browse_label(localize(cx, &FilePickerComponentText::ChooseFiles))
                    .cleanable(true),
            ));

        story_panel(
            "Native path selection",
            "This exercises FilePickerState, native GPUI path prompts, clear handling, cancellation, and error events.",
            div().flex().flex_col().gap_4().child(form).child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .mt_2()
                    .text_sm()
                    .child(format!("File value: {file_value}"))
                    .child(format!("Directory value: {directory_value}"))
                    .child(format!("Multiple value: {multiple_value}"))
                    .child(format!("Last event: {}", self.last_event)),
            ),
        )
    }
}

fn describe_paths(paths: &[PathBuf]) -> String {
    match paths {
        [] => "None".to_string(),
        [path] => display_path(path),
        _ => paths
            .iter()
            .map(|path| display_path(path))
            .collect::<Vec<_>>()
            .join(", "),
    }
}

fn display_path(path: &Path) -> String {
    path.display().to_string()
}
