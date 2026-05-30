use gpui_form_schema::{
    registry::GpuiFormShape,
    resolved::{ResolveError, ResolvedField, ResolvedGpuiFormShape},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::error::{PrototypingError, PrototypingResult};
use crate::implementations::{GeneratedSubscription, field_generator};
use crate::imports::{Alias, ImportItem, ImportSet};

macro_rules! semantic_fragment {
    ($name:ident) => {
        #[derive(Clone, Debug, Default)]
        pub struct $name(TokenStream);

        impl $name {
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            pub fn to_token_stream(&self) -> TokenStream {
                self.0.clone()
            }
        }

        impl From<TokenStream> for $name {
            fn from(tokens: TokenStream) -> Self {
                Self(tokens)
            }
        }

        impl quote::ToTokens for $name {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                tokens.extend(self.0.clone());
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

semantic_fragment!(ImportPlan);
semantic_fragment!(ComponentEntityInit);
semantic_fragment!(FieldInitializerPlan);
semantic_fragment!(RenderFieldPlan);
semantic_fragment!(EventHandlerPlan);
semantic_fragment!(SubscriptionPlan);
semantic_fragment!(PostSubscriptionInitPlan);
semantic_fragment!(ValidationPlan);
semantic_fragment!(ConditionalFragment);

/// Imports required by prototyping-core's own generated fragments.
///
/// Layout-specific imports such as `Render`, `Focusable`, `v_form`, or
/// `Divider` belong in the caller's [`FormLayout`] implementation rather than
/// in the shared form-shape adapter output.
const FRAGMENT_IMPORTS: &[ImportItem] = &[
    ImportItem::path("gpui::InteractiveElement"),
    ImportItem::aliased("gpui::ParentElement", Alias::Anonymous),
    ImportItem::path("gpui::Styled"),
];

const FIELD_FRAGMENT_IMPORTS: &[ImportItem] = &[
    ImportItem::path("gpui::div"),
    ImportItem::path("gpui_component::form::field"),
];

const VALIDATION_FRAGMENT_IMPORTS: &[ImportItem] = &[
    ImportItem::aliased("gpui::prelude::FluentBuilder", Alias::Anonymous),
    ImportItem::aliased("gpui_component::ActiveTheme", Alias::Anonymous),
];

const SUBSCRIPTION_IMPORTS: &[ImportItem] = &[ImportItem::path("gpui::Subscription")];

struct GeneratedField<'a> {
    imports: Vec<ImportItem>,
    cx_new_call: ComponentEntityInit,
    field_initializer: FieldInitializerPlan,
    render_child: RenderFieldPlan,
    subscription: GeneratedSubscription,
    post_subscription_initialization: PostSubscriptionInitPlan,
    _resolved: ResolvedField<'a>,
}

fn map_resolve_error(error: ResolveError) -> PrototypingError {
    match error {
        ResolveError::InvalidIdentifier { kind, value } => {
            PrototypingError::InvalidIdentifier { kind, value }
        },
        ResolveError::InvalidPath { kind, value, error } => {
            PrototypingError::InvalidPath { kind, value, error }
        },
        ResolveError::InvalidType {
            field_name,
            value,
            error,
        } => PrototypingError::InvalidType {
            field_name,
            value,
            error,
        },
        ResolveError::InvalidExpression {
            field_name,
            value,
            error,
        } => PrototypingError::InvalidExpression {
            field_name,
            value,
            error,
        },
    }
}

pub struct FormShapeAdapter<'a> {
    pub shape_data: &'a GpuiFormShape,
}

impl<'a> FormShapeAdapter<'a> {
    pub fn new(shape_data: &'a GpuiFormShape) -> Self {
        Self { shape_data }
    }

    fn collect_fields(
        &self,
        resolved_shape: &ResolvedGpuiFormShape<'a>,
    ) -> PrototypingResult<Vec<GeneratedField<'a>>> {
        resolved_shape
            .fields()
            .iter()
            .filter(|field| field.raw().is_component())
            .map(|resolved| {
                let resolved = resolved.clone();
                let field = resolved.raw();
                let generator = field_generator();
                let imports = generator.generate_imports(field);
                let subscription = generator.generate_subscription(&resolved, self.shape_data)?;

                Ok(GeneratedField {
                    imports,
                    cx_new_call: generator
                        .generate_cx_new_call(&resolved, self.shape_data)?
                        .into(),
                    field_initializer: generator
                        .generate_field_initializers(&resolved, self.shape_data)?
                        .into(),
                    render_child: generator
                        .generate_render_child(&resolved, self.shape_data)?
                        .into(),
                    subscription,
                    post_subscription_initialization: generator
                        .generate_post_subscription_initialization(&resolved, self.shape_data)?
                        .into(),
                    _resolved: resolved,
                })
            })
            .collect()
    }

    /// Collect all imports needed by this form's generated file.
    ///
    /// Starts with imports needed by prototyping-core's own generated fragments,
    /// then asks each field's generator for its own requirements. The result can
    /// be rendered as grouped `use` statements via [`ImportSet::to_token_stream`].
    pub fn required_imports(&self) -> ImportSet {
        let mut set = ImportSet::default();
        set.extend_items(FRAGMENT_IMPORTS);
        if self
            .shape_data
            .fields
            .iter()
            .any(|field| field.is_component())
        {
            set.extend_items(FIELD_FRAGMENT_IMPORTS);
        }
        if self.shape_data.has_validations() {
            set.extend_items(VALIDATION_FRAGMENT_IMPORTS);
        }
        if self
            .shape_data
            .fields
            .iter()
            .any(|field| field.subscribable())
        {
            set.extend_items(SUBSCRIPTION_IMPORTS);
        }
        for field in self
            .shape_data
            .fields
            .iter()
            .filter(|field| field.is_component())
        {
            let generator = field_generator();
            set.extend(generator.generate_imports(field));
        }
        set
    }

    /// Compute all token-stream fragments and identifiers for this form.
    ///
    /// Prefer this when you want to assemble a fully custom `quote!{}` template.
    /// All conditional / derived fragments (e.g. `subscriptions_field`,
    /// `current_data_field`) are pre-evaluated so you only need to splice them in.
    /// Returns a [`PrototypingError`] when the input shape metadata cannot be
    /// converted into valid Rust identifiers, types, or paths.
    pub fn parts(&self) -> PrototypingResult<FormParts> {
        let resolved_shape =
            ResolvedGpuiFormShape::new(self.shape_data).map_err(map_resolve_error)?;
        let data = self.shape_data;
        let generated_fields = self.collect_fields(&resolved_shape)?;

        let struct_name_ident = resolved_shape.name().ident().clone();
        let form_value_holder_ident = resolved_shape.name().value_holder_ident().clone();
        let form_ident = resolved_shape.name().form_ident().clone();
        let form_fields_ident = resolved_shape.name().fields_ident().clone();
        let form_id_literal = resolved_shape.name().form_id().to_string();
        let context_str = format!("{}Form", resolved_shape.name().source());
        let source_module_path = resolved_shape.source_module_path().clone();
        let has_skipped_fields = data.has_skipped_fields();
        let holder_conversion_can_fail = data.holder_conversion_can_fail();

        let is_empty = data.fields.is_empty();
        let has_koruma = data.has_koruma();

        let component_creations: TokenStream = generated_fields
            .iter()
            .map(|field| field.cx_new_call.to_token_stream())
            .collect();
        let field_initializers: TokenStream = generated_fields
            .iter()
            .map(|field| field.field_initializer.to_token_stream())
            .collect();
        let render_children: TokenStream = generated_fields
            .iter()
            .map(|field| field.render_child.to_token_stream())
            .collect();
        let subscription_call_items: Vec<TokenStream> = generated_fields
            .iter()
            .flat_map(|field| field.subscription.calls.iter().cloned())
            .collect();
        let event_handler_items: Vec<TokenStream> = generated_fields
            .iter()
            .flat_map(|field| field.subscription.handlers.iter().cloned())
            .collect();
        let subscription_calls = if subscription_call_items.is_empty() {
            TokenStream::new()
        } else {
            quote! {
                let mut _subscriptions = vec![#(#subscription_call_items),*];
            }
        };
        let event_handlers = if event_handler_items.is_empty() {
            TokenStream::new()
        } else {
            quote! {
                #(#event_handler_items)*
            }
        };
        let post_subscription_init: TokenStream = generated_fields
            .iter()
            .map(|field| field.post_subscription_initialization.to_token_stream())
            .collect();

        let validation_binding = if has_koruma {
            quote! { let validation_errors = self.current_data.validate().err(); }
        } else {
            quote! {}
        };

        let (subscriptions_field, subscriptions_init) = if subscription_calls.is_empty() {
            (quote! {}, quote! {})
        } else {
            (
                quote! { _subscriptions: Vec<Subscription>, },
                quote! { _subscriptions, },
            )
        };

        let (current_data_field, current_data_let, current_data_init, fields_init, debug_child) =
            if is_empty {
                (
                    quote! {},
                    quote! {},
                    quote! {},
                    quote! { fields: #form_fields_ident, },
                    quote! {},
                )
            } else {
                let into_original_debug_child = if has_skipped_fields {
                    quote! {
                        .child(format!(
                            "into_original: incomplete; present_fields_json: {}",
                            self.current_data.present_fields_json()
                        ))
                    }
                } else if holder_conversion_can_fail {
                    quote! {
                        .child(format!(
                            "try_into_original: {:?}",
                            self.current_data.clone().try_into_original()
                        ))
                    }
                } else {
                    quote! {
                        .child(format!(
                            "into_original: {:?}",
                            self.current_data.clone().into_original()
                        ))
                    }
                };

                (
                    quote! { current_data: #form_value_holder_ident, },
                    quote! { let current_data = #form_value_holder_ident::default(); },
                    quote! { current_data, },
                    quote! {
                        fields: #form_fields_ident {
                            #field_initializers
                        },
                    },
                    quote! {
                        .child(format!("value_holder: {:?}", self.current_data))
                        #into_original_debug_child
                    },
                )
            };

        let replace_current_data_fn = if is_empty {
            quote! {}
        } else {
            quote! {
                pub fn replace_current_data(
                    &mut self,
                    current_data: #form_value_holder_ident,
                    window: &mut Window,
                    cx: &mut Context<Self>,
                ) {
                    *self = Self::new_with_current_data(current_data, window, cx);
                    cx.notify();
                }
            }
        };

        let mut collected_imports = ImportSet::default();
        collected_imports.extend_items(FRAGMENT_IMPORTS);
        if !generated_fields.is_empty() {
            collected_imports.extend_items(FIELD_FRAGMENT_IMPORTS);
        }
        if data.has_validations() {
            collected_imports.extend_items(VALIDATION_FRAGMENT_IMPORTS);
        }
        if !subscription_call_items.is_empty() {
            collected_imports.extend_items(SUBSCRIPTION_IMPORTS);
        }
        for field in &generated_fields {
            collected_imports.extend(field.imports.clone());
        }
        let collected_imports = collected_imports.to_token_stream();
        let imports = quote! {
            use #source_module_path::*;
            #collected_imports
        };

        Ok(FormParts {
            struct_name_ident,
            form_ident,
            form_fields_ident,
            form_value_holder_ident,
            source_module_path,
            context_str,
            form_id_literal,
            is_empty,
            has_koruma,
            has_skipped_fields,
            holder_conversion_can_fail,
            imports: imports.into(),
            component_creations: component_creations.into(),
            field_initializers: field_initializers.into(),
            render_children: render_children.into(),
            event_handlers: event_handlers.into(),
            subscription_calls: subscription_calls.into(),
            post_subscription_init: post_subscription_init.into(),
            validation_binding: validation_binding.into(),
            subscriptions_field: subscriptions_field.into(),
            subscriptions_init: subscriptions_init.into(),
            current_data_field: current_data_field.into(),
            current_data_let: current_data_let.into(),
            current_data_init: current_data_init.into(),
            fields_init: fields_init.into(),
            debug_child: debug_child.into(),
            replace_current_data_fn: replace_current_data_fn.into(),
        })
    }

    /// Generate a `syn::File` from a [`FormLayout`] implementation.
    ///
    /// Returns a [`PrototypingError`] when the shape metadata is malformed.
    ///
    /// ```rust,ignore
    /// struct MyLayout;
    /// impl FormLayout for MyLayout {
    ///     fn generate_file(&self, parts: &FormParts) -> syn::File {
    ///         let FormParts { imports, render_children, form_ident, .. } = parts;
    ///         syn::parse2(quote! {
    ///             #imports
    ///             pub struct #form_ident { /* ... */ }
    ///             // splice #render_children wherever you need it
    ///         }).unwrap()
    ///     }
    /// }
    /// FormShapeAdapter::new(shape).generate_file(&MyLayout)?;
    /// ```
    pub fn generate_file(&self, layout: &impl FormLayout) -> PrototypingResult<syn::File> {
        let parts = self.parts()?;
        Ok(layout.generate_file(&parts))
    }
}

// ── FormParts ─────────────────────────────────────────────────────────────────

/// All pre-computed token-stream fragments and identifiers for one form scaffold.
///
/// Obtained via [`FormShapeAdapter::parts`] and consumed by [`FormLayout::generate_file`].
/// Every field is `pub` so custom layouts can freely destructure and splice whichever
/// pieces they need.
pub struct FormParts {
    // ── Identifiers ──────────────────────────────────────────────────────────
    /// The original struct ident, e.g. `User`.
    pub struct_name_ident: syn::Ident,
    /// Generated form view ident, e.g. `UserForm`.
    pub form_ident: syn::Ident,
    /// Generated form fields ident, e.g. `UserFormFields`.
    pub form_fields_ident: syn::Ident,
    /// Generated value holder ident, e.g. `UserFormValueHolder`.
    pub form_value_holder_ident: syn::Ident,
    /// Glob import path for the source module, e.g. `some_lib::structs::user`.
    pub source_module_path: syn::Path,
    /// GPUI key context string, e.g. `"UserForm"`.
    pub context_str: String,
    /// GPUI element id, e.g. `"user-form"`.
    pub form_id_literal: String,

    // ── Flags ─────────────────────────────────────────────────────────────────
    /// True when the struct has no non-skipped form fields.
    pub is_empty: bool,
    /// True when koruma validation is enabled.
    pub has_koruma: bool,
    /// True when at least one source field was marked with `#[gpui_form(skip)]`.
    pub has_skipped_fields: bool,
    /// True when holder-to-model conversion may fail because a required
    /// shape-backed value can be missing and no field default was declared.
    pub holder_conversion_can_fail: bool,

    // ── Semantic generated fragments ──────────────────────────────────────────
    /// Grouped `use` statements (source module glob + framework base + per-component items).
    pub imports: ImportPlan,
    /// `cx.new(|cx| FormComponents::field(window, cx))` calls.
    pub component_creations: ComponentEntityInit,
    /// Field name tokens for the `FormFields { ... }` struct literal.
    pub field_initializers: FieldInitializerPlan,
    /// `.child(field().label(...).child(...))` chains for the form body.
    pub render_children: RenderFieldPlan,
    /// Event handler `fn` items to place in an `impl` block.
    pub event_handlers: EventHandlerPlan,
    /// `let mut _subscriptions = vec![...]` binding.
    pub subscription_calls: SubscriptionPlan,
    /// Post-subscription setup (e.g. populating initial field values).
    pub post_subscription_init: PostSubscriptionInitPlan,
    /// `let validation_errors = ...` binding; empty when koruma is disabled.
    pub validation_binding: ValidationPlan,

    // ── Derived conditional fragments ─────────────────────────────────────────
    /// `_subscriptions: Vec<Subscription>,` struct field; empty when no subscriptions.
    pub subscriptions_field: ConditionalFragment,
    /// `_subscriptions,` in `Self { ... }`; empty when no subscriptions.
    pub subscriptions_init: ConditionalFragment,
    /// `current_data: FormValueHolder,` struct field; empty for empty forms.
    pub current_data_field: ConditionalFragment,
    /// `let current_data = FormValueHolder::default();` binding; empty for empty forms.
    pub current_data_let: ConditionalFragment,
    /// `current_data,` in `Self { ... }`; empty for empty forms.
    pub current_data_init: ConditionalFragment,
    /// `fields: FormFields { #field_initializers }` initializer block.
    pub fields_init: ConditionalFragment,
    /// Debug rows for value-holder and into-original status; empty for empty forms.
    pub debug_child: ConditionalFragment,
    /// `replace_current_data(...)` helper method; empty for empty forms.
    pub replace_current_data_fn: ConditionalFragment,
}

// ── FormLayout ────────────────────────────────────────────────────────────────

/// Template strategy for [`FormShapeAdapter::generate_file`].
///
/// Implement this to fully control the shape of the generated file while
/// reusing all the pre-computed [`FormParts`] fragments.
pub trait FormLayout {
    fn generate_file(&self, parts: &FormParts) -> syn::File;
}

#[cfg(test)]
mod tests {
    use super::FormShapeAdapter;
    use crate::error::PrototypingError;
    use gpui_form_schema::registry::{
        FieldComponentVariant, FieldValuePresence, FieldVariant, GpuiFormShape, RustPath, RustType,
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    const fn hidden_field(
        field_name: &'static str,
        value_type: &'static str,
        value_presence: FieldValuePresence,
    ) -> FieldVariant {
        FieldVariant::builder(field_name)
            .with_value_type(RustType::from_macro_tokens_unchecked(value_type))
            .with_value_presence(value_presence)
            .hidden()
    }

    const fn component_field(
        field_name: &'static str,
        value_type: &'static str,
        component: FieldComponentVariant,
    ) -> FieldVariant {
        FieldVariant::builder(field_name)
            .with_value_type(RustType::from_macro_tokens_unchecked(value_type))
            .with_value_presence(FieldValuePresence::DirectStorage)
            .component(component)
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
            !compact.contains("usegpui::Subscription;"),
            "subscription import should be omitted when no generated subscriptions exist: {compact}"
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
        assert!(!parts.holder_conversion_can_fail);

        const OPTIONAL_FIELDS: [FieldVariant; 1] =
            [hidden_field("name", "String", FieldValuePresence::Optional)];
        const FALLIBLE_SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &OPTIONAL_FIELDS,
            RustPath::from_macro_tokens_unchecked("some_lib::demo"),
            false,
        )
        .with_holder_conversion_can_fail(true);

        let parts = FormShapeAdapter::new(&FALLIBLE_SHAPE)
            .parts()
            .expect("valid fallible shape metadata should generate parts");
        assert!(parts.holder_conversion_can_fail);
    }
}
