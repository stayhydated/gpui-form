//! Runtime file picker support backed by GPUI's native path prompt.
//!
//! This module intentionally uses `gpui::App::prompt_for_paths` from the
//! pinned GPUI git dependency instead of adding a second native-dialog
//! dependency. The rendered control follows `gpui-component` styling and emits
//! form-friendly change events.

use std::path::PathBuf;

use gpui::{
    App, ClickEvent, Context, ElementId, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, ParentElement as _, PathPromptOptions, Render,
    RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    div, prelude::FluentBuilder as _,
};
use gpui_component::{
    ActiveTheme as _, Disableable, Icon, IconName, Sizable, Size, StyleSized as _, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex,
};

use crate::i18n::FilePickerText;
#[cfg(feature = "derive")]
use crate::shape::{ComponentValueBinding, FormValueChange, OwnedComponentValueBinding};

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

    fn default_placeholder(self) -> SharedString {
        let message = match self {
            Self::File => FilePickerText::SelectAFile,
            Self::Directory => FilePickerText::SelectADirectory,
            Self::FileOrDirectory => FilePickerText::SelectAFileOrDirectory,
        };
        message.default_text().into()
    }

    fn default_prompt(self) -> SharedString {
        let message = match self {
            Self::File => FilePickerText::SelectFile,
            Self::Directory => FilePickerText::SelectDirectory,
            Self::FileOrDirectory => FilePickerText::SelectFileOrDirectory,
        };
        message.default_text().into()
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

/// State for a native file picker control.
#[cfg_attr(feature = "derive", derive(gpui_form_derive::ComponentShape))]
#[cfg_attr(
    feature = "derive",
    gpui_form_shape(
        component = gpui_form_component::file_picker::FilePicker,
        value_binding,
        field_suffix = "file_picker",
        shape_crate = crate
    )
)]
pub struct FilePickerState {
    focus_handle: FocusHandle,
    paths: Vec<PathBuf>,
    last_error: Option<SharedString>,
}

#[cfg(feature = "derive")]
impl ComponentValueBinding<Vec<PathBuf>> for FilePickerState {
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

#[cfg(feature = "derive")]
impl OwnedComponentValueBinding<Vec<PathBuf>> for FilePickerState {}

impl Focusable for FilePickerState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<FilePickerEvent> for FilePickerState {}

impl FilePickerState {
    /// Create an empty file-picker state.
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            paths: Vec::new(),
            last_error: None,
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
        gpui::Empty
    }
}

/// A native file picker element using `gpui-component` visual primitives.
#[derive(IntoElement)]
pub struct FilePicker {
    id: ElementId,
    style: StyleRefinement,
    state: Entity<FilePickerState>,
    mode: FilePickerMode,
    multiple: bool,
    prompt: Option<SharedString>,
    placeholder: Option<SharedString>,
    browse_label: Option<SharedString>,
    cleanable: bool,
    appearance: bool,
    disabled: bool,
    size: Size,
}

impl Sizable for FilePicker {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
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
        self.disabled = disabled;
        self
    }
}

impl FilePicker {
    /// Create a file-only picker with the given state.
    pub fn new(state: &Entity<FilePickerState>) -> Self {
        Self {
            id: ("file-picker", state.entity_id()).into(),
            state: state.clone(),
            mode: FilePickerMode::default(),
            multiple: false,
            prompt: None,
            placeholder: None,
            browse_label: None,
            cleanable: false,
            appearance: true,
            disabled: false,
            size: Size::default(),
            style: StyleRefinement::default(),
        }
    }

    /// Select files only.
    pub fn files(mut self) -> Self {
        self.mode = FilePickerMode::File;
        self
    }

    /// Select directories only.
    pub fn directories(mut self) -> Self {
        self.mode = FilePickerMode::Directory;
        self
    }

    /// Select either files or directories when the platform supports it.
    pub fn files_or_directories(mut self) -> Self {
        self.mode = FilePickerMode::FileOrDirectory;
        self
    }

    /// Set the picker mode.
    pub fn mode(mut self, mode: FilePickerMode) -> Self {
        self.mode = mode;
        self
    }

    /// Allow multiple selected paths.
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
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
        self.cleanable = cleanable;
        self
    }

    /// Set whether to render the picker with the default bordered input style.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }
}

impl RenderOnce for FilePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_focused = self.focus_handle(cx).contains_focused(window, cx);
        let entity_id = self.state.entity_id();
        let state = self.state.read(cx);
        let paths = state.paths.clone();
        let last_error = state.last_error.clone();
        let has_paths = !paths.is_empty();
        let show_clean = self.cleanable && has_paths;
        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| self.mode.default_placeholder());
        let display_title = display_paths(&paths, placeholder);
        let prompt = self
            .prompt
            .clone()
            .or_else(|| Some(self.mode.default_prompt()));
        let browse_label = self
            .browse_label
            .clone()
            .unwrap_or_else(|| FilePickerText::Browse.default_text().into());
        let text_state = self.state.clone();
        let text_prompt = prompt.clone();
        let browse_state = self.state.clone();
        let browse_prompt = prompt;

        div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle(cx).tab_stop(true))
            .flex_none()
            .w_full()
            .input_text_size(self.size)
            .refine_style(&self.style)
            .when(self.disabled, |this| this.opacity(0.5))
            .child(
                div()
                    .id("file-picker-input")
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .when(self.appearance, |this| {
                        this.bg(cx.theme().background)
                            .text_color(cx.theme().foreground)
                            .border_1()
                            .border_color(cx.theme().input)
                            .rounded(cx.theme().radius)
                            .when(cx.theme().shadow, |this| this.shadow_xs())
                            .when(is_focused, |this| this.focused_border(cx))
                    })
                    .input_text_size(self.size)
                    .input_size(self.size)
                    .overflow_hidden()
                    .child(
                        h_flex()
                            .id(("file-picker-display", entity_id))
                            .w_full()
                            .items_center()
                            .gap_2()
                            .overflow_hidden()
                            .when(!self.disabled, |this| this.cursor_pointer())
                            .when(!self.disabled, |this| {
                                this.on_click(move |_, window, cx| {
                                    prompt_for_selection(
                                        text_state.clone(),
                                        self.mode,
                                        self.multiple,
                                        text_prompt.clone(),
                                        window,
                                        cx,
                                    );
                                })
                            })
                            .child(
                                Icon::new(self.mode.icon_name())
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
                    .when(!self.disabled && show_clean, |this| {
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
                            .with_size(self.size)
                            .secondary()
                            .icon(self.mode.icon_name())
                            .label(browse_label)
                            .disabled(self.disabled)
                            .on_click(move |_, window, cx| {
                                prompt_for_selection(
                                    browse_state.clone(),
                                    self.mode,
                                    self.multiple,
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
    let directories =
        mode.allows_directories() && (!mode.allows_files() || cx.can_select_mixed_files_and_dirs());
    let paths = cx.prompt_for_paths(PathPromptOptions {
        files: mode.allows_files(),
        directories,
        multiple,
        prompt,
    });

    window
        .spawn(cx, async move |cx| {
            let result = paths.await;

            _ = state.update_in(cx, |this, window, cx| match result {
                Ok(Ok(Some(paths))) => this.replace_paths(paths, true, window, cx),
                Ok(Ok(None)) => this.emit_cancel(cx),
                Ok(Err(error)) => this.emit_error(error.to_string(), cx),
                Err(_) => this.emit_error(FilePickerText::DialogDropped.default_text(), cx),
            });
        })
        .detach();
}

fn display_paths(paths: &[PathBuf], placeholder: SharedString) -> SharedString {
    match paths {
        [] => placeholder,
        [path] => path.display().to_string().into(),
        _ => FilePickerText::PathsSelected { count: paths.len() }
            .default_text()
            .into(),
    }
}
