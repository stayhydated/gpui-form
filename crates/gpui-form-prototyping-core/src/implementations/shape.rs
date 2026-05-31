use gpui_form_codegen::metadata::{apply_source_to_form_conversion_tokens, source_default_tokens};
use gpui_form_schema::registry::{ComponentCapabilities, GpuiFormShape, StorageCapability};
use proc_macro2::TokenStream;
use quote::quote;

use crate::error::PrototypingResult;
use crate::imports::ImportItem;

use super::{
    ComponentCreation, EventHandler, FieldCodeGenerator, FieldInitializer, GeneratedSubscription,
    ResolvedField, SubscriptionBinding, generate_entity_creation,
    generate_entity_field_initializer, missing_component_capability, render_standard_field,
};

/// Component shape support initializes generated form fields and wires
/// shape-owned value bindings when the metadata opts in.
pub struct ShapeCodeGenerator;

fn form_default_tokens(field: &ResolvedField<'_>, expr: &syn::Expr) -> TokenStream {
    let source_default = source_default_tokens(expr, proc_macro2::Span::call_site());
    apply_source_to_form_conversion_tokens(
        field.from_expr(),
        field
            .from_expr()
            .map_or_else(proc_macro2::Span::call_site, syn::spanned::Spanned::span),
        source_default,
    )
}

fn component_capabilities(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> PrototypingResult<ComponentCapabilities> {
    field
        .component_capabilities()
        .ok_or_else(|| missing_component_capability(component, field, "component metadata"))
}

fn runtime_shape_path(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> PrototypingResult<syn::Path> {
    field
        .runtime_shape_path()
        .ok_or_else(|| missing_component_capability(component, field, "shape path"))
}

fn ensure_default_storage_support(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> PrototypingResult<()> {
    if !field.value_holder_uses_option()
        && field.default_expr().is_none()
        && component_capabilities(field, component)?.storage() == StorageCapability::ShapePolicy
    {
        return Err(missing_component_capability(
            component,
            field,
            "default storage support",
        ));
    }

    Ok(())
}

impl FieldCodeGenerator for ShapeCodeGenerator {
    fn generate_imports(&self, field: &ResolvedField<'_>) -> Vec<ImportItem> {
        if field.value_binding() {
            vec![
                ImportItem::path("gpui_form::runtime::shape::ComponentEventOf"),
                ImportItem::path("gpui_form::runtime::shape::ComponentStateOf"),
                ImportItem::path("gpui_form::runtime::shape::FormValueChange"),
                ImportItem::path("gpui_form::runtime::shape::form_value_change"),
                ImportItem::path("gpui_form::runtime::shape::seed_value_binding_state"),
            ]
        } else {
            Vec::new()
        }
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<ComponentCreation> {
        Ok(generate_entity_creation(field, component))
    }

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> PrototypingResult<FieldInitializer> {
        Ok(generate_entity_field_initializer(field))
    }

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<TokenStream> {
        let field_in_struct_name_ident = field.field_ident();
        let capabilities = component_capabilities(field, component)?;

        if !capabilities.render_component() {
            return Err(missing_component_capability(
                component,
                field,
                "render metadata",
            ));
        }

        let shape_path = runtime_shape_path(field, component)?;
        let child_tokens = quote! {
            <<#shape_path as gpui_form::runtime::shape::ComponentShape>::RenderComponent
                as gpui_form::runtime::shape::ComponentRender<
                    <#shape_path as gpui_form::runtime::shape::ComponentShape>::State
                >>::new(&self.fields.#field_in_struct_name_ident)
        };

        Ok(render_standard_field(field, component, child_tokens))
    }

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<GeneratedSubscription> {
        let capabilities = component_capabilities(field, component)?;
        if !capabilities.value_binding_enabled() {
            return Err(missing_component_capability(
                component,
                field,
                "value binding metadata",
            ));
        }
        ensure_default_storage_support(field, component)?;

        let shape = runtime_shape_path(field, component)?;
        let field_type = field.value_type();
        let field_var_name_ident = field.field_ident();
        let field_name_ident = field.field_ident();
        let event_handler_fn_name_ident = field.component_event_handler_ident();
        let state_type = quote! { ComponentStateOf<#shape> };
        let event_type = quote! { ComponentEventOf<#shape, #field_type> };

        let bindings = vec![SubscriptionBinding::new(
            field_var_name_ident.clone(),
            event_handler_fn_name_ident.clone(),
        )];

        let set_tokens = if field.value_holder_uses_option() {
            quote! { self.current_data.#field_name_ident = Some(value); }
        } else {
            quote! { self.current_data.#field_name_ident = value; }
        };
        let clear_tokens = if field.value_holder_uses_option() {
            quote! { self.current_data.#field_name_ident = None; }
        } else if let Some(default_expr) = field.default_expr() {
            let default_expr = form_default_tokens(field, default_expr);
            quote! { self.current_data.#field_name_ident = #default_expr; }
        } else {
            quote! {
                self.current_data.#field_name_ident =
                    <<#shape as gpui_form::runtime::shape::ComponentShape>::ValueStoragePolicy
                        as gpui_form::runtime::shape::DefaultValueStorage<
                            #field_type
                        >>::default_storage();
            }
        };

        let handler_tokens = quote! {
            fn #event_handler_fn_name_ident(
                &mut self,
                state: &Entity<#state_type>,
                event: &#event_type,
                _window: &mut Window,
                _cx: &mut Context<Self>,
            ) {
                let form_change = {
                    let state = state.read(_cx);
                    form_value_change::<#shape, #field_type>(
                        state,
                        event,
                    )
                };

                match form_change {
                    FormValueChange::Set(value) => {
                        #set_tokens
                    }
                    FormValueChange::Clear => {
                        #clear_tokens
                    }
                    FormValueChange::Unchanged => {}
                }
            }
        };

        Ok(GeneratedSubscription {
            bindings,
            handlers: vec![EventHandler::new(
                event_handler_fn_name_ident,
                handler_tokens,
            )],
        })
    }

    fn generate_post_subscription_initialization(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<TokenStream> {
        let capabilities = component_capabilities(field, component)?;
        if !capabilities.value_binding_enabled() {
            return Err(missing_component_capability(
                component,
                field,
                "value binding metadata",
            ));
        }

        let shape = runtime_shape_path(field, component)?;
        let field_type = field.value_type();
        let field_var_name_ident = field.field_ident();
        let field_name_ident = field.field_ident();
        let value_tokens = if field.value_holder_uses_option() {
            quote! { current_data.#field_name_ident.as_ref() }
        } else {
            quote! { Some(&current_data.#field_name_ident) }
        };

        Ok(quote! {
            #field_var_name_ident.update(cx, |state, cx| {
                seed_value_binding_state::<#shape, #field_type>(
                    state,
                    #value_tokens,
                    window,
                    cx,
                );
            });
        })
    }
}

#[cfg(test)]
mod tests {
    use super::ShapeCodeGenerator;
    use crate::implementations::FieldCodeGenerator as _;
    use gpui_form_schema::registry::{
        ComponentSuffix, ConversionMetadata, FieldComponentVariant, FieldValuePresence,
        FieldValueSpec, FieldVariant, GpuiFormShape, RustExpr, RustPath, RustType,
    };
    use quote::quote;

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
        value_presence: FieldValuePresence,
        shape_path: &'static str,
    ) -> FieldVariant {
        FieldVariant::component(
            field_name,
            value_spec(value_type, value_presence),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(shape_path)),
        )
    }

    const SHAPE_FIELDS: [FieldVariant; 1] = [shape_field(
        "tags",
        "Vec<String>",
        FieldValuePresence::RequiresValue,
        "crate::shapes::TagsInputShape",
    )];
    const DEMO_SHAPE: GpuiFormShape = GpuiFormShape::new(
        "Demo",
        &SHAPE_FIELDS,
        RustPath::from_macro_tokens_unchecked("demo"),
        false,
    );

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    fn pretty_tokens(tokens: impl quote::ToTokens) -> String {
        let tokens = tokens.to_token_stream();
        if let Ok(file) = syn::parse2::<syn::File>(tokens.clone()) {
            return prettyplease::unparse(&file);
        }

        if let Ok(stmt) = syn::parse2::<syn::Stmt>(tokens.clone()) {
            if let Some(pretty) = pretty_file(quote! {
                fn __snapshot() {
                    #stmt
                }
            }) {
                return pretty;
            }
        }

        let raw = tokens.to_string();
        let trimmed = raw.trim();

        if trimmed.ends_with(',') {
            if let Some(pretty) = pretty_file(quote! {
                fn __snapshot() {
                    let _ = __Snapshot {
                        #tokens
                    };
                }
            }) {
                return pretty;
            }
        }

        if trimmed.starts_with('.') {
            if let Some(pretty) = pretty_file(quote! {
                fn __snapshot() {
                    __snapshot #tokens;
                }
            }) {
                return pretty;
            }
        }

        raw
    }

    fn pretty_file(tokens: proc_macro2::TokenStream) -> Option<String> {
        syn::parse2::<syn::File>(tokens)
            .map(|file| prettyplease::unparse(&file))
            .ok()
    }

    #[test]
    fn shape_generator_initializes_state_entity() {
        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&SHAPE_FIELDS[0]).unwrap();
        let tokens = generator
            .generate_cx_new_call(&field, &DEMO_SHAPE)
            .expect("valid shape-backed fields should initialize state entities");

        insta::assert_snapshot!(pretty_tokens(tokens));
    }

    #[test]
    fn shape_generator_initializes_form_fields_struct() {
        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&SHAPE_FIELDS[0]).unwrap();
        let tokens = generator
            .generate_field_initializers(&field, &DEMO_SHAPE)
            .expect("valid shape-backed fields should initialize form field structs");

        insta::assert_snapshot!(pretty_tokens(tokens));
    }

    #[test]
    fn shape_generator_emits_component_call_when_component_known() {
        const FIELDS_WITH_COMPONENT: [FieldVariant; 1] = [FieldVariant::component(
            "tags",
            value_spec("Vec<String>", FieldValuePresence::RequiresValue),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::shapes::TagsInputShape",
            ))
            .with_render_component(true),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS_WITH_COMPONENT,
            RustPath::from_macro_tokens_unchecked("demo"),
            false,
        );

        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS_WITH_COMPONENT[0]).unwrap();
        let tokens = generator
            .generate_render_child(&field, &SHAPE)
            .expect("render-enabled shape-backed fields should generate render children");

        #[cfg(feature = "fluent")]
        insta::assert_snapshot!(pretty_tokens(tokens));
        #[cfg(not(feature = "fluent"))]
        insta::assert_snapshot!(
            "shape_generator_emits_component_call_when_component_known_without_fluent",
            pretty_tokens(tokens)
        );
    }

    #[test]
    fn shape_generator_wires_shape_value_binding() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::component(
            "country",
            value_spec("CountryCode", FieldValuePresence::RequiresValue),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::shapes::CountryShape",
            ))
            .with_value_binding(true),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            false,
        );

        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("value-bound shape-backed fields should generate subscriptions");

        assert!(
            generated.bindings.len() == 1 && generated.handlers.len() == 1,
            "value-bound shape-backed fields should generate a subscription call and handler"
        );
        insta::assert_snapshot!(
            "shape_generator_wires_shape_value_binding_call",
            pretty_tokens(generated.bindings[0].clone())
        );
        insta::assert_snapshot!(
            "shape_generator_wires_shape_value_binding_handler",
            pretty_tokens(generated.handlers[0].clone())
        );

        let init = generator
            .generate_post_subscription_initialization(&field, &SHAPE)
            .expect("value-bound shape-backed fields should seed initial values");
        insta::assert_snapshot!(
            "shape_generator_wires_shape_value_binding_seed",
            pretty_tokens(init)
        );
    }

    #[test]
    fn shape_generator_clear_resets_direct_value_storage() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::component(
            "code",
            value_spec("String", FieldValuePresence::DirectStorage),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::shapes::OtpInput<String>",
            ))
            .with_value_binding(true),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            false,
        );

        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("value-bound direct-storage fields should generate subscriptions");

        insta::assert_snapshot!(pretty_tokens(generated.handlers[0].clone()));
    }

    #[test]
    fn shape_generator_clear_resets_direct_value_storage_to_declared_default() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::component(
            "code",
            value_spec("String", FieldValuePresence::DirectStorage).with_default(
                RustExpr::from_macro_tokens_unchecked("String::from(\"123456\")"),
            ),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::shapes::OtpInput<String>",
            ))
            .with_value_binding(true),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            false,
        );

        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("value-bound direct-storage fields should generate subscriptions");

        insta::assert_snapshot!(pretty_tokens(generated.handlers[0].clone()));
    }

    #[test]
    fn shape_generator_clear_converts_literal_declared_default() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::component(
            "code",
            value_spec("String", FieldValuePresence::DirectStorage)
                .with_default(RustExpr::from_macro_tokens_unchecked("\"123456\"")),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::shapes::OtpInput<String>",
            ))
            .with_value_binding(true),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            false,
        );

        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("value-bound direct-storage fields should generate subscriptions");

        insta::assert_snapshot!(pretty_tokens(generated.handlers[0].clone()));
    }

    #[test]
    fn shape_generator_clear_converts_source_default_to_form_value() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::component(
            "birth_date",
            FieldValueSpec::new(
                RustType::from_macro_tokens_unchecked("chrono::NaiveDate"),
                RustType::from_macro_tokens_unchecked("Timestamp"),
                FieldValuePresence::DirectStorage,
            )
            .with_conversions(ConversionMetadata::new(
                Some(RustExpr::from_macro_tokens_unchecked("to_form_datetime")),
                Some(RustExpr::from_macro_tokens_unchecked("to_model_timestamp")),
            ))
            .with_default(RustExpr::from_macro_tokens_unchecked(
                "Timestamp::from_micros_since_unix_epoch(0)",
            )),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::shapes::DatePicker",
            ))
            .with_value_binding(true),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            false,
        );

        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("value-bound converted fields should generate subscriptions");

        insta::assert_snapshot!(pretty_tokens(generated.handlers[0].clone()));
    }

    #[test]
    fn shape_generator_uses_declared_suffix_for_component_names() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::component(
            "country",
            value_spec("CountryCode", FieldValuePresence::RequiresValue),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::shapes::CountrySelectShape",
            ))
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("select")))
            .with_value_binding(true),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            false,
        );

        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let created = generator
            .generate_cx_new_call(&field, &SHAPE)
            .expect("valid shape-backed fields should initialize state entities");
        let generated = generator
            .generate_subscription(&field, &SHAPE)
            .expect("value-bound shape-backed fields should generate subscriptions");
        let compact_created = compact(&created.to_string());
        let compact_handler = compact(&generated.handlers[0].to_string());

        insta::assert_snapshot!(
            "shape_generator_uses_declared_suffix_for_component_names_created",
            pretty_tokens(created)
        );
        insta::assert_snapshot!(
            "shape_generator_uses_declared_suffix_for_component_names_handler",
            pretty_tokens(generated.handlers[0].clone())
        );
        assert!(
            compact_created.contains("letcountry=cx.new"),
            "declared prototyping suffixes should not affect generated entity fields: {compact_created}"
        );
        assert!(
            compact_handler.contains("fnon_country_select_event"),
            "declared prototyping suffixes should drive generated handler suffixes: {compact_handler}"
        );
    }

    #[test]
    fn shape_generator_renders_generic_component_type() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::component(
            "tags",
            value_spec("Vec<Tag>", FieldValuePresence::RequiresValue),
            FieldComponentVariant::new(RustPath::from_macro_tokens_unchecked(
                "crate::shapes::TagsComboboxShape",
            ))
            .with_render_component(true)
            .with_prototyping_field_suffix(Some(ComponentSuffix::new("combobox"))),
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            false,
        );

        let generator = ShapeCodeGenerator;
        let field = crate::implementations::ResolvedField::new(&FIELDS[0]).unwrap();
        let tokens = generator
            .generate_render_child(&field, &SHAPE)
            .expect("render-enabled generic shape-backed fields should generate render children");

        #[cfg(feature = "fluent")]
        insta::assert_snapshot!(pretty_tokens(tokens));
        #[cfg(not(feature = "fluent"))]
        insta::assert_snapshot!(
            "shape_generator_renders_generic_component_type_without_fluent",
            pretty_tokens(tokens)
        );
    }
}
