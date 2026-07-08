use chrono::{Datelike as _, Duration, Local, NaiveDate};
use gpui::{
    App, ClickEvent, Context, Div, ElementId, Empty, Entity, EventEmitter, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, RenderOnce, SharedString,
    Stateful, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    prelude::FluentBuilder as _, px, relative,
};
use gpui_component::{
    ActiveTheme as _, Disableable as _, IconName, Selectable as _, Sizable, Size, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use icu_calendar::{
    Date as IcuDate, Gregorian,
    types::Weekday,
    week::{WeekInformation, WeekPreferences},
};
use icu_datetime::{FixedCalendarDateTimeFormatter, fieldsets};
use icu_locale_core::{Locale, locale};

/// Events emitted by the calendar.
pub enum CalendarEvent {
    /// The user selected a date.
    Selected(Date),
}

/// The date of the calendar.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Date {
    Single(Option<NaiveDate>),
    Range(Option<NaiveDate>, Option<NaiveDate>),
}

impl std::fmt::Display for Date {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Single(Some(date)) => write!(f, "{}", date),
            Self::Single(None) => write!(f, "nil"),
            Self::Range(Some(start), Some(end)) => write!(f, "{} - {}", start, end),
            Self::Range(None, None) => write!(f, "nil"),
            Self::Range(Some(start), None) => write!(f, "{} - nil", start),
            Self::Range(None, Some(end)) => write!(f, "nil - {}", end),
        }
    }
}

impl From<NaiveDate> for Date {
    fn from(date: NaiveDate) -> Self {
        Self::Single(Some(date))
    }
}

impl From<(NaiveDate, NaiveDate)> for Date {
    fn from((start, end): (NaiveDate, NaiveDate)) -> Self {
        Self::Range(Some(start), Some(end))
    }
}

impl Date {
    fn is_complete(&self) -> bool {
        matches!(self, Self::Range(Some(_), Some(_)) | Self::Single(Some(_)))
    }

    fn start(&self) -> Option<NaiveDate> {
        match self {
            Self::Single(Some(date)) => Some(*date),
            Self::Range(Some(start), _) => Some(*start),
            _ => None,
        }
    }

    fn end(&self) -> Option<NaiveDate> {
        match self {
            Self::Range(_, Some(end)) => Some(*end),
            _ => None,
        }
    }

    fn is_active(&self, v: &NaiveDate) -> bool {
        let v = *v;
        match self {
            Self::Single(d) => Some(v) == *d,
            Self::Range(start, end) => Some(v) == *start || Some(v) == *end,
        }
    }

    fn is_single(&self) -> bool {
        matches!(self, Self::Single(_))
    }

    fn is_in_range(&self, v: &NaiveDate) -> bool {
        let v = *v;
        match self {
            Self::Range(Some(start), Some(end)) => v >= *start && v <= *end,
            Self::Range(_, _) => false,
            _ => false,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ViewMode {
    Day,
    Month,
    Year,
}

impl ViewMode {
    fn is_day(&self) -> bool {
        matches!(self, Self::Day)
    }

    fn is_month(&self) -> bool {
        matches!(self, Self::Month)
    }

    fn is_year(&self) -> bool {
        matches!(self, Self::Year)
    }
}

#[derive(IntoElement)]
pub struct Calendar {
    id: ElementId,
    size: Size,
    state: Entity<CalendarState>,
    style: StyleRefinement,
    number_of_months: usize,
    display_locale: Locale,
}

/// Use to store the state of the calendar.
pub struct CalendarState {
    focus_handle: FocusHandle,
    view_mode: ViewMode,
    date: Date,
    current_year: i32,
    current_month: u8,
    years: Vec<Vec<i32>>,
    year_page: i32,
    today: NaiveDate,
}

impl CalendarState {
    /// Create a new calendar state.
    pub fn new(_: &mut Window, cx: &mut Context<Self>) -> Self {
        let today = Local::now().naive_local().date();
        Self {
            focus_handle: cx.focus_handle(),
            view_mode: ViewMode::Day,
            date: Date::Single(None),
            current_month: today.month() as u8,
            current_year: today.year(),
            years: vec![],
            year_page: 0,
            today,
        }
        .year_range((today.year() - 50, today.year() + 50))
    }

    /// Set the date of the calendar.
    pub fn set_date(&mut self, date: impl Into<Date>, _: &mut Window, cx: &mut Context<Self>) {
        let date = date.into();

        self.date = date;
        match self.date {
            Date::Single(Some(date)) => {
                self.current_month = date.month() as u8;
                self.current_year = date.year();
            },
            Date::Range(Some(start), _) => {
                self.current_month = start.month() as u8;
                self.current_year = start.year();
            },
            _ => {},
        }

        cx.notify()
    }

    /// Get the date of the calendar.
    pub fn date(&self) -> Date {
        self.date
    }

    /// Set the year range of the calendar, default is 50 years before and after the current year.
    pub fn year_range(mut self, range: (i32, i32)) -> Self {
        self.apply_year_range(range);
        self
    }

    fn apply_year_range(&mut self, range: (i32, i32)) {
        self.years = (range.0..range.1)
            .collect::<Vec<_>>()
            .chunks(20)
            .map(|chunk| chunk.to_vec())
            .collect::<Vec<_>>();
        self.year_page = self
            .years
            .iter()
            .position(|years| years.contains(&self.current_year))
            .unwrap_or(0) as i32;
    }

    fn offset_year_month(&self, offset_month: usize) -> (i32, u32) {
        let mut month = self.current_month as i32 + offset_month as i32;
        let mut year = self.current_year;
        while month < 1 {
            month += 12;
            year -= 1;
        }
        while month > 12 {
            month -= 12;
            year += 1;
        }

        (year, month as u32)
    }

    fn has_prev_year_page(&self) -> bool {
        self.year_page > 0
    }

    fn has_next_year_page(&self) -> bool {
        self.year_page < self.years.len() as i32 - 1
    }

    fn prev_year_page(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.has_prev_year_page() {
            return;
        }

        self.year_page -= 1;
        cx.notify()
    }

    fn next_year_page(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.has_next_year_page() {
            return;
        }

        self.year_page += 1;
        cx.notify()
    }

    fn prev_month(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.current_month = if self.current_month == 1 {
            12
        } else {
            self.current_month - 1
        };
        self.current_year = if self.current_month == 12 {
            self.current_year - 1
        } else {
            self.current_year
        };
        cx.notify()
    }

    fn next_month(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.current_month = if self.current_month == 12 {
            1
        } else {
            self.current_month + 1
        };
        self.current_year = if self.current_month == 1 {
            self.current_year + 1
        } else {
            self.current_year
        };
        cx.notify()
    }

    fn month_name(&self, offset_month: usize, locale: &Locale) -> SharedString {
        let (year, month) = self.offset_year_month(offset_month);
        format_month_name(year, month, locale)
    }

    fn year_name(&self, offset_month: usize, locale: &Locale) -> SharedString {
        let (year, _) = self.offset_year_month(offset_month);
        format_year_name(year, locale)
    }

    fn set_view_mode(&mut self, mode: ViewMode, _: &mut Window, cx: &mut Context<Self>) {
        self.view_mode = mode;
        cx.notify();
    }
}

impl Render for CalendarState {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Empty
    }
}

impl Calendar {
    /// Create a new calendar element with [`CalendarState`].
    pub fn new(state: &Entity<CalendarState>) -> Self {
        Self {
            id: ("calendar", state.entity_id()).into(),
            size: Size::default(),
            state: state.clone(),
            style: StyleRefinement::default(),
            number_of_months: 1,
            display_locale: locale!("en-US"),
        }
    }

    /// Set number of months to show, default is 1.
    pub fn number_of_months(mut self, number_of_months: usize) -> Self {
        self.number_of_months = number_of_months;
        self
    }

    /// Set the locale used for calendar labels and week layout.
    pub fn display_locale(mut self, locale: impl Into<Locale>) -> Self {
        self.display_locale = locale.into();
        self
    }

    fn render_day(
        &self,
        d: &NaiveDate,
        offset_month: usize,
        window: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        let state = self.state.read(cx);
        let (_, month) = state.offset_year_month(offset_month);
        let day = format_day_name(d, &self.display_locale);
        let is_current_month = d.month() == month;
        let is_active = state.date.is_active(d);
        let is_in_range = state.date.is_in_range(d);

        let date = *d;
        let is_today = *d == state.today;
        let disabled = false;

        let date_id: SharedString = format!("{}_{}", date.format("%Y-%m-%d"), offset_month).into();

        self.item_button(
            date_id,
            day,
            is_active,
            is_in_range,
            !is_current_month || disabled,
            disabled,
            window,
            cx,
        )
        .when(is_today && !is_active, |this| {
            this.border_1().border_color(cx.theme().border)
        })
        .when(!disabled, |this| {
            this.on_click(window.listener_for(
                &self.state,
                move |view, _: &ClickEvent, window, cx| {
                    if view.date.is_single() {
                        view.set_date(date, window, cx);
                        cx.emit(CalendarEvent::Selected(view.date()));
                    } else {
                        let start = view.date.start();
                        let end = view.date.end();

                        if start.is_none() && end.is_none() {
                            view.set_date(Date::Range(Some(date), None), window, cx);
                        } else if let (Some(start), None) = (start, end) {
                            if date < start {
                                view.set_date(Date::Range(Some(date), None), window, cx);
                            } else {
                                view.set_date(Date::Range(Some(start), Some(date)), window, cx);
                            }
                        } else {
                            view.set_date(Date::Range(Some(date), None), window, cx);
                        }

                        if view.date.is_complete() {
                            cx.emit(CalendarEvent::Selected(view.date()));
                        }
                    }
                },
            ))
        })
    }

    fn render_header(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.read(cx);
        let current_year = state.current_year;
        let view_mode = state.view_mode;
        let disabled = view_mode.is_month();
        let multiple_months = self.number_of_months > 1;
        let icon_size = match self.size {
            Size::Small => Size::Small,
            Size::Large => Size::Medium,
            _ => Size::Medium,
        };

        h_flex()
            .gap_0p5()
            .justify_between()
            .items_center()
            .child(
                Button::new("prev")
                    .icon(IconName::ArrowLeft)
                    .tab_stop(false)
                    .ghost()
                    .disabled(disabled)
                    .with_size(icon_size)
                    .when(view_mode.is_day(), |this| {
                        this.on_click(window.listener_for(&self.state, CalendarState::prev_month))
                    })
                    .when(view_mode.is_year(), |this| {
                        this.when(!state.has_prev_year_page(), |this| this.disabled(true))
                            .on_click(
                                window.listener_for(&self.state, CalendarState::prev_year_page),
                            )
                    }),
            )
            .when(!multiple_months, |this| {
                this.child(
                    h_flex()
                        .justify_center()
                        .gap_3()
                        .child(
                            Button::new("month")
                                .ghost()
                                .label(state.month_name(0, &self.display_locale))
                                .compact()
                                .tab_stop(false)
                                .with_size(self.size)
                                .selected(view_mode.is_month())
                                .on_click(window.listener_for(
                                    &self.state,
                                    move |view, _, window, cx| {
                                        if view_mode.is_month() {
                                            view.set_view_mode(ViewMode::Day, window, cx);
                                        } else {
                                            view.set_view_mode(ViewMode::Month, window, cx);
                                        }
                                        cx.notify();
                                    },
                                )),
                        )
                        .child(
                            Button::new("year")
                                .ghost()
                                .label(format_year_name(current_year, &self.display_locale))
                                .compact()
                                .tab_stop(false)
                                .with_size(self.size)
                                .selected(view_mode.is_year())
                                .on_click(window.listener_for(
                                    &self.state,
                                    |view, _, window, cx| {
                                        if view.view_mode.is_year() {
                                            view.set_view_mode(ViewMode::Day, window, cx);
                                        } else {
                                            view.set_view_mode(ViewMode::Year, window, cx);
                                        }
                                        cx.notify();
                                    },
                                )),
                        ),
                )
            })
            .when(multiple_months, |this| {
                this.child(h_flex().flex_1().justify_around().children(
                    (0..self.number_of_months).map(|n| {
                        h_flex()
                            .justify_center()
                            .map(|this| match self.size {
                                Size::Small => this.gap_2(),
                                Size::Large => this.gap_4(),
                                _ => this.gap_3(),
                            })
                            .child(state.month_name(n, &self.display_locale))
                            .child(state.year_name(n, &self.display_locale))
                    }),
                ))
            })
            .child(
                Button::new("next")
                    .icon(IconName::ArrowRight)
                    .ghost()
                    .tab_stop(false)
                    .disabled(disabled)
                    .with_size(icon_size)
                    .when(view_mode.is_day(), |this| {
                        this.on_click(window.listener_for(&self.state, CalendarState::next_month))
                    })
                    .when(view_mode.is_year(), |this| {
                        this.when(!state.has_next_year_page(), |this| this.disabled(true))
                            .on_click(
                                window.listener_for(&self.state, CalendarState::next_year_page),
                            )
                    }),
            )
    }

    #[allow(clippy::too_many_arguments)]
    fn item_button(
        &self,
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        active: bool,
        secondary_active: bool,
        muted: bool,
        disabled: bool,
        _: &mut Window,
        cx: &mut App,
    ) -> Stateful<Div> {
        h_flex()
            .id(id.into())
            .map(|this| match self.size {
                Size::Small => this.size_7().rounded(cx.theme().radius / 2.),
                Size::Large => this.size_10().rounded(cx.theme().radius * 2.),
                _ => this.size_9().rounded(cx.theme().radius),
            })
            .justify_center()
            .when(muted, |this| {
                this.text_color(if disabled {
                    cx.theme().muted_foreground.opacity(0.3)
                } else {
                    cx.theme().muted_foreground
                })
            })
            .when(secondary_active, |this| {
                this.bg(if muted {
                    cx.theme().accent.opacity(0.5)
                } else {
                    cx.theme().accent
                })
                .text_color(cx.theme().accent_foreground)
            })
            .when(!active && !disabled, |this| {
                this.hover(|this| {
                    this.bg(cx.theme().accent)
                        .text_color(cx.theme().accent_foreground)
                })
            })
            .when(active, |this| {
                this.bg(cx.theme().primary)
                    .text_color(cx.theme().primary_foreground)
            })
            .child(label.into())
    }

    fn render_days(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let first_weekday = first_weekday(&self.display_locale);
        let weeks = weekday_names(&self.display_locale);
        let month_days = {
            let state = self.state.read(cx);
            (0..self.number_of_months)
                .map(|offset_month| {
                    let (year, month) = state.offset_year_month(offset_month);
                    (offset_month, days_in_month(year, month, first_weekday))
                })
                .collect::<Vec<_>>()
        };

        h_flex()
            .map(|this| match self.size {
                Size::Small => this.gap_3().text_sm(),
                Size::Large => this.gap_5().text_base(),
                _ => this.gap_4().text_sm(),
            })
            .justify_between()
            .children(month_days.iter().map(|(offset_month, days)| {
                v_flex()
                    .gap_0p5()
                    .child(
                        h_flex().gap_0p5().justify_between().children(
                            weeks
                                .iter()
                                .map(|week| self.render_week(week.clone(), window, cx)),
                        ),
                    )
                    .children(days.iter().map(|week| {
                        h_flex().gap_0p5().justify_between().children(
                            week.iter()
                                .map(|d| self.render_day(d, *offset_month, window, cx)),
                        )
                    }))
            }))
    }

    fn render_week(&self, week: impl Into<SharedString>, _: &mut Window, cx: &mut App) -> Div {
        h_flex()
            .map(|this| match self.size {
                Size::Small => this.size_7().rounded(cx.theme().radius / 2.0),
                Size::Large => this.size_10().rounded(cx.theme().radius),
                _ => this.size_9().rounded(cx.theme().radius),
            })
            .justify_center()
            .text_color(cx.theme().muted_foreground)
            .text_sm()
            .child(week.into())
    }

    fn render_months(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.read(cx);
        let months = month_names(&self.display_locale);
        let current_month = state.current_month;

        h_flex()
            .mt_3()
            .gap_0p5()
            .gap_y_3()
            .map(|this| match self.size {
                Size::Small => this.mt_2().gap_y_2().w(px(208.)),
                Size::Large => this.mt_4().gap_y_4().w(px(292.)),
                _ => this.mt_3().gap_y_3().w(px(264.)),
            })
            .justify_between()
            .flex_wrap()
            .children(
                months
                    .iter()
                    .enumerate()
                    .map(|(ix, month)| {
                        let active = (ix + 1) as u8 == current_month;

                        self.item_button(ix, month.clone(), active, false, false, false, window, cx)
                            .w(relative(0.3))
                            .text_sm()
                            .on_click(window.listener_for(
                                &self.state,
                                move |view, _, window, cx| {
                                    view.current_month = (ix + 1) as u8;
                                    view.set_view_mode(ViewMode::Day, window, cx);
                                    cx.notify();
                                },
                            ))
                    })
                    .collect::<Vec<_>>(),
            )
    }

    fn render_years(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.read(cx);
        let current_year = state.current_year;
        let current_page_years = &self.state.read(cx).years[state.year_page as usize].clone();

        h_flex()
            .id("years")
            .gap_0p5()
            .map(|this| match self.size {
                Size::Small => this.mt_2().gap_y_2().w(px(208.)),
                Size::Large => this.mt_4().gap_y_4().w(px(292.)),
                _ => this.mt_3().gap_y_3().w(px(264.)),
            })
            .justify_between()
            .flex_wrap()
            .children(
                current_page_years
                    .iter()
                    .enumerate()
                    .map(|(ix, year)| {
                        let year = *year;
                        let active = year == current_year;

                        self.item_button(
                            ix,
                            format_year_name(year, &self.display_locale),
                            active,
                            false,
                            false,
                            false,
                            window,
                            cx,
                        )
                        .w(relative(0.2))
                        .on_click(window.listener_for(
                            &self.state,
                            move |view, _, window, cx| {
                                view.current_year = year;
                                view.set_view_mode(ViewMode::Day, window, cx);
                                cx.notify();
                            },
                        ))
                    })
                    .collect::<Vec<_>>(),
            )
    }
}

impl Sizable for Calendar {
    fn with_size(mut self, size: impl Into<Size>) -> Self {
        self.size = size.into();
        self
    }
}

impl Styled for Calendar {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl EventEmitter<CalendarEvent> for CalendarState {}

impl RenderOnce for Calendar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let view_mode = self.state.read(cx).view_mode;

        v_flex()
            .id(self.id.clone())
            .track_focus(&self.state.read(cx).focus_handle)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius_lg)
            .p_3()
            .gap_0p5()
            .refine_style(&self.style)
            .child(self.render_header(window, cx))
            .child(
                v_flex()
                    .when(view_mode.is_day(), |this| {
                        this.child(self.render_days(window, cx))
                    })
                    .when(view_mode.is_month(), |this| {
                        this.child(self.render_months(window, cx))
                    })
                    .when(view_mode.is_year(), |this| {
                        this.child(self.render_years(window, cx))
                    }),
            )
    }
}

fn first_weekday(locale: &Locale) -> Weekday {
    WeekInformation::try_new(WeekPreferences::from(locale))
        .map(|info| info.first_weekday)
        .unwrap_or(Weekday::Sunday)
}

fn days_in_month(year: i32, month: u32, first_weekday: Weekday) -> Vec<Vec<NaiveDate>> {
    let first_day = NaiveDate::from_ymd_opt(year, month, 1).expect("valid calendar month");
    let next_month = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    }
    .expect("valid following calendar month");
    let days_in_month = next_month.signed_duration_since(first_day).num_days() as u32;
    let leading_days = (7 + first_day.weekday().num_days_from_sunday() as i32
        - days_since_sunday(first_weekday) as i32)
        % 7;
    let grid_start = first_day
        .checked_sub_signed(Duration::days(i64::from(leading_days)))
        .expect("valid calendar grid start");

    let row_count = ((leading_days as u32 + days_in_month).div_ceil(7)).max(6);
    (0..row_count)
        .map(|week| {
            (0..7)
                .map(|weekday| {
                    grid_start
                        .checked_add_signed(Duration::days(i64::from(week * 7 + weekday)))
                        .expect("valid calendar grid date")
                })
                .collect()
        })
        .collect()
}

fn weekday_names(locale: &Locale) -> Vec<SharedString> {
    let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.clone().into(),
        fieldsets::E::short(),
    )
    .ok();
    let first_weekday = first_weekday(locale);

    (0..7)
        .map(|offset| {
            let weekday = add_weekdays(first_weekday, offset);
            formatter
                .as_ref()
                .map(|formatter| formatter.format(&weekday).to_string().into())
                .unwrap_or_else(|| fallback_weekday_name(weekday))
        })
        .collect()
}

fn month_names(locale: &Locale) -> Vec<SharedString> {
    (1..=12)
        .map(|month| format_month_name(2025, month, locale))
        .collect()
}

fn format_month_name(year: i32, month: u32, locale: &Locale) -> SharedString {
    let formatted = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.clone().into(),
        fieldsets::M::long(),
    )
    .ok()
    .and_then(|formatter| {
        let month = u8::try_from(month).ok()?;
        let icu_date = IcuDate::try_new_gregorian(year, month, 1).ok()?;
        Some(formatter.format(&icu_date).to_string())
    });

    formatted
        .unwrap_or_else(|| fallback_month_name(month).to_string())
        .into()
}

fn format_year_name(year: i32, locale: &Locale) -> SharedString {
    let formatted = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.clone().into(),
        fieldsets::Y::medium(),
    )
    .ok()
    .and_then(|formatter| {
        let icu_date = IcuDate::try_new_gregorian(year, 1, 1).ok()?;
        Some(formatter.format(&icu_date).to_string())
    });

    formatted.unwrap_or_else(|| year.to_string()).into()
}

fn format_day_name(date: &NaiveDate, locale: &Locale) -> SharedString {
    let formatted = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.clone().into(),
        fieldsets::D::short(),
    )
    .ok()
    .and_then(|formatter| {
        let month = u8::try_from(date.month()).ok()?;
        let day = u8::try_from(date.day()).ok()?;
        let icu_date = IcuDate::try_new_gregorian(date.year(), month, day).ok()?;
        Some(formatter.format(&icu_date).to_string())
    });

    formatted.unwrap_or_else(|| date.day().to_string()).into()
}

fn add_weekdays(weekday: Weekday, offset: u8) -> Weekday {
    Weekday::from_days_since_sunday(days_since_sunday(weekday) as isize + isize::from(offset))
}

fn days_since_sunday(weekday: Weekday) -> u8 {
    match weekday {
        Weekday::Sunday => 0,
        Weekday::Monday => 1,
        Weekday::Tuesday => 2,
        Weekday::Wednesday => 3,
        Weekday::Thursday => 4,
        Weekday::Friday => 5,
        Weekday::Saturday => 6,
    }
}

fn fallback_weekday_name(weekday: Weekday) -> SharedString {
    match weekday {
        Weekday::Sunday => "Sun",
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
    }
    .into()
}

fn fallback_month_name(month: u32) -> &'static str {
    match month {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use icu_calendar::types::Weekday;
    use icu_locale_core::locale;

    use super::{days_in_month, first_weekday, format_month_name, weekday_names};

    #[test]
    fn formats_month_names_with_icu4x() {
        assert_eq!(
            format_month_name(2025, 1, &locale!("fr-FR")).to_string(),
            "janvier"
        );
    }

    #[test]
    fn orders_weekdays_by_locale() {
        assert_eq!(first_weekday(&locale!("en-US")), Weekday::Sunday);
        assert_eq!(first_weekday(&locale!("fr-FR")), Weekday::Monday);

        let weeks = weekday_names(&locale!("fr-FR"))
            .into_iter()
            .map(|name| name.to_string())
            .collect::<Vec<_>>();

        assert_eq!(weeks.first().map(String::as_str), Some("lun."));
    }

    #[test]
    fn lays_out_days_from_locale_first_weekday() {
        let us_days = days_in_month(2025, 9, Weekday::Sunday);
        assert_eq!(us_days[0][0], NaiveDate::from_ymd_opt(2025, 8, 31).unwrap());
        assert_eq!(us_days[0][1], NaiveDate::from_ymd_opt(2025, 9, 1).unwrap());

        let fr_days = days_in_month(2025, 9, Weekday::Monday);
        assert_eq!(fr_days[0][0], NaiveDate::from_ymd_opt(2025, 9, 1).unwrap());
        assert_eq!(fr_days[0][6], NaiveDate::from_ymd_opt(2025, 9, 7).unwrap());
    }
}
