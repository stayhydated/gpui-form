use super::{
    ComponentCreationPlan, ConditionalFragment, EventHandlerPlan, FieldInitializerPlan, FormLayout,
    FormShapeAdapter, ImportPlan, PostSubscriptionInitPlan, RenderFieldPlan, SubscriptionPlan,
    ValidationPlan,
};
use crate::error::PrototypingError;
use crate::implementations::{
    ComponentCreation, EventHandler, FieldInitializer, SubscriptionBinding,
};
use gpui_form_schema::registry::{
    ComponentFieldName, FieldComponentVariant, FieldValuePresence, FieldValueSpec, FieldVariant,
    GpuiFormShape, HolderConversionMetadata, HolderConversionShape, RustPath, RustType,
};
use quote::{ToTokens as _, quote};

const fn value_spec(
    value_type: &'static str,
    value_presence: FieldValuePresence,
) -> FieldValueSpec {
    let value_type = RustType::from_macro_tokens_unchecked(value_type);
    FieldValueSpec::new(value_type, value_type, value_presence)
}

fn compact(input: &str) -> String {
    input.chars().filter(|c| !c.is_whitespace()).collect()
}

const fn hidden_field(
    field_name: &'static str,
    value_type: &'static str,
    value_presence: FieldValuePresence,
) -> FieldVariant {
    FieldVariant::hidden(
        ComponentFieldName::new(field_name),
        value_spec(value_type, value_presence),
    )
}

const fn component_field(
    field_name: &'static str,
    value_type: &'static str,
    component: FieldComponentVariant,
) -> FieldVariant {
    FieldVariant::component(
        ComponentFieldName::new(field_name),
        value_spec(value_type, FieldValuePresence::DirectStorage),
        component,
    )
}

#[test]
fn semantic_fragments_and_plans_preserve_their_generated_tokens() {
    macro_rules! assert_fragment {
        ($ty:ty, $tokens:expr) => {{
            let empty = <$ty>::default();
            assert!(empty.is_empty());
            assert!(empty.to_token_stream().is_empty());

            let fragment = <$ty>::from($tokens);
            assert!(!fragment.is_empty());
            assert_eq!(fragment.to_token_stream().to_string(), "generated");
            assert_eq!(quote!(#fragment).to_string(), "generated");
        }};
    }

    assert_fragment!(ImportPlan, quote!(generated));
    assert_fragment!(RenderFieldPlan, quote!(generated));
    assert_fragment!(PostSubscriptionInitPlan, quote!(generated));
    assert_fragment!(ValidationPlan, quote!(generated));
    assert_fragment!(ConditionalFragment, quote!(generated));

    let creation = ComponentCreation::new(
        syn::parse_quote!(name_input),
        syn::parse_quote!(DemoFormFields),
    );
    let creations = ComponentCreationPlan::new(vec![creation.clone()]);
    assert!(!creations.is_empty());
    assert_eq!(creations.items(), &[creation]);
    assert!(creations.to_token_stream().to_string().contains("cx . new"));
    assert!(ComponentCreationPlan::default().is_empty());

    let initializer = FieldInitializer::new(syn::parse_quote!(name_input));
    let initializers = FieldInitializerPlan::new(vec![initializer.clone()]);
    assert!(!initializers.is_empty());
    assert_eq!(initializers.items(), &[initializer]);
    assert_eq!(initializers.to_token_stream().to_string(), "name_input ,");
    assert!(FieldInitializerPlan::default().is_empty());

    let binding = SubscriptionBinding::new(
        syn::parse_quote!(name_input),
        syn::parse_quote!(on_name_change),
    );
    let subscriptions = SubscriptionPlan::new(vec![binding.clone()]);
    assert!(!subscriptions.is_empty());
    assert_eq!(subscriptions.items(), &[binding]);
    assert!(
        subscriptions
            .to_token_stream()
            .to_string()
            .contains("let mut _subscriptions")
    );
    assert!(SubscriptionPlan::default().to_token_stream().is_empty());

    let handler = EventHandler::new(
        syn::parse_quote!(on_name_change),
        quote!(
            fn on_name_change() {}
        ),
    );
    let handlers = EventHandlerPlan::new(vec![handler]);
    assert!(!handlers.is_empty());
    assert_eq!(handlers.items().len(), 1);
    assert!(
        handlers
            .to_token_stream()
            .to_string()
            .contains("on_name_change")
    );
    assert!(EventHandlerPlan::default().is_empty());
}

#[test]
fn adapter_hooks_and_layout_generation_work_for_empty_forms() {
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "EmptyDemo",
        &[],
        RustPath::from_macro_tokens_unchecked("some_lib::empty_demo"),
        false,
    );

    struct MinimalLayout;

    impl FormLayout for MinimalLayout {
        fn generate_file(&self, parts: &super::FormParts) -> syn::File {
            let form_ident = &parts.form_ident;
            syn::parse2(quote!(pub struct #form_ident;)).unwrap()
        }
    }

    let adapter = FormShapeAdapter::new(&SHAPE)
        .remap_paths(|path| {
            (path == &syn::parse_quote!(some_lib::empty_demo))
                .then(|| syn::parse_quote!(consumer::empty_demo))
        })
        .render_validation_messages_with(|value| quote!(format!("{}", #value)))
        .render_children_with(|_, _, _| Some(quote!(custom_child)));

    let options = adapter.field_options();
    assert!(options.path_remapper.is_some());
    assert!(options.validation_message_renderer.is_some());
    assert!(options.render_child_renderer.is_some());
    assert_eq!(
        adapter
            .remap_path(&syn::parse_quote!(some_lib::empty_demo))
            .to_token_stream()
            .to_string(),
        "consumer :: empty_demo"
    );
    assert_eq!(
        adapter
            .remap_path(&syn::parse_quote!(other::path))
            .to_token_stream()
            .to_string(),
        "other :: path"
    );

    let imports = adapter.required_imports().unwrap().to_token_stream();
    assert!(imports.to_string().contains("InteractiveElement"));
    assert!(!imports.to_string().contains("Subscription"));

    let parts = adapter.parts().unwrap();
    assert!(parts.is_empty);
    assert!(parts.component_creations.is_empty());
    assert!(parts.field_initializers.is_empty());
    assert!(parts.subscription_calls.is_empty());
    assert!(parts.event_handlers.is_empty());
    assert!(parts.current_data_field.is_empty());
    assert!(parts.replace_current_data_fn.is_empty());
    assert_eq!(
        parts.source_module_path.to_token_stream().to_string(),
        "consumer :: empty_demo"
    );

    let generated = adapter.generate_file(&MinimalLayout).unwrap();
    assert!(prettyplease::unparse(&generated).contains("pub struct EmptyDemoForm;"));
}

#[test]
fn parts_return_error_for_invalid_source_module_path() {
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &[],
        RustPath::from_macro_tokens_unchecked("demo::"),
        false,
    );

    let error = match FormShapeAdapter::new(&SHAPE).parts() {
        Ok(_) => panic!("invalid source paths should return an error"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        PrototypingError::InvalidPath {
            kind: "source module path",
            value: "demo::".to_string(),
            error: "unexpected end of input, expected identifier".to_string(),
        }
    );
}

#[test]
fn required_imports_resolves_shape_metadata_once() {
    const FIELDS: [FieldVariant; 1] = [component_field(
        "country",
        "CountryCode",
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked("crate::"))
            .with_render_component(true)
            .with_value_binding(true),
    )];
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    );

    let error = match FormShapeAdapter::new(&SHAPE).required_imports() {
        Ok(_) => panic!("required imports should resolve metadata before generating imports"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        PrototypingError::InvalidFieldPath {
            field_name: "country".to_string(),
            kind: "component shape path",
            value: "crate::".to_string(),
            error: "unexpected end of input, expected identifier".to_string(),
        }
    );
}

#[test]
fn parts_return_error_for_invalid_field_type_metadata() {
    const FIELDS: [FieldVariant; 1] = [hidden_field(
        "country",
        "Vec<",
        FieldValuePresence::RequiresValue,
    )];
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    );

    let error = match FormShapeAdapter::new(&SHAPE).parts() {
        Ok(_) => panic!("invalid field types should return an error"),
        Err(error) => error,
    };

    assert!(
        matches!(
            error,
            PrototypingError::InvalidType {
                ref field_name,
                ref value,
                ..
            } if field_name == "country" && value == "Vec<"
        ),
        "unexpected error: {error:?}"
    );
}

#[test]
fn parts_return_error_for_invalid_component_shape_path_with_field_context() {
    const FIELDS: [FieldVariant; 1] = [component_field(
        "country",
        "CountryCode",
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked("crate::"))
            .with_render_component(true)
            .with_value_binding(true),
    )];
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    );

    let error = match FormShapeAdapter::new(&SHAPE).parts() {
        Ok(_) => panic!("invalid component shape paths should return an error"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        PrototypingError::InvalidFieldPath {
            field_name: "country".to_string(),
            kind: "component shape path",
            value: "crate::".to_string(),
            error: "unexpected end of input, expected identifier".to_string(),
        }
    );
}

#[test]
fn parts_return_error_for_missing_component_render_metadata() {
    const FIELDS: [FieldVariant; 1] = [component_field(
        "country",
        "CountryCode",
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
            "crate::shapes::CountryShape",
        ))
        .with_value_binding(true),
    )];
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    );

    let error = match FormShapeAdapter::new(&SHAPE).parts() {
        Ok(_) => panic!("missing render metadata should return an error"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        PrototypingError::MissingComponentCapability {
            struct_name: "Demo".to_string(),
            field_name: "country".to_string(),
            capability: "render metadata",
        }
    );
}

#[test]
fn parts_return_error_for_missing_component_value_binding_metadata() {
    const FIELDS: [FieldVariant; 1] = [component_field(
        "country",
        "CountryCode",
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
            "crate::shapes::CountryShape",
        ))
        .with_render_component(true),
    )];
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    );

    let error = match FormShapeAdapter::new(&SHAPE).parts() {
        Ok(_) => panic!("missing value binding metadata should return an error"),
        Err(error) => error,
    };

    assert_eq!(
        error,
        PrototypingError::MissingComponentCapability {
            struct_name: "Demo".to_string(),
            field_name: "country".to_string(),
            capability: "value binding metadata",
        }
    );
}

#[test]
fn parts_remap_source_module_path_before_rendering_imports() {
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &[],
        RustPath::from_macro_tokens_unchecked("crate::requests::system"),
        false,
    );

    let parts = FormShapeAdapter::new(&SHAPE)
        .remap_paths(|path| {
            (compact(&quote::quote! { #path }.to_string()) == "crate::requests::system")
                .then(|| syn::parse_quote!(tm_zmq_client::requests::system))
        })
        .parts()
        .expect("valid shape metadata should produce form parts");
    let source_module_path = &parts.source_module_path;

    assert_eq!(
        compact(&quote::quote! { #source_module_path }.to_string()),
        "tm_zmq_client::requests::system"
    );
    assert!(
        compact(&parts.imports.to_string()).contains("usetm_zmq_client::requests::system::*;"),
        "remapped source path should be used in generated imports: {}",
        parts.imports
    );
}

#[test]
fn required_imports_only_include_subscription_when_needed() {
    const FIELDS: [FieldVariant; 1] = [hidden_field(
        "enabled",
        "bool",
        FieldValuePresence::RequiresValue,
    )];
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    );

    let parts = FormShapeAdapter::new(&SHAPE)
        .parts()
        .expect("valid checkbox shapes should generate parts");
    let compact = compact(&parts.imports.to_string());

    assert!(
        !compact.contains("usegpui_kit::Subscription;"),
        "subscription import should be omitted when no generated subscriptions exist: {compact}"
    );
}

#[test]
fn parts_expose_component_creation_and_initializer_semantics() {
    const FIELDS: [FieldVariant; 1] = [component_field(
        "country",
        "CountryCode",
        FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
            "crate::shapes::CountryShape",
        ))
        .with_render_component(true)
        .with_value_binding(true),
    )];
    const SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    );

    let parts = FormShapeAdapter::new(&SHAPE)
        .parts()
        .expect("valid component metadata should generate parts");

    let [creation] = parts.component_creations.items() else {
        panic!("expected one component creation");
    };
    assert_eq!(creation.entity_ident().to_string(), "country");
    assert_eq!(
        creation.components_struct_ident().to_string(),
        "DemoFormComponents"
    );

    let [initializer] = parts.field_initializers.items() else {
        panic!("expected one field initializer");
    };
    assert_eq!(initializer.field_ident().to_string(), "country");

    let [binding] = parts.subscription_calls.items() else {
        panic!("expected one subscription binding");
    };
    assert_eq!(binding.entity_ident().to_string(), "country");
    assert_eq!(
        binding.handler_ident().to_string(),
        "on_country_shape_event"
    );

    let [handler] = parts.event_handlers.items() else {
        panic!("expected one event handler");
    };
    assert_eq!(
        handler.handler_ident().to_string(),
        "on_country_shape_event"
    );
}

#[test]
fn parts_use_inventory_conversion_fallibility_metadata() {
    const REQUIRED_FIELDS: [FieldVariant; 1] = [hidden_field(
        "name",
        "String",
        FieldValuePresence::DirectStorage,
    )];
    const INFALLIBLE_SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &REQUIRED_FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    );

    let parts = FormShapeAdapter::new(&INFALLIBLE_SHAPE)
        .parts()
        .expect("valid infallible shape metadata should generate parts");
    assert_eq!(
        parts.holder_conversion_shape,
        HolderConversionShape::Infallible
    );

    const OPTIONAL_FIELDS: [FieldVariant; 1] =
        [hidden_field("name", "String", FieldValuePresence::Optional)];
    const FALLIBLE_SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &OPTIONAL_FIELDS,
        RustPath::from_macro_tokens_unchecked("some_lib::demo"),
        false,
    )
    .with_holder_conversion(HolderConversionMetadata::new(
        HolderConversionShape::FallibleRequired,
        true,
    ));

    let parts = FormShapeAdapter::new(&FALLIBLE_SHAPE)
        .parts()
        .expect("valid fallible shape metadata should generate parts");
    assert_eq!(
        parts.holder_conversion_shape,
        HolderConversionShape::FallibleRequired
    );
}
