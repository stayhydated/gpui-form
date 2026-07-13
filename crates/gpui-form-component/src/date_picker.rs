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

impl DatePickerState {
    /// Create a new date state.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let calendar = cx.new(|cx| {
            let mut this = CalendarState::new(window, cx);
            this.set_date(CalendarDate::Single(None), window, cx);
            this
        });

        let subscriptions = vec![cx.subscribe_in(
            &calendar,
            window,
            |this, _, ev: &CalendarEvent, window, cx| match ev {
                CalendarEvent::Selected(CalendarDate::Single(date)) => {
                    this.update_date(date.and_then(jiff_date_from_chrono), true, window, cx);
                    this.focus_handle.focus(window, cx);
                },
                CalendarEvent::Selected(CalendarDate::Range(_, _)) => {},
            },
        )];

        Self {
            focus_handle: cx.focus_handle(),
            date: None,
            open: false,
            calendar,
            display_locale: None,
            display_style: DateDisplayStyle::default(),
            _subscriptions: subscriptions,
        }
    }

    /// Get the selected date.
    pub fn date(&self) -> Option<JiffDate> {
        self.date
    }

    /// Set the selected date.
    pub fn set_date(
        &mut self,
        date: impl Into<Option<JiffDate>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.update_date(date.into(), false, window, cx);
    }

    /// Override the locale used for display formatting.
    pub fn display_locale(mut self, locale: impl Into<Locale>) -> Self {
        self.display_locale = Some(locale.into());
        self
    }

    /// Set the localized display width used for the input text.
    pub fn display_style(mut self, display_style: DateDisplayStyle) -> Self {
        self.display_style = display_style;
        self
    }

    fn update_date(
        &mut self,
        date: Option<JiffDate>,
        emit: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.date = date;
        self.calendar.update(cx, |state, cx| {
            state.set_date(
                CalendarDate::Single(date.and_then(chrono_date_from_jiff)),
                window,
                cx,
            );
        });
        self.open = false;
        if emit {
            cx.emit(DatePickerEvent::Change(date));
        }
        cx.notify();
    }

    fn close_calendar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_back_if_need(window, cx);
        self.open = false;
        cx.notify();
    }

    fn focus_back_if_need(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }

        if let Some(focused) = window.focused(cx)
            && focused.contains(&self.focus_handle, window)
        {
            self.focus_handle.focus(window, cx);
        }
    }

    fn clean(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.update_date(None, true, window, cx);
    }

    fn toggle_calendar(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        cx.notify();
    }
}

impl DateRangePickerState {
    /// Create a new date range state.
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let calendar = cx.new(|cx| {
            let mut this = CalendarState::new(window, cx);
            this.set_date(CalendarDate::Range(None, None), window, cx);
            this
        });

        let subscriptions = vec![cx.subscribe_in(
            &calendar,
            window,
            |this, _, ev: &CalendarEvent, window, cx| match ev {
                CalendarEvent::Selected(CalendarDate::Range(start_date, end_date)) => {
                    this.update_range(
                        start_date.and_then(jiff_date_from_chrono),
                        end_date.and_then(jiff_date_from_chrono),
                        true,
                        window,
                        cx,
                    );
                    this.focus_handle.focus(window, cx);
                },
                CalendarEvent::Selected(CalendarDate::Single(_)) => {},
            },
        )];

        Self {
            focus_handle: cx.focus_handle(),
            start_date: None,
            end_date: None,
            open: false,
            calendar,
            display_locale: None,
            display_style: DateDisplayStyle::default(),
            _subscriptions: subscriptions,
        }
    }

    /// Get the selected date range.
    pub fn range(&self) -> (Option<JiffDate>, Option<JiffDate>) {
        (self.start_date, self.end_date)
    }

    /// Set the selected date range.
    pub fn set_range(
        &mut self,
        start_date: impl Into<Option<JiffDate>>,
        end_date: impl Into<Option<JiffDate>>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.update_range(start_date.into(), end_date.into(), false, window, cx);
    }

    /// Override the locale used for display formatting.
    pub fn display_locale(mut self, locale: impl Into<Locale>) -> Self {
        self.display_locale = Some(locale.into());
        self
    }

    /// Set the localized display width used for the input text.
    pub fn display_style(mut self, display_style: DateDisplayStyle) -> Self {
        self.display_style = display_style;
        self
    }

    fn update_range(
        &mut self,
        start_date: Option<JiffDate>,
        end_date: Option<JiffDate>,
        emit: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.start_date = start_date;
        self.end_date = end_date;
        self.calendar.update(cx, |state, cx| {
            state.set_date(
                CalendarDate::Range(
                    start_date.and_then(chrono_date_from_jiff),
                    end_date.and_then(chrono_date_from_jiff),
                ),
                window,
                cx,
            );
        });
        self.open = false;
        if emit {
            cx.emit(DateRangePickerEvent::Change(start_date, end_date));
        }
        cx.notify();
    }

    fn close_calendar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_back_if_need(window, cx);
        self.open = false;
        cx.notify();
    }

    fn focus_back_if_need(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }

        if let Some(focused) = window.focused(cx)
            && focused.contains(&self.focus_handle, window)
        {
            self.focus_handle.focus(window, cx);
        }
    }

    fn clean(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.update_range(None, None, true, window, cx);
    }

    fn toggle_calendar(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.open = !self.open;
        cx.notify();
    }
}

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
                            .when(is_focused, |this| this.focused_border(cx))
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
                            .when(is_focused, |this| this.focused_border(cx))
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

/// Parse a selected date into a form field value using its `FromStr` implementation.
pub fn parse_form_date<T>(date: JiffDate) -> Option<T>
where
    T: std::str::FromStr,
{
    T::from_str(&date.to_string()).ok()
}

fn chrono_date_from_jiff(date: JiffDate) -> Option<NaiveDate> {
    NaiveDate::from_ymd_opt(
        i32::from(date.year()),
        u32::from(date.month().unsigned_abs()),
        u32::from(date.day().unsigned_abs()),
    )
}

fn jiff_date_from_chrono(date: NaiveDate) -> Option<JiffDate> {
    let year = i16::try_from(date.year()).ok()?;
    let month = i8::try_from(date.month()).ok()?;
    let day = i8::try_from(date.day()).ok()?;
    JiffDate::new(year, month, day).ok()
}

#[cfg(feature = "component-shape")]
fn jiff_date_range_from_chrono(
    start: JiffDate,
    end: JiffDate,
) -> Option<(chrono::NaiveDate, chrono::NaiveDate)> {
    Some((parse_form_date(start)?, parse_form_date(end)?))
}

fn active_locale() -> Locale {
    let raw = gpui_component::locale();
    let normalized = raw.deref().replace('_', "-");
    Locale::from_str(&normalized).unwrap_or_else(|error| {
        panic!("gpui-component locale `{normalized}` is not a valid ICU locale: {error}")
    })
}

fn format_display_date(
    date: JiffDate,
    locale: &Locale,
    display_style: DateDisplayStyle,
) -> Option<SharedString> {
    let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.clone().into(),
        match display_style {
            DateDisplayStyle::Short => fieldsets::YMD::short(),
            DateDisplayStyle::Medium => fieldsets::YMD::medium(),
            DateDisplayStyle::Long => fieldsets::YMD::long(),
        },
    )
    .unwrap_or_else(|error| {
        panic!("failed to create date formatter for locale `{locale}`: {error:?}")
    });

    let icu_date = IcuDate::try_new_gregorian(
        i32::from(date.year()),
        date.month().unsigned_abs(),
        date.day().unsigned_abs(),
    )
    .ok()?;

    Some(formatter.format(&icu_date).to_string().into())
}

fn format_display_range(
    start_date: Option<JiffDate>,
    end_date: Option<JiffDate>,
    locale: &Locale,
    display_style: DateDisplayStyle,
) -> Option<SharedString> {
    match (start_date, end_date) {
        (None, None) => None,
        (Some(start_date), Some(end_date)) => Some(
            format!(
                "{} - {}",
                format_display_date(start_date, locale, display_style)?.as_ref(),
                format_display_date(end_date, locale, display_style)?.as_ref()
            )
            .into(),
        ),
        (Some(start_date), None) => Some(
            format!(
                "{} - ...",
                format_display_date(start_date, locale, display_style)?.as_ref()
            )
            .into(),
        ),
        (None, Some(end_date)) => Some(
            format!(
                "... - {}",
                format_display_date(end_date, locale, display_style)?.as_ref()
            )
            .into(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use icu_locale_core::locale;
    use jiff::civil::date;

    use super::{DateDisplayStyle, format_display_date, format_display_range, parse_form_date};

    #[test]
    fn formats_dates_with_icu4x() {
        let formatted = format_display_date(
            date(2025, 1, 15),
            &locale!("en-US"),
            DateDisplayStyle::Medium,
        )
        .expect("formatted date");

        assert_eq!(formatted.to_string(), "Jan 15, 2025");
    }

    #[test]
    fn parses_dates_through_jiff_display() {
        let parsed = parse_form_date::<NaiveDate>(date(2025, 1, 15)).expect("parsed date");

        assert_eq!(parsed, NaiveDate::from_ymd_opt(2025, 1, 15).unwrap());
    }

    #[test]
    fn formats_date_ranges_with_icu4x() {
        let formatted = format_display_range(
            Some(date(2025, 1, 15)),
            Some(date(2025, 1, 20)),
            &locale!("en-US"),
            DateDisplayStyle::Medium,
        )
        .expect("formatted date range");

        assert_eq!(formatted.to_string(), "Jan 15, 2025 - Jan 20, 2025");
    }

    #[test]
    fn formats_partial_ranges_and_all_display_widths() {
        let value = date(2025, 1, 15);
        assert_eq!(
            format_display_range(
                Some(value),
                None,
                &locale!("en-US"),
                DateDisplayStyle::Short,
            )
            .unwrap()
            .to_string(),
            "1/15/25 - ..."
        );
        assert_eq!(
            format_display_range(None, Some(value), &locale!("en-US"), DateDisplayStyle::Long,)
                .unwrap()
                .to_string(),
            "... - January 15, 2025"
        );
        assert!(
            format_display_range(None, None, &locale!("en-US"), DateDisplayStyle::Medium).is_none()
        );
    }
}
