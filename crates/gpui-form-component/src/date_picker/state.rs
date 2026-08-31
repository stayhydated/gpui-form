use super::*;

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

    pub(super) fn close_calendar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    pub(super) fn clean(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.update_date(None, true, window, cx);
    }

    pub(super) fn toggle_calendar(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
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

    pub(super) fn close_calendar(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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

    pub(super) fn clean(&mut self, _: &ClickEvent, window: &mut Window, cx: &mut Context<Self>) {
        cx.stop_propagation();
        self.update_range(None, None, true, window, cx);
    }

    pub(super) fn toggle_calendar(
        &mut self,
        _: &ClickEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.open = !self.open;
        cx.notify();
    }
}
