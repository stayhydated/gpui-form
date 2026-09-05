use chrono::{Datelike as _, Duration, Local, NaiveDate};
use gpui_kit as gpui;
use gpui_kit::component::{
    ActiveTheme as _, Disableable as _, IconName, Selectable as _, Sizable, Size, StyledExt as _,
    button::{Button, ButtonVariants as _},
    h_flex, v_flex,
};
use gpui_kit::{
    App, ClickEvent, Context, Div, ElementId, Empty, Entity, EventEmitter, FocusHandle,
    InteractiveElement as _, IntoElement, ParentElement as _, Render, RenderOnce, SharedString,
    Stateful, StatefulInteractiveElement as _, StyleRefinement, Styled, Window,
    prelude::FluentBuilder as _, px, relative,
};
use icu_calendar::{
    Date as IcuDate, Gregorian,
    types::Weekday,
    week::{WeekInformation, WeekPreferences},
};
use icu_datetime::{FixedCalendarDateTimeFormatter, fieldsets};
use icu_locale_core::{Locale, locale};

use crate::calendar_navigation::CalendarNavigation;

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

    fn is_active(&self, v: &NaiveDate) -> bool {
        let v = *v;
        match self {
            Self::Single(d) => Some(v) == *d,
            Self::Range(start, end) => Some(v) == *start || Some(v) == *end,
        }
    }

    fn is_in_range(&self, v: &NaiveDate) -> bool {
        let v = *v;
        match self {
            Self::Range(Some(start), Some(end)) => v >= *start && v <= *end,
            Self::Range(_, _) => false,
            _ => false,
        }
    }

    fn select(self, date: NaiveDate) -> Self {
        match self {
            Self::Single(_) => Self::Single(Some(date)),
            Self::Range(None, None) => Self::Range(Some(date), None),
            Self::Range(Some(start), None) if date >= start => Self::Range(Some(start), Some(date)),
            Self::Range(_, _) => Self::Range(Some(date), None),
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
    navigation: CalendarNavigation,
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
            navigation: CalendarNavigation::new(
                today.year(),
                today.month() as u8,
                (today.year() - 50, today.year() + 50),
            ),
            today,
        }
    }

    /// Set the date of the calendar.
    pub fn set_date(&mut self, date: impl Into<Date>, _: &mut Window, cx: &mut Context<Self>) {
        let date = date.into();

        self.date = date;
        match self.date {
            Date::Single(Some(date)) => {
                self.navigation
                    .set_position(date.year(), date.month() as u8);
            },
            Date::Range(Some(start), _) => {
                self.navigation
                    .set_position(start.year(), start.month() as u8);
            },
            _ => {},
        }

        cx.notify()
    }

    fn offset_year_month(&self, offset_month: usize) -> (i32, u32) {
        self.navigation.offset_year_month(offset_month)
    }

    fn has_prev_year_page(&self) -> bool {
        self.navigation.has_previous_year_page()
    }

    fn has_next_year_page(&self) -> bool {
        self.navigation.has_next_year_page()
    }

    fn prev_year_page(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.navigation.previous_year_page() {
            return;
        }
        cx.notify()
    }

    fn next_year_page(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        if !self.navigation.next_year_page() {
            return;
        }
        cx.notify()
    }

    fn prev_month(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.navigation.previous_month();
        cx.notify()
    }

    fn next_month(&mut self, _: &ClickEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.navigation.next_month();
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
                    let selection = view.date.select(date);
                    view.set_date(selection, window, cx);
                    if selection.is_complete() {
                        cx.emit(CalendarEvent::Selected(selection));
                    }
                },
            ))
        })
    }

    fn render_header(&self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = self.state.read(cx);
        let current_year = state.navigation.current_year();
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
        let current_month = state.navigation.current_month();

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
                                    view.navigation.set_month((ix + 1) as u8);
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
        let current_year = state.navigation.current_year();
        let current_page_years = state.navigation.current_page_years().to_vec();

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
                                view.navigation.set_year(year);
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
        .unwrap_or_else(|error| {
            panic!("failed to resolve week information for locale `{locale}`: {error:?}")
        })
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
    .unwrap_or_else(|error| {
        panic!("failed to create weekday formatter for locale `{locale}`: {error:?}")
    });
    let first_weekday = first_weekday(locale);

    (0..7)
        .map(|offset| {
            let weekday = add_weekdays(first_weekday, offset);
            formatter.format(&weekday).to_string().into()
        })
        .collect()
}

fn month_names(locale: &Locale) -> Vec<SharedString> {
    (1..=12)
        .map(|month| format_month_name(2025, month, locale))
        .collect()
}

fn format_month_name(year: i32, month: u32, locale: &Locale) -> SharedString {
    let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.clone().into(),
        fieldsets::M::long(),
    )
    .unwrap_or_else(|error| {
        panic!("failed to create month formatter for locale `{locale}`: {error:?}")
    });
    let month = u8::try_from(month)
        .unwrap_or_else(|error| panic!("calendar month `{month}` is out of range: {error}"));
    let icu_date = IcuDate::try_new_gregorian(year, month, 1)
        .unwrap_or_else(|error| panic!("invalid Gregorian month `{year}-{month:02}`: {error:?}"));

    formatter.format(&icu_date).to_string().into()
}

fn format_year_name(year: i32, locale: &Locale) -> SharedString {
    let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.clone().into(),
        fieldsets::Y::medium(),
    )
    .unwrap_or_else(|error| {
        panic!("failed to create year formatter for locale `{locale}`: {error:?}")
    });
    let icu_date = IcuDate::try_new_gregorian(year, 1, 1)
        .unwrap_or_else(|error| panic!("invalid Gregorian year `{year}`: {error:?}"));

    formatter.format(&icu_date).to_string().into()
}

fn format_day_name(date: &NaiveDate, locale: &Locale) -> SharedString {
    let formatter = FixedCalendarDateTimeFormatter::<Gregorian, _>::try_new(
        locale.clone().into(),
        fieldsets::D::short(),
    )
    .unwrap_or_else(|error| {
        panic!("failed to create day formatter for locale `{locale}`: {error:?}")
    });
    let month = u8::try_from(date.month()).expect("chrono months fit ICU month values");
    let day = u8::try_from(date.day()).expect("chrono days fit ICU day values");
    let icu_date = IcuDate::try_new_gregorian(date.year(), month, day).unwrap_or_else(|error| {
        panic!("invalid Gregorian date `{date}` for ICU formatting: {error:?}")
    });

    formatter.format(&icu_date).to_string().into()
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

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use icu_calendar::types::Weekday;
    use icu_locale_core::locale;

    use super::{
        Date, ViewMode, add_weekdays, days_in_month, days_since_sunday, first_weekday,
        format_day_name, format_month_name, format_year_name, month_names, weekday_names,
    };

    #[test]
    fn date_modes_report_completion_ranges_and_display_values() {
        let before = NaiveDate::from_ymd_opt(2025, 1, 5).unwrap();
        let start = NaiveDate::from_ymd_opt(2025, 1, 10).unwrap();
        let end = NaiveDate::from_ymd_opt(2025, 1, 20).unwrap();

        let single = Date::from(start);
        assert!(single.is_complete());
        assert!(single.is_active(&start));
        assert!(!single.is_in_range(&start));
        assert_eq!(single.to_string(), "2025-01-10");
        assert_eq!(Date::Single(None).to_string(), "nil");

        let range = Date::from((start, end));
        assert!(range.is_complete());
        assert!(range.is_active(&end));
        assert!(range.is_in_range(&NaiveDate::from_ymd_opt(2025, 1, 15).unwrap()));
        assert_eq!(range.to_string(), "2025-01-10 - 2025-01-20");
        assert_eq!(Date::Range(None, None).to_string(), "nil");
        assert_eq!(
            Date::Range(Some(start), None).to_string(),
            "2025-01-10 - nil"
        );
        assert_eq!(Date::Range(None, Some(end)).to_string(), "nil - 2025-01-20");
        assert!(!Date::Range(Some(start), None).is_complete());
        assert!(!Date::Range(Some(start), None).is_in_range(&start));

        assert_eq!(Date::Single(None).select(start), Date::Single(Some(start)));
        assert_eq!(
            Date::Range(None, None).select(start),
            Date::Range(Some(start), None)
        );
        assert_eq!(
            Date::Range(Some(start), None).select(end),
            Date::Range(Some(start), Some(end))
        );
        assert_eq!(
            Date::Range(Some(start), None).select(before),
            Date::Range(Some(before), None)
        );
        assert_eq!(
            Date::Range(Some(start), Some(end)).select(before),
            Date::Range(Some(before), None)
        );
    }

    #[test]
    fn calendar_view_modes_and_icu_names_cover_all_variants() {
        assert!(ViewMode::Day.is_day());
        assert!(ViewMode::Month.is_month());
        assert!(ViewMode::Year.is_year());
        assert!(!ViewMode::Day.is_month());

        let weekdays = [
            Weekday::Sunday,
            Weekday::Monday,
            Weekday::Tuesday,
            Weekday::Wednesday,
            Weekday::Thursday,
            Weekday::Friday,
            Weekday::Saturday,
        ];
        let names = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
        let localized_names = weekday_names(&locale!("en-US"));
        for (index, weekday) in weekdays.into_iter().enumerate() {
            assert_eq!(days_since_sunday(weekday), index as u8);
            assert_eq!(localized_names[index].as_ref(), names[index]);
            assert_eq!(add_weekdays(Weekday::Sunday, index as u8), weekday);
        }

        let months = month_names(&locale!("en-US"));
        assert_eq!(months.len(), 12);
        assert_eq!(months[0].as_ref(), "January");
        assert_eq!(months[11].as_ref(), "December");
        assert_eq!(format_year_name(2025, &locale!("en-US")).as_ref(), "2025");
        assert_eq!(
            format_day_name(
                &NaiveDate::from_ymd_opt(2025, 1, 9).unwrap(),
                &locale!("en-US")
            )
            .as_ref(),
            "9"
        );
    }

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
