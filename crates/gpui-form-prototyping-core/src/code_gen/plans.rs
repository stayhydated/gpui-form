use super::*;

/// All pre-computed token-stream fragments and identifiers for one form scaffold.
///
/// Obtained via [`FormShapeAdapter::parts`] and consumed by [`FormLayout::generate_file`].
/// Every field is `pub` so custom layouts can freely destructure and splice whichever
/// pieces they need.
pub struct FormParts {
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
    /// True when the struct has no non-skipped form fields.
    pub is_empty: bool,
    /// True when koruma validation is enabled.
    pub has_koruma: bool,
    /// True when holder conversion needs caller-provided skipped field values.
    pub has_skipped_fields: bool,
    /// Generated holder-to-model conversion API shape.
    pub holder_conversion_shape: HolderConversionShape,
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

/// Template strategy for [`FormShapeAdapter::generate_file`].
///
/// Implement this to fully control the shape of the generated file while
/// reusing all the pre-computed [`FormParts`] fragments.
pub trait FormLayout {
    fn generate_file(&self, parts: &FormParts) -> syn::File;
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
