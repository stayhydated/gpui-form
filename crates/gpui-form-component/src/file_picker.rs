//! Runtime file picker support backed by GPUI's native path prompt.
//!
//! This module intentionally uses `gpui_kit::App::prompt_for_paths` from the
//! GPUI Kit facade instead of adding a second native-dialog dependency. The
//! rendered control follows GPUI Kit styling and emits
//! form-friendly change events.

use std::path::PathBuf;

use gpui_es_fluent::localize_message;
use gpui_kit as gpui;
use gpui_kit::component::{
    ActiveTheme as _, Disableable, Icon, IconName, Sizable, Size, StyleSized as _, StyledExt as _,
    ThemeStyled as _,
    button::{Button, ButtonVariants as _},
    h_flex,
};
use gpui_kit::{
    App, ClickEvent, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, PathPromptOptions, Render,
    RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _,
};

use crate::i18n::FilePickerText;
#[cfg(feature = "component-shape")]
use gpui_form_runtime::shape::ValueChange;

#[cfg(feature = "component-shape")]
fn file_picker_form_value_change(event: &FilePickerEvent) -> ValueChange<Vec<PathBuf>> {
    match event {
        FilePickerEvent::Change(paths) if paths.is_empty() => ValueChange::Clear,
        FilePickerEvent::Change(paths) => ValueChange::Set(paths.clone()),
        _ => ValueChange::Unchanged,
    }
}

/// Which path kinds a [`FilePicker`] should ask GPUI to select.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum FilePickerMode {
    /// Select files only.
    #[default]
    File,
    /// Select directories only.
    Directory,
    /// Select either files or directories when the platform supports it.
    FileOrDirectory,
}

impl FilePickerMode {
    /// Returns whether this mode allows files.
    pub fn allows_files(self) -> bool {
        matches!(self, Self::File | Self::FileOrDirectory)
    }

    /// Returns whether this mode allows directories.
    pub fn allows_directories(self) -> bool {
        matches!(self, Self::Directory | Self::FileOrDirectory)
    }

    fn icon_name(self) -> IconName {
        match self {
            Self::File => IconName::File,
            Self::Directory => IconName::Folder,
            Self::FileOrDirectory => IconName::FolderOpen,
        }
    }

    fn default_placeholder(self, cx: &impl std::borrow::Borrow<App>) -> SharedString {
        let message = match self {
            Self::File => FilePickerText::SelectAFile,
            Self::Directory => FilePickerText::SelectADirectory,
            Self::FileOrDirectory => FilePickerText::SelectAFileOrDirectory,
        };
        localize_message(cx, &message).into()
    }

    fn default_prompt(self, cx: &impl std::borrow::Borrow<App>) -> SharedString {
        let message = match self {
            Self::File => FilePickerText::SelectFile,
            Self::Directory => FilePickerText::SelectDirectory,
            Self::FileOrDirectory => FilePickerText::SelectFileOrDirectory,
        };
        localize_message(cx, &message).into()
    }
}

/// Events emitted by [`FilePickerState`].
#[derive(Clone, Debug)]
pub enum FilePickerEvent {
    /// The selected paths changed. An empty list means the picker was cleared.
    Change(Vec<PathBuf>),
    /// The platform dialog was cancelled without changing the current value.
    Cancel,
    /// The platform dialog failed to open or return a result.
    Error(SharedString),
}

/// Render and native-dialog options for [`FilePicker`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "component-shape", derive(bon::Builder))]
pub struct FilePickerOptions {
    #[cfg_attr(feature = "component-shape", builder(default))]
    mode: FilePickerMode,
    #[cfg_attr(feature = "component-shape", builder(default))]
    multiple: bool,
    prompt: Option<SharedString>,
    placeholder: Option<SharedString>,
    browse_label: Option<SharedString>,
    #[cfg_attr(feature = "component-shape", builder(default))]
    cleanable: bool,
    #[cfg_attr(feature = "component-shape", builder(default = true))]
    appearance: bool,
    #[cfg_attr(feature = "component-shape", builder(default))]
    disabled: bool,
    #[cfg_attr(feature = "component-shape", builder(default))]
    size: Size,
}

impl Default for FilePickerOptions {
    fn default() -> Self {
        Self {
            mode: FilePickerMode::default(),
            multiple: false,
            prompt: None,
            placeholder: None,
            browse_label: None,
            cleanable: false,
            appearance: true,
            disabled: false,
            size: Size::default(),
        }
    }
}

impl FilePickerOptions {
    /// Create file-picker options with default file-only selection.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the selected path mode.
    pub fn mode(&self) -> FilePickerMode {
        self.mode
    }

    /// Returns whether the native prompt allows multiple selected paths.
    pub fn multiple(&self) -> bool {
        self.multiple
    }
}

/// State for a native file picker control.
pub struct FilePickerState {
    focus_handle: FocusHandle,
    paths: Vec<PathBuf>,
    last_error: Option<SharedString>,
    options: FilePickerOptions,
}

#[cfg(feature = "component-shape")]
impl gpui_form_runtime::shape::GpuiComponentStateValueBinding<Vec<PathBuf>> for FilePickerState {
    type Event = FilePickerEvent;

    fn seed_value_binding_state(
        state: &mut Self,
        value: Option<&Vec<PathBuf>>,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        match value {
            Some(value) => state.set_paths(value.clone(), window, cx),
            None => state.clear_paths(window, cx),
        }
    }

    fn value_change(_state: &Self, event: &Self::Event) -> ValueChange<Vec<PathBuf>> {
        file_picker_form_value_change(event)
    }
}

impl Focusable for FilePickerState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<FilePickerEvent> for FilePickerState {}

impl FilePickerState {
    /// Create an empty file-picker state.
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_options(FilePickerOptions::default(), cx)
    }

    /// Create an empty file-picker state with configured render/dialog options.
    pub fn new_with_options(options: FilePickerOptions, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            paths: Vec::new(),
            last_error: None,
            options,
        }
    }

    /// Returns the currently selected paths.
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }

    /// Returns the first selected path, when present.
    pub fn path(&self) -> Option<&PathBuf> {
        self.paths.first()
    }

    /// Returns true when no path is selected.
    pub fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Returns the latest platform-dialog error, if one occurred.
    pub fn last_error(&self) -> Option<&SharedString> {
        self.last_error.as_ref()
    }

    /// Returns the configured render/dialog options for this picker.
    pub fn options(&self) -> &FilePickerOptions {
        &self.options
    }

    /// Programmatically replace the current selection with one path.
    pub fn set_path(
        &mut self,
        path: impl Into<PathBuf>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.replace_paths(vec![path.into()], false, window, cx);
    }

    /// Programmatically replace the current selection with zero or more paths.
    pub fn set_paths<I, P>(&mut self, paths: I, window: &mut Window, cx: &mut Context<Self>)
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        self.replace_paths(
            paths.into_iter().map(Into::into).collect(),
            false,
            window,
            cx,
        );
    }

    /// Programmatically clear the current selection.
    pub fn clear_paths(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.replace_paths(Vec::new(), false, window, cx);
    }

    /// Get the focus handle owned by this state.
    pub fn focus_handle(&self) -> &FocusHandle {
        &self.focus_handle
    }

    /// Focus the picker.
    pub fn focus(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
    }

    fn replace_paths(
        &mut self,
        paths: Vec<PathBuf>,
        emit: bool,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.paths = paths;
        self.last_error = None;
        if emit {
            cx.emit(FilePickerEvent::Change(self.paths.clone()));
        }
        cx.notify();
    }

    fn emit_cancel(&mut self, cx: &mut Context<Self>) {
        self.last_error = None;
        cx.emit(FilePickerEvent::Cancel);
        cx.notify();
    }

    fn emit_error(&mut self, message: impl Into<SharedString>, cx: &mut Context<Self>) {
        let message = message.into();
        self.last_error = Some(message.clone());
        cx.emit(FilePickerEvent::Error(message));
        cx.notify();
    }

    fn clear_from_click(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.replace_paths(Vec::new(), true, window, cx);
    }
}

impl Render for FilePickerState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui_kit::Empty
    }
}

/// A native file picker element using GPUI Kit visual primitives.
#[cfg_attr(
    feature = "component-shape",
    derive(component_shape_gpui::GpuiComponentShape)
)]
#[cfg_attr(
    feature = "component-shape",
    gpui_component_shape(
        state = FilePickerState,
        value = Vec<std::path::PathBuf>,
        field_suffix = "file_picker",
        value_binding
    )
)]
#[derive(IntoElement)]
pub struct FilePicker {
    id: ElementId,
    style: StyleRefinement,
    state: Entity<FilePickerState>,
    mode: Option<FilePickerMode>,
    multiple: Option<bool>,
    prompt: Option<SharedString>,
    placeholder: Option<SharedString>,
    browse_label: Option<SharedString>,
    cleanable: Option<bool>,
    appearance: Option<bool>,
    disabled: Option<bool>,
    size: Option<Size>,
}

#[cfg(feature = "component-shape")]
impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for FilePicker {
    type ValueStoragePolicy = gpui_form_runtime::shape::DirectValueStorage;
}

impl Sizable for FilePicker {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = Some(size.into());
        self
    }
}

impl Focusable for FilePicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}

impl Styled for FilePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Disableable for FilePicker {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = Some(disabled);
        self
    }
}

impl FilePicker {
    /// Use configured file-picker options as a component-shape builder.
    #[cfg(feature = "component-shape")]
    pub fn from(options: FilePickerOptions) -> FilePickerOptions {
        options
    }

    /// Create a file-only picker with the given state.
    pub fn new(state: &Entity<FilePickerState>) -> Self {
        Self {
            id: ("file-picker", state.entity_id()).into(),
            state: state.clone(),
            mode: None,
            multiple: None,
            prompt: None,
            placeholder: None,
            browse_label: None,
            cleanable: None,
            appearance: None,
            disabled: None,
            size: None,
            style: StyleRefinement::default(),
        }
    }

    /// Select files only.
    pub fn files(mut self) -> Self {
        self.mode = Some(FilePickerMode::File);
        self
    }

    /// Select directories only.
    pub fn directories(mut self) -> Self {
        self.mode = Some(FilePickerMode::Directory);
        self
    }

    /// Select either files or directories when the platform supports it.
    pub fn files_or_directories(mut self) -> Self {
        self.mode = Some(FilePickerMode::FileOrDirectory);
        self
    }

    /// Set the picker mode.
    pub fn mode(mut self, mode: FilePickerMode) -> Self {
        self.mode = Some(mode);
        self
    }

    /// Allow multiple selected paths.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = Some(multiple);
        self
    }

    /// Set the native dialog prompt.
    pub fn prompt(mut self, prompt: impl Into<SharedString>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Set the empty display placeholder.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set the browse button label.
    pub fn browse_label(mut self, label: impl Into<SharedString>) -> Self {
        self.browse_label = Some(label.into());
        self
    }

    /// Show a clear button when paths are selected.
    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = Some(cleanable);
        self
    }

    /// Set whether to render the picker with the default bordered input style.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = Some(appearance);
        self
    }
}

#[cfg(feature = "component-shape")]
impl component_shape_gpui::GpuiComponentShapeBuilder<FilePicker> for FilePickerOptions {
    fn build(self, _window: &mut Window, cx: &mut Context<'_, FilePickerState>) -> FilePickerState {
        FilePickerState::new_with_options(self, cx)
    }
}

impl RenderOnce for FilePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_focused = self.focus_handle(cx).contains_focused(window, cx);
        let entity_id = self.state.entity_id();
        let state = self.state.read(cx);
        let paths = state.paths.clone();
        let last_error = state.last_error.clone();
        let options = state.options.clone();
        let mode = self.mode.unwrap_or(options.mode);
        let multiple = self.multiple.unwrap_or(options.multiple);
        let prompt = self.prompt.clone().or(options.prompt.clone());
        let placeholder = self.placeholder.clone().or(options.placeholder.clone());
        let browse_label = self.browse_label.clone().or(options.browse_label.clone());
        let cleanable = self.cleanable.unwrap_or(options.cleanable);
        let appearance = self.appearance.unwrap_or(options.appearance);
        let disabled = self.disabled.unwrap_or(options.disabled);
        let size = self.size.unwrap_or(options.size);
        let has_paths = !paths.is_empty();
        let show_clean = cleanable && has_paths;
        let placeholder = placeholder.unwrap_or_else(|| mode.default_placeholder(cx));
        let display_title = display_paths(&paths, placeholder, |count| {
            localize_message(cx, &FilePickerText::PathsSelected { count }).into()
        });
        let prompt = prompt.or_else(|| Some(mode.default_prompt(cx)));
        let browse_label =
            browse_label.unwrap_or_else(|| localize_message(cx, &FilePickerText::Browse).into());
        let text_state = self.state.clone();
        let text_prompt = prompt.clone();
        let browse_state = self.state.clone();
        let browse_prompt = prompt;

        div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle(cx).tab_stop(true))
            .flex_none()
            .w_full()
            .input_text_size(size)
            .refine_style(&self.style)
            .when(disabled, |this| this.opacity(0.5))
            .child(
                div()
                    .id("file-picker-input")
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .when(appearance, |this| {
                        this.bg(cx.theme().background)
                            .text_color(cx.theme().foreground)
                            .border_1()
                            .border_color(cx.theme().input)
                            .rounded(cx.theme().radius)
                            .when(cx.theme().shadow, |this| this.shadow_xs())
                            .when(is_focused, |this| this.border_color(cx.theme().ring))
                    })
                    .when(is_focused && appearance && !disabled, |this| {
                        this.focus_ring_style(window, cx)
                    })
                    .input_text_size(size)
                    .input_size(size)
                    .overflow_hidden()
                    .child(
                        h_flex()
                            .id(("file-picker-display", entity_id))
                            .w_full()
                            .items_center()
                            .gap_2()
                            .overflow_hidden()
                            .when(!disabled, |this| this.cursor_pointer())
                            .when(!disabled, |this| {
                                this.on_click(move |_, window, cx| {
                                    prompt_for_selection(
                                        text_state.clone(),
                                        mode,
                                        multiple,
                                        text_prompt.clone(),
                                        window,
                                        cx,
                                    );
                                })
                            })
                            .child(
                                Icon::new(mode.icon_name())
                                    .xsmall()
                                    .text_color(cx.theme().muted_foreground),
                            )
                            .child(
                                div()
                                    .w_full()
                                    .overflow_hidden()
                                    .when(!has_paths, |this| {
                                        this.text_color(cx.theme().muted_foreground)
                                    })
                                    .child(display_title),
                            ),
                    )
                    .when(!disabled && show_clean, |this| {
                        this.child(
                            Button::new(("clear-file-picker", entity_id))
                                .small()
                                .ghost()
                                .icon(IconName::Close)
                                .on_click(
                                    window.listener_for(
                                        &self.state,
                                        FilePickerState::clear_from_click,
                                    ),
                                ),
                        )
                    })
                    .child(
                        Button::new(("browse-file-picker", entity_id))
                            .with_size(size)
                            .secondary()
                            .icon(mode.icon_name())
                            .label(browse_label)
                            .disabled(disabled)
                            .on_click(move |_, window, cx| {
                                prompt_for_selection(
                                    browse_state.clone(),
                                    mode,
                                    multiple,
                                    browse_prompt.clone(),
                                    window,
                                    cx,
                                );
                            }),
                    ),
            )
            .when_some(last_error, |this, message| {
                this.child(
                    div()
                        .mt_1()
                        .text_xs()
                        .text_color(cx.theme().danger)
                        .child(message),
                )
            })
    }
}

fn prompt_for_selection(
    state: Entity<FilePickerState>,
    mode: FilePickerMode,
    multiple: bool,
    prompt: Option<SharedString>,
    window: &mut Window,
    cx: &mut App,
) {
    let options = path_prompt_options(mode, multiple, prompt, cx.can_select_mixed_files_and_dirs());
    let paths = cx.prompt_for_paths(options);

    window
        .spawn(cx, async move |cx| {
            let outcome = classify_prompt_result(paths.await);

            _ = state.update_in(cx, |this, window, cx| match outcome {
                FilePickerPromptOutcome::Selected(paths) => {
                    this.replace_paths(paths, true, window, cx);
                },
                FilePickerPromptOutcome::Cancelled => this.emit_cancel(cx),
                FilePickerPromptOutcome::Failed(message) => this.emit_error(message, cx),
                FilePickerPromptOutcome::Dropped => {
                    this.emit_error(localize_message(cx, &FilePickerText::DialogDropped), cx);
                },
            });
        })
        .detach();
}

fn path_prompt_options(
    mode: FilePickerMode,
    multiple: bool,
    prompt: Option<SharedString>,
    mixed_selection_supported: bool,
) -> PathPromptOptions {
    PathPromptOptions {
        files: mode.allows_files(),
        directories: mode.allows_directories()
            && (!mode.allows_files() || mixed_selection_supported),
        multiple,
        prompt,
    }
}

#[derive(Debug, Eq, PartialEq)]
enum FilePickerPromptOutcome {
    Selected(Vec<PathBuf>),
    Cancelled,
    Failed(String),
    Dropped,
}

fn classify_prompt_result<E, D>(
    result: Result<Result<Option<Vec<PathBuf>>, E>, D>,
) -> FilePickerPromptOutcome
where
    E: std::fmt::Display,
{
    match result {
        Ok(Ok(Some(paths))) => FilePickerPromptOutcome::Selected(paths),
        Ok(Ok(None)) => FilePickerPromptOutcome::Cancelled,
        Ok(Err(error)) => FilePickerPromptOutcome::Failed(error.to_string()),
        Err(_) => FilePickerPromptOutcome::Dropped,
    }
}

fn display_paths(
    paths: &[PathBuf],
    placeholder: SharedString,
    multiple_label: impl FnOnce(usize) -> SharedString,
) -> SharedString {
    match paths {
        [] => placeholder,
        [path] => path.display().to_string().into(),
        _ => multiple_label(paths.len()),
    }
}

#[cfg(test)]
mod prompt_tests {
    use std::{io, path::PathBuf};

    use super::{
        FilePickerMode, FilePickerPromptOutcome, classify_prompt_result, path_prompt_options,
    };

    #[test]
    fn classifies_every_native_dialog_completion() {
        let paths = vec![PathBuf::from("one.txt"), PathBuf::from("two.txt")];
        assert_eq!(
            classify_prompt_result::<io::Error, ()>(Ok(Ok(Some(paths.clone())))),
            FilePickerPromptOutcome::Selected(paths)
        );
        assert_eq!(
            classify_prompt_result::<io::Error, ()>(Ok(Ok(None))),
            FilePickerPromptOutcome::Cancelled
        );
        assert_eq!(
            classify_prompt_result::<io::Error, ()>(Ok(Err(io::Error::other("denied")))),
            FilePickerPromptOutcome::Failed("denied".to_string())
        );
        assert_eq!(
            classify_prompt_result::<io::Error, ()>(Err(())),
            FilePickerPromptOutcome::Dropped
        );
    }

    #[test]
    fn prompt_options_respect_mode_and_platform_mixed_selection_support() {
        let files = path_prompt_options(FilePickerMode::File, true, Some("Pick".into()), false);
        assert!(files.files);
        assert!(!files.directories);
        assert!(files.multiple);
        assert_eq!(files.prompt.as_deref(), Some("Pick"));

        let directories = path_prompt_options(FilePickerMode::Directory, false, None, false);
        assert!(!directories.files);
        assert!(directories.directories);
        assert!(!directories.multiple);

        let unsupported_mixed =
            path_prompt_options(FilePickerMode::FileOrDirectory, false, None, false);
        assert!(unsupported_mixed.files);
        assert!(!unsupported_mixed.directories);

        let supported_mixed =
            path_prompt_options(FilePickerMode::FileOrDirectory, false, None, true);
        assert!(supported_mixed.files);
        assert!(supported_mixed.directories);
    }
}

#[cfg(all(test, feature = "component-shape"))]
mod tests {
    use super::{
        FilePicker, FilePickerEvent, FilePickerMode, FilePickerOptions, display_paths,
        file_picker_form_value_change,
    };
    use gpui_form_runtime::shape::ValueChange;

    #[test]
    fn empty_file_picker_change_clears_form_value() {
        let change = file_picker_form_value_change(&FilePickerEvent::Change(Vec::new()));

        assert!(matches!(change, ValueChange::Clear));
    }

    #[test]
    fn file_picker_options_are_shape_builders() {
        fn accepts_file_picker_builder(
            _: impl component_shape_gpui::GpuiComponentShapeBuilder<FilePicker>,
        ) {
        }

        let options = FilePicker::from(
            FilePickerOptions::builder()
                .mode(FilePickerMode::Directory)
                .multiple(true)
                .build(),
        );

        assert_eq!(options.mode(), FilePickerMode::Directory);
        assert!(options.multiple());
        accepts_file_picker_builder(options);
    }

    #[test]
    fn file_picker_modes_and_events_preserve_selection_contracts() {
        assert!(FilePickerMode::File.allows_files());
        assert!(!FilePickerMode::File.allows_directories());
        assert!(!FilePickerMode::Directory.allows_files());
        assert!(FilePickerMode::Directory.allows_directories());
        assert!(FilePickerMode::FileOrDirectory.allows_files());
        assert!(FilePickerMode::FileOrDirectory.allows_directories());

        let paths = vec!["one.txt".into(), "two.txt".into()];
        assert!(matches!(
            file_picker_form_value_change(&FilePickerEvent::Change(paths.clone())),
            ValueChange::Set(value) if value == paths
        ));
        assert!(matches!(
            file_picker_form_value_change(&FilePickerEvent::Cancel),
            ValueChange::Unchanged
        ));
        assert!(matches!(
            file_picker_form_value_change(&FilePickerEvent::Error("failed".into())),
            ValueChange::Unchanged
        ));

        let multiple_label = |count| format!("{count} paths selected").into();
        assert_eq!(
            display_paths(&[], "Choose".into(), multiple_label).as_ref(),
            "Choose"
        );
        assert_eq!(
            display_paths(&["one.txt".into()], "Choose".into(), multiple_label).as_ref(),
            "one.txt"
        );
        assert_eq!(
            display_paths(&paths, "Choose".into(), multiple_label).as_ref(),
            "2 paths selected"
        );
    }
}
