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
#[fluent_label(origin, variants)]
#[fluent_variants(keys = ["description", "label"])]
#[gpui_form(koruma(fluent))]
pub struct User {
    #[gpui_form(component = gpui_form_collection::input::InputShape::<_>)]
    #[koruma(
        NonEmptyValidation::<_>::builder(),
        PrefixValidation::<_>::builder().prefix("Xx"),
        SuffixValidation::<_>::builder().suffix("xX")
    )]
    pub username: String,

    #[gpui_form(
        component = gpui_form_collection::input::InputShape::<_>,
        default = "test@example.com"
    )]
    #[koruma(EmailValidation::<_>::builder())]
    pub email: String,

    #[gpui_form(component = gpui_form_collection::input::InputShape::<_>)]
    #[koruma(RangeValidation::<_>::builder().min(18).max(167))]
    pub age: Option<u32>,

    #[gpui_form(
        component = gpui_form_collection::input::InputShape::<_>,
        default = 67
    )]
    #[koruma(PositiveValidation::<_>::builder())]
    pub balance: rust_decimal::Decimal,

    #[gpui_form(component = gpui_form_collection::input::InputShape::<_>)]
    #[koruma(NegativeValidation::<_>::builder())]
    pub debt: rust_decimal::Decimal,

    #[gpui_form(component = gpui_form_collection::switch::SwitchShape)]
    pub subscribe_newsletter: bool,

    #[gpui_form(component = gpui_form_collection::checkbox::CheckboxShape)]
    pub enable_notifications: bool,

    #[gpui_form(component = gpui_form_collection::select::SelectShape::<_>)]
    pub preferred: PreferredLanguage,

    #[gpui_form(
        component = gpui_form_collection::select::SelectShape::<_>::searchable(true),
        default = EnumCountry::France
    )]
    pub country: Option<EnumCountry>,

    #[gpui_form(
        type = chrono::NaiveDate,
        from = to_form_datetime,
        into = to_model_timestamp,
        component = gpui_form_collection::input::InputShape::<_>
    )]
    pub birth_date: Option<Timestamp>,

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
