use super::*;

/// Parse a selected date into a form field value using its `FromStr` implementation.
pub fn parse_form_date<T>(date: JiffDate) -> Option<T>
where
    T: std::str::FromStr,
{
    T::from_str(&date.to_string()).ok()
}

pub(super) fn active_locale() -> Locale {
    let raw = gpui_kit::component::locale();
    let normalized = raw.deref().replace('_', "-");
    Locale::from_str(&normalized).unwrap_or_else(|error| {
        panic!("GPUI Kit component locale `{normalized}` is not a valid ICU locale: {error}")
    })
}

pub(super) fn format_display_date(
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

pub(super) fn format_display_range(
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
