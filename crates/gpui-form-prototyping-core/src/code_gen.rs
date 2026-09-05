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

mod plans;

pub use plans::*;

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
    ImportItem::aliased("gpui_kit::InteractiveElement", Alias::Anonymous),
    ImportItem::aliased("gpui_kit::ParentElement", Alias::Anonymous),
    ImportItem::aliased("gpui_kit::Styled", Alias::Anonymous),
];

const FIELD_FRAGMENT_IMPORTS: &[ImportItem] = &[
    ImportItem::path("gpui_kit::div"),
    ImportItem::path("gpui_kit::component::form::field"),
];

const VALIDATION_FRAGMENT_IMPORTS: &[ImportItem] = &[
    ImportItem::aliased("gpui_kit::prelude::FluentBuilder", Alias::Anonymous),
    ImportItem::aliased("gpui_kit::component::ActiveTheme", Alias::Anonymous),
];

const SUBSCRIPTION_IMPORTS: &[ImportItem] = &[ImportItem::path("gpui_kit::Subscription")];

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

#[cfg(test)]
mod tests;
