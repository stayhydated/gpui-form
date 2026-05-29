use gpui_form_schema::registry::GpuiFormShape;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::error::{PrototypingError, PrototypingResult};
use crate::implementations::{
    GeneratedSubscription, ResolvedField, ShapeIdentities as _, field_generator,
};
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
    cx_new_call: Option<TokenStream>,
    field_initializer: Option<TokenStream>,
    render_child: TokenStream,
    subscription: Option<GeneratedSubscription>,
    post_subscription_initialization: Option<TokenStream>,
    _resolved: ResolvedField<'a>,
}

fn parse_ident(kind: &'static str, value: &str) -> PrototypingResult<syn::Ident> {
    syn::parse_str::<syn::Ident>(value).map_err(|_| PrototypingError::InvalidIdentifier {
        kind,
        value: value.to_string(),
    })
}

pub struct FormShapeAdapter<'a> {
    pub shape_data: &'a GpuiFormShape,
}

impl<'a> FormShapeAdapter<'a> {
    pub fn new(shape_data: &'a GpuiFormShape) -> Self {
        Self { shape_data }
    }

    fn validate_shape_data(&self) -> PrototypingResult<()> {
        let data = self.shape_data;

        parse_ident("struct name", data.struct_name)?;
        parse_ident("generated form ident", &format!("{}Form", data.struct_name))?;
        parse_ident(
            "generated form fields ident",
            &format!("{}FormFields", data.struct_name),
        )?;
        parse_ident(
            "generated form value holder ident",
            &format!("{}FormValueHolder", data.struct_name),
        )?;

        data.source_module_path
            .parse()
            .map_err(|error| PrototypingError::InvalidPath {
                kind: "source module path",
                value: error.value().to_string(),
                error: error.source_error().to_string(),
            })?;

        for field in data.components {
            parse_ident("field name", field.field_name())?;
            let _ = ResolvedField::new(field)?;
        }

        Ok(())
    }

    fn collect_fields(&self) -> PrototypingResult<Vec<GeneratedField<'a>>> {
        self.shape_data
            .components
            .iter()
            .filter(|field| field.is_component())
            .map(|field| {
                parse_ident("field name", field.field_name())?;
                parse_ident("field pascal ident", &field.field_name_pascal())?;
                parse_ident(
                    "field component ident",
                    &field.field_name_with_component_suffix(),
                )?;

                let resolved = ResolvedField::new(field)?;
                let generator = field_generator();
                let imports = generator.generate_imports(field);
                let subscription = if field.subscribable() {
                    generator.generate_subscription(&resolved, self.shape_data)
                } else {
                    None
                };

                Ok(GeneratedField {
                    imports,
                    cx_new_call: generator.generate_cx_new_call(&resolved, self.shape_data),
                    field_initializer: generator
                        .generate_field_initializers(&resolved, self.shape_data),
                    render_child: generator.generate_render_child(&resolved, self.shape_data),
                    subscription,
                    post_subscription_initialization: generator
                        .generate_post_subscription_initialization(&resolved, self.shape_data),
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
            .components
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
            .components
            .iter()
            .any(|field| field.subscribable())
        {
            set.extend_items(SUBSCRIPTION_IMPORTS);
        }
        for field in self
            .shape_data
            .components
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
        self.validate_shape_data()?;
        let data = self.shape_data;
        let generated_fields = self.collect_fields()?;

        let struct_name_ident = parse_ident("struct name", data.struct_name)?;
        let form_value_holder_ident = format_ident!("{}FormValueHolder", struct_name_ident);
        let form_ident = parse_ident("generated form ident", &format!("{}Form", data.struct_name))?;
        let form_fields_ident = parse_ident(
            "generated form fields ident",
            &format!("{}FormFields", data.struct_name),
        )?;
        let form_id_literal = data.form_id_literal();
        let context_str = format!("{}Form", data.struct_name);
        let source_module_path =
            data.source_module_path
                .parse()
                .map_err(|error| PrototypingError::InvalidPath {
                    kind: "source module path",
                    value: error.value().to_string(),
                    error: error.source_error().to_string(),
                })?;
        let has_skipped_fields = data.has_skipped_fields();
        let holder_conversion_can_fail = data.holder_conversion_can_fail();

        let is_empty = data.components.is_empty();
        let has_koruma = data.has_koruma();

        let component_creations: TokenStream = generated_fields
            .iter()
            .filter_map(|field| field.cx_new_call.clone())
            .collect();
        let field_initializers: TokenStream = generated_fields
            .iter()
            .filter_map(|field| field.field_initializer.clone())
            .collect();
        let render_children: TokenStream = generated_fields
            .iter()
            .map(|field| field.render_child.clone())
            .collect();
        let subscription_call_items: Vec<TokenStream> = generated_fields
            .iter()
            .filter_map(|field| field.subscription.as_ref())
            .flat_map(|subscription| subscription.calls.iter().cloned())
            .collect();
        let event_handler_items: Vec<TokenStream> = generated_fields
            .iter()
            .filter_map(|field| field.subscription.as_ref())
            .flat_map(|subscription| subscription.handlers.iter().cloned())
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
            .filter_map(|field| field.post_subscription_initialization.clone())
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
    /// True when the struct has no component fields.
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
        FieldValuePresence, FieldVariant, GpuiFormShape, RustPath, RustType,
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn parts_return_error_for_invalid_source_module_path() {
        const SHAPE: GpuiFormShape =
            GpuiFormShape::new("Demo", &[], RustPath::new_unchecked("demo::"), false);

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
        const FIELDS: [FieldVariant; 1] = [FieldVariant::hidden(
            "country",
            RustType::new_unchecked("Vec<"),
            FieldValuePresence::RequiresValue,
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::new_unchecked("some_lib::demo"),
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
    fn required_imports_only_include_subscription_when_needed() {
        const FIELDS: [FieldVariant; 1] = [FieldVariant::hidden(
            "enabled",
            RustType::new_unchecked("bool"),
            FieldValuePresence::RequiresValue,
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::new_unchecked("some_lib::demo"),
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
        const REQUIRED_FIELDS: [FieldVariant; 1] = [FieldVariant::hidden(
            "name",
            RustType::new_unchecked("String"),
            FieldValuePresence::DirectStorage,
        )];
        const INFALLIBLE_SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &REQUIRED_FIELDS,
            RustPath::new_unchecked("some_lib::demo"),
            false,
        );

        let parts = FormShapeAdapter::new(&INFALLIBLE_SHAPE)
            .parts()
            .expect("valid infallible shape metadata should generate parts");
        assert!(!parts.holder_conversion_can_fail);

        const OPTIONAL_FIELDS: [FieldVariant; 1] = [FieldVariant::hidden(
            "name",
            RustType::new_unchecked("String"),
            FieldValuePresence::Optional,
        )];
        const FALLIBLE_SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &OPTIONAL_FIELDS,
            RustPath::new_unchecked("some_lib::demo"),
            false,
        )
        .with_holder_conversion_can_fail(true);

        let parts = FormShapeAdapter::new(&FALLIBLE_SHAPE)
            .parts()
            .expect("valid fallible shape metadata should generate parts");
        assert!(parts.holder_conversion_can_fail);
    }
}
