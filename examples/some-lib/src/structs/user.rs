use anyhow::Context as _;
use es_fluent::{EsFluent, EsFluentLabel, EsFluentVariants};
use gpui_form::GpuiForm;
use gpui_form_collection_derive::SelectItem;
use koruma::{Koruma, KorumaAllFluent};
use koruma_collection::{
    collection::NonEmptyValidation,
    format::EmailValidation,
    numeric::{NegativeValidation, PositiveValidation, RangeValidation},
    string::{PrefixValidation, SuffixValidation},
};
use strum::EnumIter;

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
#[select_item(fluent)]
pub enum PreferredLanguage {
    #[default]
    English,
    French,
    Chinese,
}

#[derive(Clone, Debug, Default, EnumIter, EsFluent, PartialEq, SelectItem)]
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
        PrefixValidation::<_>::prefix("Xx"),
        SuffixValidation::<_>::suffix("xX")
    )]
    pub username: String,

    #[gpui_form(component(
        gpui_form_collection::input::Input::<_>,
        default = "test@example.com"
    ))]
    #[koruma(EmailValidation::<_>)]
    pub email: String,

    #[gpui_form(component(gpui_form_collection::input::Input::<_>))]
    #[koruma(RangeValidation::<_>::min(18).max(167))]
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
    pub brand_color: Option<gpui::Hsla>,

    #[gpui_form(component(gpui_form_collection::otp_input::OtpInput::<_>))]
    pub otp_code: String,

    #[gpui_form(component(gpui_form_component::file_picker::FilePicker))]
    pub uploaded_files: Vec<std::path::PathBuf>,

    #[gpui_form(component(gpui_form_collection::date_picker::DateRangePicker))]
    pub holiday_range: Option<(chrono::NaiveDate, chrono::NaiveDate)>,

    #[gpui_form(component(gpui_form_collection::switch::Switch))]
    pub subscribe_newsletter: bool,

    #[gpui_form(component(gpui_form_collection::checkbox::Checkbox))]
    pub enable_notifications: bool,

    #[gpui_form(component(gpui_form_collection::select::Select::<_>::searchable(true)))]
    pub preferred: PreferredLanguage,

    #[gpui_form(component(
        gpui_form_collection::select::Select::<_>,
        default = EnumCountry::France
    ))]
    pub country: Option<EnumCountry>,

    #[gpui_form(component(
        gpui_form_collection::date_picker::DatePicker,
        value(
            type = chrono::NaiveDate,
            from_source = to_form_datetime,
            into_source = to_model_timestamp,
        ),
        default = Timestamp::from_micros_since_unix_epoch(0)
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
        chrono::DateTime::parse_from_rfc3339(str)
             .map_err(|err| anyhow::anyhow!(err))
             .with_context(|| "Invalid timestamp format. Expected RFC 3339 format (e.g. '2025-02-10 15:45:30').")
             .map(|dt| dt.timestamp_micros())
             .map(Timestamp::from_micros_since_unix_epoch)
    }
    pub fn from_micros_since_unix_epoch(micros: i64) -> Self {
        Self {
            __timestamp_micros_since_unix_epoch__: micros,
        }
    }
}

#[allow(dead_code)]
fn to_form_datetime(value: Timestamp) -> chrono::NaiveDate {
    chrono::DateTime::<chrono::Utc>::from_timestamp_micros(
        value.__timestamp_micros_since_unix_epoch__,
    )
    .unwrap_or_else(|| chrono::DateTime::<chrono::Utc>::from_timestamp_micros(0).unwrap())
    .date_naive()
}

fn to_model_timestamp(value: chrono::NaiveDate) -> Timestamp {
    let naive_datetime = value.and_hms_opt(0, 0, 0).unwrap();
    let datetime =
        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(naive_datetime, chrono::Utc);
    Timestamp::from_micros_since_unix_epoch(datetime.timestamp_micros())
}
