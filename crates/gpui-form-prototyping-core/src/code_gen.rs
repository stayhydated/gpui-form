use gpui_form_schema::{
    registry::{GpuiFormShape, HolderConversionShape},
    resolved::{ResolveError, ResolvedField, ResolvedGpuiFormShape},
};
use proc_macro2::TokenStream;
use quote::quote;

use crate::error::{PrototypingError, PrototypingResult};
use crate::implementations::{
    AdditionalValidationMessageRenderer, ComponentCreation, EventHandler, FieldChangeRenderer,
    FieldCodegenOptions, FieldInitializer, GeneratedSubscription, RenderChildRenderer,
    SubscriptionBinding, ValidationVisibilityRenderer, field_generator,
};
use crate::imports::{Alias, ImportItem, ImportSet};

macro_rules! semantic_fragment {
    ($name:ident) => {
        #[derive(Clone, Debug, Default, derive_more::Display, derive_more::From)]
        pub struct $name(TokenStream);

        impl $name {
            pub fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            pub fn to_token_stream(&self) -> TokenStream {
                self.0.clone()
            }
        }

        impl quote::ToTokens for $name {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                tokens.extend(self.0.clone());
            }
        }
    };
}

semantic_fragment!(ImportPlan);
semantic_fragment!(RenderFieldPlan);
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
    cx_new_call: ComponentCreation,
    field_initializer: FieldInitializer,
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
        ResolveError::InvalidFieldPath {
            field_name,
            kind,
            value,
            error,
        } => PrototypingError::InvalidFieldPath {
            field_name,
            kind,
            value,
            error,
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
    path_remapper: Option<Box<PathRemapper<'a>>>,
    validation_message_renderer: Option<Box<ValidationMessageRenderer<'a>>>,
    validation_visibility_renderer: Option<Box<ValidationVisibilityRenderer<'a>>>,
    additional_validation_message_renderer: Option<Box<AdditionalValidationMessageRenderer<'a>>>,
    field_change_renderer: Option<Box<FieldChangeRenderer<'a>>>,
    render_child_renderer: Option<Box<RenderChildRenderer<'a>>>,
}

impl<'a> FormShapeAdapter<'a> {
    pub fn new(shape_data: &'a GpuiFormShape) -> Self {
        Self {
            shape_data,
            path_remapper: None,
            validation_message_renderer: None,
            validation_visibility_renderer: None,
            additional_validation_message_renderer: None,
            field_change_renderer: None,
            render_child_renderer: None,
        }
    }

    /// Rewrite generated Rust paths before they are emitted.
    ///
    /// This is intended for generators that consume inventory from one crate
    /// but write forms into another crate, for example remapping
    /// `crate::requests::...` to `tm_zmq_client::requests::...`.
    pub fn remap_paths(mut self, remapper: impl Fn(&syn::Path) -> Option<syn::Path> + 'a) -> Self {
        self.path_remapper = Some(Box::new(remapper));
        self
    }

    /// Render one borrowed validation error value into a user-facing string.
    ///
    /// The callback receives tokens for the borrowed validator-ref value inside
    /// the generated `.map(|v| ...)` closure and must return an expression that
    /// evaluates to `String`.
    pub fn render_validation_messages_with(
        mut self,
        renderer: impl Fn(TokenStream) -> TokenStream + 'a,
    ) -> Self {
        self.validation_message_renderer = Some(Box::new(renderer));
        self
    }

    /// Control whether generated validation messages are visible for each field.
    ///
    /// The callback receives the resolved field and returns a boolean expression.
    /// This is useful for touched-field and submit-attempted presentation policies.
    pub fn show_validation_messages_when(
        mut self,
        renderer: impl for<'field> Fn(&ResolvedField<'field>) -> TokenStream + 'a,
    ) -> Self {
        self.validation_visibility_renderer = Some(Box::new(renderer));
        self
    }

    /// Render an additional field-error expression alongside Koruma errors.
    ///
    /// The callback expression must evaluate to `Option<String>` and is used for
    /// widget parsing or staged-candidate errors owned by the caller.
    pub fn render_additional_validation_messages_with(
        mut self,
        renderer: impl for<'field> Fn(&ResolvedField<'field>) -> TokenStream + 'a,
    ) -> Self {
        self.additional_validation_message_renderer = Some(Box::new(renderer));
        self
    }

    /// Add caller-owned statements after a generated field is set or cleared.
    ///
    /// The callback receives the resolved field and tokens for a cloned previous
    /// value. It is not emitted for `ValueChange::Unchanged`.
    pub fn after_field_change_with(
        mut self,
        renderer: impl for<'field> Fn(&ResolvedField<'field>, TokenStream) -> TokenStream + 'a,
    ) -> Self {
        self.field_change_renderer = Some(Box::new(renderer));
        self
    }

    /// Render component-backed form rows with caller-owned metadata.
    ///
    /// The default shape generator requires component shapes to publish render
    /// metadata. This hook lets integration generators bridge shapes whose
    /// value-binding metadata lives in one crate while their concrete render
    /// components live in another crate.
    pub fn render_children_with(
        mut self,
        renderer: impl Fn(
            &gpui_form_schema::resolved::ResolvedField<'_>,
            &GpuiFormShape,
            &FieldCodegenOptions<'_>,
        ) -> Option<TokenStream>
        + 'a,
    ) -> Self {
        self.render_child_renderer = Some(Box::new(renderer));
        self
    }

    fn field_options(&self) -> FieldCodegenOptions<'_> {
        FieldCodegenOptions {
            path_remapper: self.path_remapper.as_deref(),
            validation_message_renderer: self.validation_message_renderer.as_deref(),
            validation_visibility_renderer: self.validation_visibility_renderer.as_deref(),
            additional_validation_message_renderer: self
                .additional_validation_message_renderer
                .as_deref(),
            field_change_renderer: self.field_change_renderer.as_deref(),
            render_child_renderer: self.render_child_renderer.as_deref(),
        }
    }

    fn remap_path(&self, path: &syn::Path) -> syn::Path {
        self.path_remapper
            .as_ref()
            .and_then(|remapper| remapper(path))
            .unwrap_or_else(|| path.clone())
    }

    fn collect_fields(
        &self,
        resolved_shape: &ResolvedGpuiFormShape<'a>,
    ) -> PrototypingResult<Vec<GeneratedField<'a>>> {
        let options = self.field_options();
        resolved_shape
            .fields()
            .iter()
            .filter(|field| field.is_component())
            .map(|resolved| {
                let resolved = resolved.clone();
                let generator = field_generator();
                let imports = generator.generate_imports(&resolved);

                Ok(GeneratedField {
                    imports,
                    cx_new_call: generator.generate_cx_new_call(&resolved, self.shape_data)?,
                    field_initializer: generator
                        .generate_field_initializers(&resolved, self.shape_data)?,
                    render_child: generator
                        .generate_render_child(&resolved, self.shape_data, &options)?
                        .into(),
                    subscription: generator.generate_subscription(
                        &resolved,
                        self.shape_data,
                        &options,
                    )?,
                    post_subscription_initialization: generator
                        .generate_post_subscription_initialization(
                            &resolved,
                            self.shape_data,
                            &options,
                        )?
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
    pub fn required_imports(&self) -> PrototypingResult<ImportSet> {
        let resolved_shape =
            ResolvedGpuiFormShape::new(self.shape_data).map_err(map_resolve_error)?;
        let mut set = ImportSet::default();
        set.extend_items(FRAGMENT_IMPORTS);
        if resolved_shape
            .fields()
            .iter()
            .any(ResolvedField::is_component)
        {
            set.extend_items(FIELD_FRAGMENT_IMPORTS);
        }
        if self.shape_data.has_validations() {
            set.extend_items(VALIDATION_FRAGMENT_IMPORTS);
        }
        if resolved_shape
            .fields()
            .iter()
            .any(ResolvedField::subscribable)
        {
            set.extend_items(SUBSCRIPTION_IMPORTS);
        }
        for resolved in resolved_shape
            .fields()
            .iter()
            .filter(|field| field.is_component())
        {
            let generator = field_generator();
            set.extend(generator.generate_imports(resolved));
        }
        Ok(set)
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
        let source_module_path = self.remap_path(resolved_shape.source_module_path());
        let holder_conversion_shape = data.holder_conversion_shape();
        let has_skipped_fields = holder_conversion_shape.needs_skipped_fields();

        let is_empty = data.fields.is_empty();
        let has_koruma = data.has_koruma();

        let component_creations = ComponentCreationPlan::new(
            generated_fields
                .iter()
                .map(|field| field.cx_new_call.clone())
                .collect(),
        );
        let field_initializers = FieldInitializerPlan::new(
            generated_fields
                .iter()
                .map(|field| field.field_initializer.clone())
                .collect(),
        );
        let field_initializer_tokens: TokenStream = field_initializers
            .items()
            .iter()
            .map(quote::ToTokens::to_token_stream)
            .collect();
        let render_children: TokenStream = generated_fields
            .iter()
            .map(|field| field.render_child.to_token_stream())
            .collect();
        let subscription_calls = SubscriptionPlan::new(
            generated_fields
                .iter()
                .flat_map(|field| field.subscription.bindings.iter().cloned())
                .collect(),
        );
        let event_handlers = EventHandlerPlan::new(
            generated_fields
                .iter()
                .flat_map(|field| field.subscription.handlers.iter().cloned())
                .collect(),
        );
        let subscription_call_items: Vec<TokenStream> = subscription_calls
            .items()
            .iter()
            .map(quote::ToTokens::to_token_stream)
            .collect();
        let subscription_call_tokens = if subscription_call_items.is_empty() {
            quote! {}
        } else {
            quote! {
                let mut _subscriptions = vec![#(#subscription_call_items),*];
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

        let (subscriptions_field, subscriptions_init) = if subscription_call_tokens.is_empty() {
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
                            "into_original: incomplete; present_fields: {:?}",
                            self.current_data.present_fields()
                        ))
                    }
                } else if matches!(
                    holder_conversion_shape,
                    HolderConversionShape::FallibleRequired
                ) {
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
                            #field_initializer_tokens
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
            holder_conversion_shape,
            imports: imports.into(),
            component_creations,
            field_initializers,
            render_children: render_children.into(),
            event_handlers,
            subscription_calls,
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

type PathRemapper<'a> = dyn Fn(&syn::Path) -> Option<syn::Path> + 'a;
type ValidationMessageRenderer<'a> = dyn Fn(TokenStream) -> TokenStream + 'a;

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
    /// True when holder conversion needs caller-provided skipped field values.
    pub has_skipped_fields: bool,
    /// Generated holder-to-model conversion API shape.
    pub holder_conversion_shape: HolderConversionShape,

    // ── Semantic generated fragments ──────────────────────────────────────────
    /// Grouped `use` statements (source module glob + framework base + per-component items).
    pub imports: ImportPlan,
    /// `cx.new(|cx| FormComponents::field(window, cx))` calls.
    pub component_creations: ComponentCreationPlan,
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
    use super::{
        ComponentCreationPlan, ConditionalFragment, EventHandlerPlan, FieldInitializerPlan,
        FormLayout, FormShapeAdapter, ImportPlan, PostSubscriptionInitPlan, RenderFieldPlan,
        SubscriptionPlan, ValidationPlan,
    };
    use crate::error::PrototypingError;
    use crate::implementations::{
        ComponentCreation, EventHandler, FieldInitializer, SubscriptionBinding,
    };
    use gpui_form_schema::registry::{
        ComponentFieldName, FieldComponentVariant, FieldValuePresence, FieldValueSpec,
        FieldVariant, GpuiFormShape, HolderConversionMetadata, HolderConversionShape, RustPath,
        RustType,
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
            !compact.contains("usegpui::Subscription;"),
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
}
#[derive(Clone, Debug, Default)]
pub struct ComponentCreationPlan(Vec<ComponentCreation>);

impl ComponentCreationPlan {
    pub fn new(items: Vec<ComponentCreation>) -> Self {
        Self(items)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn items(&self) -> &[ComponentCreation] {
        &self.0
    }
}

impl quote::ToTokens for ComponentCreationPlan {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for item in &self.0 {
            quote::ToTokens::to_tokens(item, tokens);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct FieldInitializerPlan(Vec<FieldInitializer>);

impl FieldInitializerPlan {
    pub fn new(items: Vec<FieldInitializer>) -> Self {
        Self(items)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn items(&self) -> &[FieldInitializer] {
        &self.0
    }
}

impl quote::ToTokens for FieldInitializerPlan {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for item in &self.0 {
            quote::ToTokens::to_tokens(item, tokens);
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SubscriptionPlan(Vec<SubscriptionBinding>);

impl SubscriptionPlan {
    pub fn new(items: Vec<SubscriptionBinding>) -> Self {
        Self(items)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn items(&self) -> &[SubscriptionBinding] {
        &self.0
    }
}

impl quote::ToTokens for SubscriptionPlan {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.0.is_empty() {
            return;
        }
        let bindings = &self.0;

        tokens.extend(quote! {
            let mut _subscriptions = vec![#(#bindings),*];
        });
    }
}

#[derive(Clone, Debug, Default)]
pub struct EventHandlerPlan(Vec<EventHandler>);

impl EventHandlerPlan {
    pub fn new(items: Vec<EventHandler>) -> Self {
        Self(items)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn items(&self) -> &[EventHandler] {
        &self.0
    }
}

impl quote::ToTokens for EventHandlerPlan {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for item in &self.0 {
            quote::ToTokens::to_tokens(item, tokens);
        }
    }
}
