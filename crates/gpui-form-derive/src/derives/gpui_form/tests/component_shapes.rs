use super::support::*;

#[test]
fn test_component_shape_generates_shape_based_state_and_constructor() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(crate::ui::BioInput))]
            bio: String,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("pubbio:::gpui_kit::Entity<")
            && compact.contains(
                "<crate::ui::BioInputas::gpui_form::runtime::shape::GpuiComponentShape>::State"
            ),
        "Component shape field should use the source field name and shape state type"
    );

    assert!(
        compact.contains(
            "<crate::ui::BioInputas::gpui_form::runtime::shape::GpuiComponentShape>::new(window,cx)"
        ),
        "Component shape constructor should delegate to shape::new"
    );

    assert!(
        compact.contains(
            "FieldVariant::component_for_field(\"bio\",::gpui_form::schema::registry::FieldValueSpec::new(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"String\"),::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"String\"),if<<crate::ui::BioInputas::gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicy"
        ),
        "FieldVariant metadata should come from the declared component shape"
    );

    assert!(
        compact.contains("with_render(if<<crate::ui::BioInputas::gpui_form::runtime::shape::GpuiComponentShape>::RenderComponentas::gpui_form::runtime::shape::GpuiComponentRender<"),
        "FieldVariant should inherit render component metadata from the shape: {compact}"
    );
    assert!(
        compact.contains(
            "FieldComponentVariant::new(::gpui_form::schema::registry::RustPath::from_macro_tokens_unchecked(\"crate::ui::BioInput\"))"
        ),
        "FieldVariant should carry the component shape path: {compact}"
    );
    assert!(
        compact.contains("with_value_binding(if<crate::ui::BioInputas::gpui_form::runtime::shape::ComponentShapeMetadata>::CAPABILITIES.value_binding().enabled()"),
        "FieldVariant should inherit component value binding metadata from the shape: {compact}"
    );
    assert!(
        compact.contains("PROTOTYPING.field_suffix")
            && compact.contains("ComponentSuffix::new(\"input\")"),
        "FieldVariant should inherit shape prototyping metadata with a generated fallback: {compact}"
    );
}

#[test]
fn test_component_shape_generates_fields() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(crate::shapes::Input::<_>, default = "test@example.com"))]
            email: String,

            #[gpui_form(component(crate::shapes::Switch))]
            enabled: bool,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("pubemail:::gpui_kit::Entity<"),
        "explicit component shape syntax should generate a source-named entity field: {compact}"
    );
    assert!(
        compact.contains("Into::into(\"test@example.com\")"),
        "explicit component syntax should compose with key-value field helpers"
    );
    assert!(
        compact.contains("pubenabled:"),
        "explicit component syntax should generate a value-holder field: {compact}"
    );
    assert!(
        compact.contains(
            "if<<crate::shapes::Switchas::gpui_form::runtime::shape::GpuiFormComponentShapePolicy>::ValueStoragePolicyas::gpui_form::runtime::shape::ComponentValueStoragePolicy>::REQUIRES_VALUE{::gpui_form::schema::registry::FieldValuePresence::RequiresValue}else{::gpui_form::schema::registry::FieldValuePresence::DirectStorage}"
        ),
        "explicit component syntax should inherit required-value metadata from the shape"
    );
    assert!(
        compact.contains("PROTOTYPING.field_suffix")
            && compact.contains("ComponentSuffix::new(\"input\")"),
        "component inventory metadata should inherit shape prototyping metadata with a generated fallback"
    );
}

#[test]
fn test_component_shape_rejects_duplicate_component_expression() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(
                component(crate::shapes::Input::<_>),
                component(crate::Input)
            )]
            email: String,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("duplicate`component`option"),
        "duplicate component syntax should produce an actionable error: {compact}"
    );
}

#[test]
fn test_component_value_binding_metadata_assertion_is_generated() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(crate::Input))]
            name: String,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());
    let removed_binding_policy_helper =
        ["Assert", "Component", "Value", "Binding", "Policy"].concat();

    assert!(
        compact.contains("Shape:::gpui_form::runtime::shape::GpuiComponentShape+::gpui_form::runtime::shape::ComponentShapeMetadata")
            && compact.contains(
                "__gpui_form_assert_name_component_value_binding::<crate::Input,String"
            )
            && !compact.contains(&removed_binding_policy_helper),
        "shape value-binding metadata should emit a derive-time metadata assertion using shape metadata: {compact}"
    );
    assert!(
        compact.contains("__gpui_form_assert_name_declared_component_shape")
            && compact.contains("__gpui_form_assert_name_component_value_compatibility")
            && compact.contains("__gpui_form_assert_name_component_value_binding"),
        "component type checks should be split into field-named assertions: {compact}"
    );
}

#[test]
fn test_component_shape_infers_field_type_generic() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(crate::shapes::Input::<_>))]
            account_no: crate::types::AccountCode,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains(
            "<crate::shapes::Input<crate::types::AccountCode>as::gpui_form::runtime::shape::GpuiComponentShape>::State"
        ),
        "component shape `_` should be resolved to the field type in FormFields: {compact}"
    );
    assert!(
        compact.contains("pubaccount_no:::gpui_kit::Entity<"),
        "generated FormFields identifiers should use the source field name: {compact}"
    );
    assert!(
        compact.contains("PROTOTYPING.field_suffix")
            && compact.contains("ComponentSuffix::new(\"input\")"),
        "inventory metadata should use the resolved generated suffix: {compact}"
    );
    assert!(
        compact.contains(
            "FieldComponentVariant::new(::gpui_form::schema::registry::RustPath::from_macro_tokens_unchecked(\"crate::shapes::Input<crate::types::AccountCode>\"))"
        ),
        "component shape metadata should store the resolved shape path: {compact}"
    );
}

#[test]
fn test_component_shape_generates_multiword_component_suffix() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(crate::shapes::DatePicker))]
            birth_date: chrono::NaiveDate,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("pubbirth_date:::gpui_kit::Entity<"),
        "multiword shape names should not affect generated component entity fields: {compact}"
    );
    assert!(
        compact.contains("PROTOTYPING.field_suffix")
            && compact.contains("ComponentSuffix::new(\"date_picker\")"),
        "inventory metadata should use the resolved generated suffix: {compact}"
    );
}
