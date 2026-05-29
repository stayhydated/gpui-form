use koruma_collection::numeric::NonNegativeValidation;

#[derive(
    koruma::Koruma,
    koruma::KorumaAllFluent,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Default,
    Ord,
    derive_more::Display,
    derive_more::From,
    derive_more::Into,
    derive_more::Deref,
    derive_more::AsRef,
    derive_more::FromStr,
)]
#[display("{}", value)]
#[koruma(try_new, newtype)]
pub struct Age {
    #[koruma(NonNegativeValidation::<_>::builder())]
    pub value: i32,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "ui", derive(gpui_form::GpuiForm,))]
#[cfg_attr(
    feature = "fluent",
    derive(es_fluent::EsFluentVariants, es_fluent::EsFluentLabel,)
)]
#[cfg_attr(
    feature = "validation",
    derive(koruma::Koruma, koruma::KorumaAllFluent)
)]
#[cfg_attr(feature = "fluent", fluent_variants(keys = ["description", "label"]))]
#[cfg_attr(feature = "fluent", fluent_label(origin, variants))]
#[cfg_attr(feature = "ui", gpui_form(koruma(fluent)))]
pub struct Item {
    #[cfg_attr(
        feature = "ui",
        gpui_form(component(gpui_form_collection::input::Input::<_>))
    )]
    #[cfg_attr(feature = "validation", koruma(newtype))]
    pub index: Age,
}
