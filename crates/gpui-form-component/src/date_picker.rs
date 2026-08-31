use std::{ops::Deref as _, str::FromStr as _};

use chrono::{Datelike as _, NaiveDate};
use gpui::{
    App, AppContext as _, ClickEvent, Context, ElementId, Empty, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement as _, IntoElement, MouseButton, ParentElement as _, Render,
    RenderOnce, SharedString, StatefulInteractiveElement as _, StyleRefinement, Styled,
    Subscription, Window, anchored, deferred, div, prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, Disableable, Icon, IconName, Sizable, Size, StyleSized as _, StyledExt as _,
    ThemeStyled as _,
    button::{Button, ButtonVariants as _},
    h_flex,
};
use gpui_es_fluent::localize_message;
use icu_calendar::{Date as IcuDate, Gregorian};
use icu_datetime::{FixedCalendarDateTimeFormatter, fieldsets};
use icu_locale_core::Locale;
use jiff::civil::Date as JiffDate;

use crate::calendar::{Calendar, CalendarEvent, CalendarState, Date as CalendarDate};
use crate::i18n::DatePickerText;
#[cfg(feature = "component-shape")]
use gpui_form_runtime::shape::ValueChange;

mod formatting;
mod state;

pub use formatting::parse_form_date;
use formatting::*;

/// Localized date display widths for the runtime date picker.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DateDisplayStyle {
    Short,
    #[default]
    Medium,
    Long,
}

/// Events emitted by the runtime date picker.
#[derive(Clone)]
pub enum DatePickerEvent {
    Change(Option<JiffDate>),
}

/// Events emitted by the runtime date range picker.
#[derive(Clone)]
pub enum DateRangePickerEvent {
    Change(Option<JiffDate>, Option<JiffDate>),
}

/// Use to store the state of the date picker.
pub struct DatePickerState {
    focus_handle: FocusHandle,
    date: Option<JiffDate>,
    open: bool,
    calendar: Entity<CalendarState>,
    display_locale: Option<Locale>,
    display_style: DateDisplayStyle,
    _subscriptions: Vec<Subscription>,
}

#[cfg(feature = "component-shape")]
impl gpui_form_runtime::shape::GpuiComponentStateValueBinding<chrono::NaiveDate>
    for DatePickerState
{
    type Event = DatePickerEvent;

    fn seed_value_binding_state(
        state: &mut Self,
        value: Option<&chrono::NaiveDate>,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        let date = value.and_then(|value| value.to_string().parse::<JiffDate>().ok());
        state.set_date(date, window, cx);
    }

    fn value_change(_state: &Self, event: &Self::Event) -> ValueChange<chrono::NaiveDate> {
        match event {
            DatePickerEvent::Change(Some(date)) => parse_form_date::<chrono::NaiveDate>(*date)
                .map_or(ValueChange::Unchanged, ValueChange::Set),
            DatePickerEvent::Change(None) => ValueChange::Clear,
        }
    }
}

impl Focusable for DatePickerState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DatePickerEvent> for DatePickerState {}

/// Use to store the state of the date range picker.
pub struct DateRangePickerState {
    focus_handle: FocusHandle,
    start_date: Option<JiffDate>,
    end_date: Option<JiffDate>,
    open: bool,
    calendar: Entity<CalendarState>,
    display_locale: Option<Locale>,
    display_style: DateDisplayStyle,
    _subscriptions: Vec<Subscription>,
}

#[cfg(feature = "component-shape")]
impl
    gpui_form_runtime::shape::GpuiComponentStateValueBinding<(chrono::NaiveDate, chrono::NaiveDate)>
    for DateRangePickerState
{
    type Event = DateRangePickerEvent;

    fn seed_value_binding_state(
        state: &mut Self,
        value: Option<&(chrono::NaiveDate, chrono::NaiveDate)>,
        window: &mut Window,
        cx: &mut Context<'_, Self>,
    ) {
        state.set_range(
            value.and_then(|(start, _)| jiff_date_from_chrono(*start)),
            value.and_then(|(_, end)| jiff_date_from_chrono(*end)),
            window,
            cx,
        );
    }

    fn value_change(
        _state: &Self,
        event: &Self::Event,
    ) -> ValueChange<(chrono::NaiveDate, chrono::NaiveDate)> {
        match event {
            DateRangePickerEvent::Change(Some(start), Some(end)) => {
                jiff_date_range_from_chrono(*start, *end)
                    .map_or(ValueChange::Unchanged, ValueChange::Set)
            },
            DateRangePickerEvent::Change(_, _) => ValueChange::Clear,
        }
    }
}

impl Focusable for DateRangePickerState {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<DateRangePickerEvent> for DateRangePickerState {}

impl Render for DatePickerState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

impl Render for DateRangePickerState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

/// A localized date picker element.
#[cfg_attr(
    feature = "component-shape",
    derive(component_shape_gpui::GpuiComponentShape)
)]
#[cfg_attr(
    feature = "component-shape",
    gpui_component_shape(
        state = DatePickerState,
        value = chrono::NaiveDate,
        field_suffix = "date_picker",
        value_binding
    )
)]
#[derive(IntoElement)]
pub struct DatePicker {
    id: ElementId,
    style: StyleRefinement,
    state: Entity<DatePickerState>,
    cleanable: bool,
    placeholder: Option<SharedString>,
    size: Size,
    number_of_months: usize,
    appearance: bool,
    disabled: bool,
}

#[cfg(feature = "component-shape")]
impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for DatePicker {
    type ValueStoragePolicy = gpui_form_runtime::shape::RequiredValueStorage;
}

/// A localized date range picker element.
#[cfg_attr(
    feature = "component-shape",
    derive(component_shape_gpui::GpuiComponentShape)
)]
#[cfg_attr(
    feature = "component-shape",
    gpui_component_shape(
        state = DateRangePickerState,
        value = (chrono::NaiveDate, chrono::NaiveDate),
        field_suffix = "date_range_picker",
        value_binding
    )
)]
#[derive(IntoElement)]
pub struct DateRangePicker {
    id: ElementId,
    style: StyleRefinement,
    state: Entity<DateRangePickerState>,
    cleanable: bool,
    placeholder: Option<SharedString>,
    size: Size,
    number_of_months: usize,
    appearance: bool,
    disabled: bool,
}

#[cfg(feature = "component-shape")]
impl gpui_form_runtime::shape::GpuiFormComponentShapePolicy for DateRangePicker {
    type ValueStoragePolicy = gpui_form_runtime::shape::RequiredValueStorage;
}

impl Sizable for DatePicker {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Sizable for DateRangePicker {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Focusable for DatePicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}

impl Focusable for DateRangePicker {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.state.focus_handle(cx)
    }
}

impl Styled for DatePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Styled for DateRangePicker {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl Disableable for DatePicker {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl Disableable for DateRangePicker {
    fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

impl DatePicker {
    /// Create a new `DatePicker` with the given `DatePickerState`.
    pub fn new(state: &Entity<DatePickerState>) -> Self {
        Self {
            id: ("date-picker", state.entity_id()).into(),
            state: state.clone(),
            cleanable: false,
            placeholder: None,
            size: Size::default(),
            style: StyleRefinement::default(),
            number_of_months: 2,
            appearance: true,
            disabled: false,
        }
    }

    /// Set the placeholder text, default: `Select date`.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set whether to show the clear button when a date is selected.
    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    /// Set the number of months to show, default is `2`.
    pub fn number_of_months(mut self, number_of_months: usize) -> Self {
        self.number_of_months = number_of_months;
        self
    }

    /// Set whether to render the picker with the default bordered input style.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }
}

impl DateRangePicker {
    /// Create a new `DateRangePicker` with the given `DateRangePickerState`.
    pub fn new(state: &Entity<DateRangePickerState>) -> Self {
        Self {
            id: ("date-range-picker", state.entity_id()).into(),
            state: state.clone(),
            cleanable: false,
            placeholder: None,
            size: Size::default(),
            style: StyleRefinement::default(),
            number_of_months: 2,
            appearance: true,
            disabled: false,
        }
    }

    /// Set the placeholder text, default: `Select date`.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set whether to show the clear button when a range is selected.
    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    /// Set the number of months to show, default is `2`.
    pub fn number_of_months(mut self, number_of_months: usize) -> Self {
        self.number_of_months = number_of_months;
        self
    }

    /// Set whether to render the picker with the default bordered input style.
    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }
}

impl RenderOnce for DatePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_focused = self.focus_handle(cx).contains_focused(window, cx);
        let state = self.state.read(cx);
        let show_clean = self.cleanable && state.date.is_some();
        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| localize_message(cx, &DatePickerText::SelectDate).into());
        let locale = state.display_locale.clone().unwrap_or_else(active_locale);
        let display_title = state
            .date
            .and_then(|date| format_display_date(date, &locale, state.display_style))
            .unwrap_or_else(|| placeholder.clone());

        div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle(cx).tab_stop(true))
            .flex_none()
            .w_full()
            .relative()
            .input_text_size(self.size)
            .refine_style(&self.style)
            .child(
                div()
                    .id("date-picker-input")
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .when(self.appearance, |this| {
                        this.bg(cx.theme().background)
                            .text_color(cx.theme().foreground)
                            .when(self.disabled, |this| this.opacity(0.5))
                            .border_1()
                            .border_color(cx.theme().input)
                            .rounded(cx.theme().radius)
                            .when(cx.theme().shadow, |this| this.shadow_xs())
                            .when(is_focused, |this| this.border_color(cx.theme().ring))
                    })
                    .when(is_focused && self.appearance && !self.disabled, |this| {
                        this.focus_ring_style(window, cx)
                    })
                    .overflow_hidden()
                    .input_text_size(self.size)
                    .input_size(self.size)
                    .when(!self.disabled, |this| {
                        this.on_click(
                            window.listener_for(&self.state, DatePickerState::toggle_calendar),
                        )
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_1()
                            .child(
                                div()
                                    .w_full()
                                    .overflow_hidden()
                                    .when(state.date.is_none(), |this| {
                                        this.text_color(cx.theme().muted_foreground)
                                    })
                                    .child(display_title),
                            )
                            .when(!self.disabled, |this| {
                                this.when(show_clean, |this| {
                                    this.child(
                                        Button::new(("clear-date", self.state.entity_id()))
                                            .small()
                                            .ghost()
                                            .icon(IconName::Close)
                                            .on_click(
                                                window.listener_for(
                                                    &self.state,
                                                    DatePickerState::clean,
                                                ),
                                            ),
                                    )
                                })
                                .when(!show_clean, |this| {
                                    this.child(
                                        Icon::new(IconName::Calendar)
                                            .xsmall()
                                            .text_color(cx.theme().muted_foreground),
                                    )
                                })
                            }),
                    ),
            )
            .when(state.open, |this| {
                this.child(
                    deferred(
                        anchored().snap_to_window_with_margin(px(8.)).child(
                            div()
                                .occlude()
                                .mt_1p5()
                                .p_3()
                                .border_1()
                                .border_color(cx.theme().border)
                                .shadow_lg()
                                .rounded((cx.theme().radius * 2.).min(px(8.)))
                                .bg(cx.theme().popover)
                                .text_color(cx.theme().popover_foreground)
                                .on_mouse_up_out(
                                    MouseButton::Left,
                                    window.listener_for(&self.state, |state, _, window, cx| {
                                        state.close_calendar(window, cx);
                                    }),
                                )
                                .child(
                                    Calendar::new(&state.calendar)
                                        .display_locale(locale.clone())
                                        .number_of_months(self.number_of_months)
                                        .border_0()
                                        .rounded_none()
                                        .p_0()
                                        .with_size(self.size),
                                ),
                        ),
                    )
                    .with_priority(2),
                )
            })
    }
}

impl RenderOnce for DateRangePicker {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let is_focused = self.focus_handle(cx).contains_focused(window, cx);
        let state = self.state.read(cx);
        let show_clean = self.cleanable && (state.start_date.is_some() || state.end_date.is_some());
        let placeholder = self
            .placeholder
            .clone()
            .unwrap_or_else(|| localize_message(cx, &DatePickerText::SelectDate).into());
        let locale = state.display_locale.clone().unwrap_or_else(active_locale);
        let display_title = format_display_range(
            state.start_date,
            state.end_date,
            &locale,
            state.display_style,
        )
        .unwrap_or_else(|| placeholder.clone());

        div()
            .id(self.id.clone())
            .track_focus(&self.focus_handle(cx).tab_stop(true))
            .flex_none()
            .w_full()
            .relative()
            .input_text_size(self.size)
            .refine_style(&self.style)
            .child(
                div()
                    .id("date-range-picker-input")
                    .relative()
                    .flex()
                    .items_center()
                    .justify_between()
                    .when(self.appearance, |this| {
                        this.bg(cx.theme().background)
                            .text_color(cx.theme().foreground)
                            .when(self.disabled, |this| this.opacity(0.5))
                            .border_1()
                            .border_color(cx.theme().input)
                            .rounded(cx.theme().radius)
                            .when(cx.theme().shadow, |this| this.shadow_xs())
                            .when(is_focused, |this| this.border_color(cx.theme().ring))
                    })
                    .when(is_focused && self.appearance && !self.disabled, |this| {
                        this.focus_ring_style(window, cx)
                    })
                    .overflow_hidden()
                    .input_text_size(self.size)
                    .input_size(self.size)
                    .when(!self.disabled, |this| {
                        this.on_click(
                            window.listener_for(&self.state, DateRangePickerState::toggle_calendar),
                        )
                    })
                    .child(
                        h_flex()
                            .w_full()
                            .items_center()
                            .justify_between()
                            .gap_1()
                            .child(
                                div()
                                    .w_full()
                                    .overflow_hidden()
                                    .when(
                                        state.start_date.is_none() && state.end_date.is_none(),
                                        |this| this.text_color(cx.theme().muted_foreground),
                                    )
                                    .child(display_title),
                            )
                            .when(!self.disabled, |this| {
                                this.when(show_clean, |this| {
                                    this.child(
                                        Button::new(("clear-date-range", self.state.entity_id()))
                                            .small()
                                            .ghost()
                                            .icon(IconName::Close)
                                            .on_click(window.listener_for(
                                                &self.state,
                                                DateRangePickerState::clean,
                                            )),
                                    )
                                })
                                .when(!show_clean, |this| {
                                    this.child(
                                        Icon::new(IconName::Calendar)
                                            .xsmall()
                                            .text_color(cx.theme().muted_foreground),
                                    )
                                })
                            }),
                    ),
            )
            .when(state.open, |this| {
                this.child(
                    deferred(
                        anchored().snap_to_window_with_margin(px(8.)).child(
                            div()
                                .occlude()
                                .mt_1p5()
                                .p_3()
                                .border_1()
                                .border_color(cx.theme().border)
                                .shadow_lg()
                                .rounded((cx.theme().radius * 2.).min(px(8.)))
                                .bg(cx.theme().popover)
                                .text_color(cx.theme().popover_foreground)
                                .on_mouse_up_out(
                                    MouseButton::Left,
                                    window.listener_for(&self.state, |state, _, window, cx| {
                                        state.close_calendar(window, cx);
                                    }),
                                )
                                .child(
                                    Calendar::new(&state.calendar)
                                        .display_locale(locale.clone())
                                        .number_of_months(self.number_of_months)
                                        .border_0()
                                        .rounded_none()
                                        .p_0()
                                        .with_size(self.size),
                                ),
                        ),
                    )
                    .with_priority(2),
                )
            })
    }
}

#[cfg(test)]
mod tests;
