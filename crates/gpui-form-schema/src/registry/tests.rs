use super::{
    ComponentCapabilities, ComponentFieldName, ComponentSuffix, ConversionMetadata,
    FieldComponentVariant, FieldValuePresence, FieldValueSpec, FieldVariant, GpuiFormShape,
    HolderConversionMetadata, HolderConversionShape, RenderCapability, RustExpr, RustPath,
    RustSyntaxKind, RustType, StorageCapability, ValidationRuleId, ValueBindingCapability,
    is_valid_component_suffix, validate_component_suffix,
};

const fn value_spec(
    value_type: &'static str,
    value_presence: FieldValuePresence,
) -> FieldValueSpec {
    let value_type = RustType::from_macro_tokens_unchecked(value_type);
    FieldValueSpec::new(value_type, value_type, value_presence)
}

const fn shape_field(
    field_name: &'static str,
    value_type: &'static str,
    shape_path: &'static str,
) -> FieldVariant {
    FieldVariant::component(
        ComponentFieldName::new(field_name),
        value_spec(value_type, FieldValuePresence::RequiresValue),
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(shape_path)),
    )
}

#[test]
fn rust_metadata_constructors_validate_syntax() {
    assert_eq!(
        RustType::new("Vec<String>").unwrap().as_str(),
        "Vec<String>"
    );
    assert_eq!(
        RustPath::new("crate::fields::EmailInputShape")
            .unwrap()
            .as_str(),
        "crate::fields::EmailInputShape"
    );
    assert_eq!(
        RustExpr::new("|value| value.to_string()").unwrap().as_str(),
        "|value| value.to_string()"
    );

    let type_error = RustType::new("Vec<").unwrap_err();
    assert_eq!(type_error.kind(), RustSyntaxKind::Type);
    assert_eq!(type_error.value(), "Vec<");

    let path_error = RustPath::new("crate::").unwrap_err();
    assert_eq!(path_error.kind(), RustSyntaxKind::Path);
    assert_eq!(path_error.value(), "crate::");

    let expr_error = RustExpr::new("let").unwrap_err();
    assert_eq!(expr_error.kind(), RustSyntaxKind::Expr);
    assert_eq!(expr_error.value(), "let");
}

#[test]
fn shape_name_without_prototyping_suffix_falls_back_to_shape() {
    let field = shape_field("country", "Country", "crate::fields::CountrySelectShape");

    assert_eq!(field.field_name_with_component_suffix(), "country_shape");
    assert_eq!(field.kebab_id(), "country-shape");
}

#[test]
fn prototyping_suffix_drives_component_suffix() {
    let field = FieldVariant::component(
        ComponentFieldName::new("country"),
        value_spec("Country", FieldValuePresence::RequiresValue),
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
            "crate::fields::CountrySelectorState",
        ))
        .with_prototyping_field_suffix(Some(ComponentSuffix::new("select"))),
    );

    assert_eq!(field.field_name_with_component_suffix(), "country_select");
}

#[test]
fn prototyping_suffix_removes_duplicate_field_prefix() {
    let field = FieldVariant::component(
        ComponentFieldName::new("email"),
        value_spec("String", FieldValuePresence::RequiresValue),
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
            "crate::fields::TextInputShape",
        ))
        .with_prototyping_field_suffix(Some(ComponentSuffix::new("email_input"))),
    );

    assert_eq!(field.field_name_with_component_suffix(), "email_input");
}

#[test]
fn prototyping_suffix_exact_duplicate_uses_shape_fallback() {
    let field = FieldVariant::component(
        ComponentFieldName::new("tags"),
        value_spec("Vec<String>", FieldValuePresence::RequiresValue),
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
            "crate::fields::TagsInputShape",
        ))
        .with_prototyping_field_suffix(Some(ComponentSuffix::new("tags"))),
    );

    assert_eq!(field.field_name_with_component_suffix(), "tags_shape");
}

#[test]
fn optional_presence_uses_value_holder_option() {
    let field = FieldVariant::hidden(
        ComponentFieldName::new("email"),
        value_spec("String", FieldValuePresence::Optional),
    );

    assert_eq!(field.field_name().as_str(), "email");
    assert!(field.optional());
    assert!(!field.requires_value());
    assert!(field.value_holder_uses_option());
}

#[test]
fn holder_conversion_metadata_encodes_api_shape() {
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &[],
        RustPath::from_macro_tokens_unchecked("crate::demo"),
        false,
    )
    .with_holder_conversion(HolderConversionMetadata::new(
        HolderConversionShape::NeedsSkippedFields,
        true,
    ));

    assert_eq!(
        SHAPE.holder_conversion_shape(),
        HolderConversionShape::NeedsSkippedFields
    );
    assert!(SHAPE.has_skipped_fields());
    assert!(SHAPE.holder_conversion().uses_fallible_api());
    assert!(SHAPE.holder_conversion().runtime_can_fail());
}

#[test]
fn component_metadata_is_constructed_before_field_variant() {
    let component = FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
        "crate::fields::EmailInputShape",
    ))
    .with_render_component(true)
    .with_value_binding(true)
    .with_prototyping_field_suffix(Some(ComponentSuffix::new("input")));
    let field = FieldVariant::component(
        ComponentFieldName::new("email"),
        value_spec("String", FieldValuePresence::DirectStorage),
        component,
    );

    assert_eq!(
        field.shape_path().map(RustPath::as_str),
        Some("crate::fields::EmailInputShape")
    );
    assert!(field.render_component());
    assert!(field.value_binding());
    assert!(!field.value_holder_uses_option());
}

#[test]
fn component_variant_stores_neutral_shape_use_metadata() {
    let field = FieldVariant::component(
        ComponentFieldName::new("email"),
        value_spec("String", FieldValuePresence::DirectStorage),
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
            "crate::fields::EmailInputShape",
        ))
        .with_prototyping_field_suffix(Some(ComponentSuffix::new("input"))),
    );

    let shape_use = field
        .component_variant()
        .expect("component metadata should be attached")
        .shape_use;

    assert_eq!(shape_use.field_name().as_str(), "email");
    assert_eq!(shape_use.field_type().map(RustType::as_str), Some("String"));
    assert_eq!(
        shape_use.shape_path().as_str(),
        "crate::fields::EmailInputShape"
    );
    assert_eq!(
        shape_use
            .prototyping()
            .field_suffix
            .map(ComponentSuffix::as_str),
        Some("input")
    );
}

#[test]
fn shape_path_does_not_drive_field_suffix() {
    let field = shape_field("email", "String", "crate::fields::EmailInputShape");

    assert_eq!(field.field_name_with_component_suffix(), "email_shape");
}

#[test]
fn multiword_shape_name_without_prototyping_suffix_uses_shape() {
    let field = shape_field(
        "location",
        "Country",
        "crate::shapes::InfiniteSelect<Country>",
    );

    assert_eq!(field.field_name_with_component_suffix(), "location_shape");
}

#[test]
fn exact_duplicate_shape_name_falls_back_to_shape_suffix() {
    let field = shape_field("tags", "Vec<String>", "crate::state::TagsState");

    assert_eq!(field.field_name_with_component_suffix(), "tags_shape");
}

#[test]
fn component_suffix_validation_accepts_identifier_suffixes() {
    assert!(is_valid_component_suffix("input"));
    assert!(is_valid_component_suffix("number_input"));
    assert!(is_valid_component_suffix("_internal"));
}

#[test]
fn component_suffix_validation_rejects_invalid_suffixes() {
    for value in ["", "_", "123", "field suffix", "field-suffix"] {
        assert!(
            validate_component_suffix(value).is_err(),
            "{value:?} should be rejected"
        );
    }
}

#[test]
fn field_value_presence_names_are_stable_schema_metadata() {
    assert_eq!(FieldValuePresence::Optional.as_str(), "optional");
    assert_eq!(FieldValuePresence::RequiresValue.as_str(), "requires_value");
    assert_eq!(FieldValuePresence::DirectStorage.as_str(), "direct_storage");
}

#[test]
fn form_shape_reports_validation_and_conversion_capabilities() {
    let validated_field = FieldVariant::hidden_for_field(
        "email",
        value_spec("String", FieldValuePresence::RequiresValue)
            .with_validations(&[ValidationRuleId::Required]),
    );
    let shape = GpuiFormShape::new(
        "Profile",
        Box::leak(vec![validated_field].into_boxed_slice()),
        RustPath::from_macro_tokens_unchecked("crate::profile"),
        true,
    );
    assert_eq!(shape.struct_name, "Profile");
    assert_eq!(shape.source_module_path.as_str(), "crate::profile");
    assert!(shape.has_validations());
    assert!(shape.has_koruma());
    assert!(!shape.has_skipped_fields());
    assert_eq!(
        shape.holder_conversion_shape(),
        HolderConversionShape::Infallible
    );
    assert!(!shape.holder_conversion().runtime_can_fail());

    let no_koruma = GpuiFormShape::new(
        "Profile",
        shape.fields,
        RustPath::from_macro_tokens_unchecked("crate::profile"),
        false,
    );
    assert!(!no_koruma.has_validations());
    assert!(!no_koruma.has_koruma());
}

#[test]
fn holder_conversion_shapes_distinguish_generated_apis() {
    assert!(!HolderConversionShape::Infallible.uses_fallible_api());
    assert!(HolderConversionShape::FallibleRequired.uses_fallible_api());
    assert!(!HolderConversionShape::FallibleRequired.needs_skipped_fields());
    assert!(HolderConversionShape::NeedsSkippedFields.needs_skipped_fields());

    let fallible = HolderConversionMetadata::new(HolderConversionShape::FallibleRequired, true);
    assert_eq!(fallible.shape(), HolderConversionShape::FallibleRequired);
    assert!(fallible.uses_fallible_api());
    assert!(fallible.runtime_can_fail());
}

#[test]
fn field_presence_and_validation_ids_publish_stable_contracts() {
    assert!(FieldValuePresence::Optional.optional());
    assert!(!FieldValuePresence::DirectStorage.optional());
    assert!(FieldValuePresence::RequiresValue.requires_value());
    assert!(!FieldValuePresence::DirectStorage.requires_value());
    assert!(FieldValuePresence::RequiresValue.value_holder_uses_option());
    assert!(!FieldValuePresence::DirectStorage.value_holder_uses_option());
    assert_eq!(
        FieldValuePresence::Optional.component_storage_capability(),
        StorageCapability::OptionalValue
    );
    assert_eq!(
        FieldValuePresence::RequiresValue.component_storage_capability(),
        StorageCapability::RequiredValue
    );
    assert_eq!(
        FieldValuePresence::DirectStorage.component_storage_capability(),
        StorageCapability::DirectValue
    );

    assert_eq!(ValidationRuleId::Required.as_str(), "RequiredValidation");
    assert_eq!(ValidationRuleId::Newtype.as_str(), "NewtypeValidation");
    assert_eq!(ValidationRuleId::Nested.as_str(), "NestedValidation");
    assert_eq!(ValidationRuleId::custom("Email").as_str(), "Email");
}

#[test]
fn component_capability_builders_round_trip_all_axes() {
    let capabilities = ComponentCapabilities::default()
        .with_render(RenderCapability::Component)
        .with_value_binding(ValueBindingCapability::Inherited)
        .with_storage(StorageCapability::DirectValue);
    assert_eq!(capabilities.render(), RenderCapability::Component);
    assert_eq!(
        capabilities.value_binding(),
        ValueBindingCapability::Inherited
    );
    assert_eq!(capabilities.storage(), StorageCapability::DirectValue);
    assert!(capabilities.render_component());
    assert!(capabilities.value_binding_enabled());

    let disabled =
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked("crate::TextInput"))
            .with_render_component(false)
            .with_value_binding(false);
    assert!(!disabled.render_component());
    assert!(!disabled.value_binding());
}

#[test]
fn field_value_and_component_metadata_round_trip() {
    let from = RustExpr::from_macro_tokens_unchecked("to_form");
    let into = RustExpr::from_macro_tokens_unchecked("to_model");
    let default = RustExpr::from_macro_tokens_unchecked("String::new()");
    let conversions = ConversionMetadata::new(Some(from), Some(into));
    assert_eq!(conversions.from_expr(), Some(from));
    assert_eq!(conversions.into_expr(), Some(into));
    assert_eq!(ConversionMetadata::identity().from_expr(), None);

    let value = FieldValueSpec::new(
        RustType::from_macro_tokens_unchecked("FormEmail"),
        RustType::from_macro_tokens_unchecked("Email"),
        FieldValuePresence::Optional,
    )
    .with_conversions(conversions)
    .with_default(default)
    .with_validations(&[ValidationRuleId::Required]);
    let field = FieldVariant::component_for_field(
        "email_address",
        value,
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked("crate::EmailInput"))
            .with_capabilities(
                ComponentCapabilities::new()
                    .with_render(RenderCapability::Component)
                    .with_value_binding(ValueBindingCapability::Inherited),
            )
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("input"))),
    )
    .with_label("Email")
    .with_description("Primary email")
    .with_examples(&["person@example.com"]);

    assert_eq!(field.explicit_label(), Some("Email"));
    assert_eq!(field.description(), Some("Primary email"));
    assert_eq!(field.examples(), &["person@example.com"]);
    assert_eq!(field.value_type().as_str(), "FormEmail");
    assert_eq!(field.source_value_type().as_str(), "Email");
    assert_eq!(field.value_presence(), FieldValuePresence::Optional);
    assert_eq!(field.default_expr(), Some(default));
    assert_eq!(field.from_expr(), Some(from));
    assert_eq!(field.into_expr(), Some(into));
    assert!(field.is_component());
    assert!(field.subscribable());
    assert_eq!(field.field_name_pascal(), "EmailAddress");
    assert_eq!(field.validation_rules(), &[ValidationRuleId::Required]);

    let component = field.component_variant().unwrap();
    assert_eq!(component.shape_path().as_str(), "crate::EmailInput");
    assert_eq!(component.storage, StorageCapability::OptionalValue);
    assert_eq!(
        component
            .prototyping_field_suffix()
            .map(ComponentSuffix::as_str),
        Some("input")
    );

    let hidden = FieldVariant::hidden_for_field(
        "internal",
        value_spec("String", FieldValuePresence::DirectStorage),
    );
    assert!(!hidden.is_component());
    assert_eq!(hidden.shape_path(), None);
    assert!(!hidden.render_component());
    assert!(!hidden.value_binding());
    assert_eq!(hidden.prototyping_field_suffix(), None);
    assert!(!hidden.subscribable());
}
