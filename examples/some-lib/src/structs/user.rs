use anyhow::Context as _;
use es_fluent::{EsFluent, EsFluentLabel, EsFluentVariants};
use gpui_form::GpuiForm;
use gpui_form_collection_derive::SelectItem;
use jiff::{Timestamp as JiffTimestamp, civil::Date as CivilDate, tz::TimeZone};
use koruma::{Koruma, KorumaAllFluent};
use koruma_collection::{
    collection::NonEmptyValidation,
    format::EmailValidation,
    numeric::{NegativeValidation, PositiveValidation, RangeValidation},
    string::{PrefixValidation, SuffixValidation},
};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, Eq, EsFluent, PartialEq, SelectItem)]
#[select_item(fluent)]
pub enum PreferredLanguage {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, EnumIter, Eq, EsFluent, PartialEq, SelectItem)]
#[select_item(fluent)]
pub enum EnumCountry {
    #[default]
    UnitedStates,
    France,
    China,
}

#[derive(Clone, Debug, EsFluentLabel, EsFluentVariants, GpuiForm, Koruma, KorumaAllFluent)]
#[fluent_variants(keys = ["description", "label"])]
#[gpui_form(koruma(fluent))]
pub struct User {
    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    #[koruma(
        NonEmptyValidation::<_>,
        PrefixValidation::<_>.prefix("Xx"),
        SuffixValidation::<_>.suffix("xX")
    )]
    pub username: String,

    #[gpui_form(component(
        gpui_form_collection::input::Input::<_>,
        default = "test@example.com"
    ))]
    #[koruma(EmailValidation::<_>)]
    pub email: String,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    #[koruma(RangeValidation::<_>.min(18).max(167))]
    pub age: Option<u32>,

    #[gpui_form(component(
        gpui_form_collection::input::Input::<_>,
        default = 67
    ))]
    #[koruma(PositiveValidation::<_>)]
    pub balance: rust_decimal::Decimal,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    #[koruma(NegativeValidation::<_>)]
    pub debt: rust_decimal::Decimal,

    #[gpui_form(component(gpui_form_collection::number_input::NumberInput::<_>))]
    pub rating: Option<u32>,

    #[gpui_form(component(gpui_form_collection::slider::Slider))]
    pub attention_level: f32,

    #[gpui_form(component(gpui_form_collection::color_picker::ColorPicker))]
    pub brand_color: Option<gpui_kit::Hsla>,

    #[gpui_form(component(gpui_form_collection::otp_input::OtpInput::<_>))]
    pub otp_code: String,

    #[gpui_form(component(gpui_form_component::file_picker::FilePicker))]
    pub uploaded_files: Vec<std::path::PathBuf>,

    #[gpui_form(component(gpui_form_collection::date_picker::DateRangePicker))]
    pub holiday_range: Option<(jiff::civil::Date, jiff::civil::Date)>,

    #[gpui_form(component(gpui_form_collection::switch::Switch))]
    pub subscribe_newsletter: bool,

    #[gpui_form(component(gpui_form_collection::checkbox::Checkbox))]
    pub enable_notifications: bool,

    #[gpui_form(component(gpui_form_collection::select::Select::<_>.searchable(true)))]
    pub preferred: PreferredLanguage,

    #[gpui_form(component(
        gpui_form_collection::select::Select::<_>,
        default = EnumCountry::France
    ))]
    pub country: Option<EnumCountry>,

    #[gpui_form(component(
        gpui_form_collection::date_picker::DatePicker,
        value(
            type = jiff::civil::Date,
            from_source = to_form_datetime,
            into_source = to_model_timestamp,
        )
    ))]
    pub birth_date: Timestamp,

    #[gpui_form(skip)]
    #[fluent_variants(skip)]
    pub skip_me: bool,
}

#[derive(Clone, Debug)]
pub struct Timestamp {
    __timestamp_micros_since_unix_epoch__: i64,
}

impl Timestamp {
    pub fn parse_from_rfc3339(str: &str) -> anyhow::Result<Timestamp> {
        let instant = str.parse::<JiffTimestamp>().with_context(
            || "Invalid timestamp format. Expected RFC 3339 (e.g. '2025-02-10T15:45:30Z').",
        )?;
        let micros = i64::try_from(instant.as_nanosecond().div_euclid(1_000))
            .context("Timestamp is out of range")?;
        Ok(Self::from_micros_since_unix_epoch(micros))
    }
    pub fn from_micros_since_unix_epoch(micros: i64) -> Self {
        Self {
            __timestamp_micros_since_unix_epoch__: micros,
        }
    }
}

#[allow(dead_code)]
fn to_form_datetime(value: Timestamp) -> CivilDate {
    JiffTimestamp::from_microsecond(value.__timestamp_micros_since_unix_epoch__)
        .unwrap_or(JiffTimestamp::UNIX_EPOCH)
        .to_zoned_datetime(TimeZone::UTC)
        .date()
}

fn to_model_timestamp(value: CivilDate) -> Timestamp {
    let instant = value.to_zoned_datetime(TimeZone::UTC).map_or_else(
        |_| {
            if value.year() < 1970 {
                JiffTimestamp::MIN
            } else {
                JiffTimestamp::MAX
            }
        },
        |zoned| zoned.timestamp(),
    );
    Timestamp::from_micros_since_unix_epoch(instant.as_microsecond())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_use_utc_and_sub_microsecond_instants_round_down() {
        let value = Timestamp::parse_from_rfc3339("1970-01-01T01:59:59.999999999+02:00").unwrap();

        assert_eq!(value.__timestamp_micros_since_unix_epoch__, -1);
        let date = to_form_datetime(value);
        assert_eq!(date, CivilDate::new(1969, 12, 31).unwrap());
        assert_eq!(
            to_model_timestamp(date).__timestamp_micros_since_unix_epoch__,
            -86_400_000_000
        );
        assert_eq!(
            to_form_datetime(Timestamp::from_micros_since_unix_epoch(i64::MAX)),
            CivilDate::new(1970, 1, 1).unwrap()
        );
    }

    #[test]
    fn default_holder_leaves_required_birth_date_empty() {
        let holder = UserFormValueHolder::default();

        assert!(holder.birth_date.is_none());

        let error = holder.into_original(false).unwrap_err();
        assert_eq!(error.field_name, "birth_date");
    }
}
