pub mod shape;

use gpui_form_schema::{registry::GpuiFormShape, resolved::ResolvedField};
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::error::{PrototypingError, PrototypingResult};
use crate::imports::ImportItem;

static SHAPE_GENERATOR: shape::ShapeCodeGenerator = shape::ShapeCodeGenerator;

pub fn field_generator() -> &'static dyn FieldCodeGenerator {
    &SHAPE_GENERATOR
}

#[derive(Default)]
pub struct GeneratedSubscription {
    pub bindings: Vec<SubscriptionBinding>,
    pub handlers: Vec<EventHandler>,
}

impl GeneratedSubscription {
    pub fn is_empty(&self) -> bool {
        self.bindings.is_empty() && self.handlers.is_empty()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComponentCreation {
    entity_ident: syn::Ident,
    components_struct_ident: syn::Ident,
}

impl ComponentCreation {
    pub fn new(entity_ident: syn::Ident, components_struct_ident: syn::Ident) -> Self {
        Self {
            entity_ident,
            components_struct_ident,
        }
    }

    pub fn entity_ident(&self) -> &syn::Ident {
        &self.entity_ident
    }

    pub fn components_struct_ident(&self) -> &syn::Ident {
        &self.components_struct_ident
    }
}

impl quote::ToTokens for ComponentCreation {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entity_ident = &self.entity_ident;
        let components_struct_ident = &self.components_struct_ident;
        tokens.extend(quote! {
            let #entity_ident =
                cx.new(|cx| #components_struct_ident::#entity_ident(window, cx));
        });
    }
}

impl std::fmt::Display for ComponentCreation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", quote::ToTokens::to_token_stream(self))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FieldInitializer {
    field_ident: syn::Ident,
}

impl FieldInitializer {
    pub fn new(field_ident: syn::Ident) -> Self {
        Self { field_ident }
    }

    pub fn field_ident(&self) -> &syn::Ident {
        &self.field_ident
    }
}

impl quote::ToTokens for FieldInitializer {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let field_ident = &self.field_ident;
        tokens.extend(quote! { #field_ident, });
    }
}

impl std::fmt::Display for FieldInitializer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", quote::ToTokens::to_token_stream(self))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SubscriptionBinding {
    entity_ident: syn::Ident,
    handler_ident: syn::Ident,
}

impl SubscriptionBinding {
    pub fn new(entity_ident: syn::Ident, handler_ident: syn::Ident) -> Self {
        Self {
            entity_ident,
            handler_ident,
        }
    }

    pub fn entity_ident(&self) -> &syn::Ident {
        &self.entity_ident
    }

    pub fn handler_ident(&self) -> &syn::Ident {
        &self.handler_ident
    }
}

impl quote::ToTokens for SubscriptionBinding {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let entity_ident = &self.entity_ident;
        let handler_ident = &self.handler_ident;
        tokens.extend(quote! {
            cx.subscribe_in(&#entity_ident, window, Self::#handler_ident)
        });
    }
}

impl std::fmt::Display for SubscriptionBinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", quote::ToTokens::to_token_stream(self))
    }
}

#[derive(Clone, Debug)]
pub struct EventHandler {
    handler_ident: syn::Ident,
    tokens: TokenStream,
}

impl EventHandler {
    pub fn new(handler_ident: syn::Ident, tokens: TokenStream) -> Self {
        Self {
            handler_ident,
            tokens,
        }
    }

    pub fn handler_ident(&self) -> &syn::Ident {
        &self.handler_ident
    }

    pub fn to_token_stream(&self) -> TokenStream {
        self.tokens.clone()
    }
}

impl quote::ToTokens for EventHandler {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.tokens.clone());
    }
}

impl std::fmt::Display for EventHandler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tokens)
    }
}

pub trait FieldCodeGenerator {
    fn generate_imports(&self, _field: &ResolvedField<'_>) -> Vec<ImportItem> {
        vec![]
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<ComponentCreation>;

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<FieldInitializer>;

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<TokenStream>;

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> PrototypingResult<GeneratedSubscription>;

    fn generate_post_subscription_initialization(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> PrototypingResult<TokenStream> {
        Ok(TokenStream::new())
    }
}

pub fn missing_component_capability(
    component: &GpuiFormShape,
    field: &ResolvedField<'_>,
    capability: &'static str,
) -> PrototypingError {
    PrototypingError::MissingComponentCapability {
        struct_name: component.struct_name.to_string(),
        field_name: field.field_name().to_string(),
        capability,
    }
}

pub trait ShapeIdentities {
    fn struct_name(&self) -> &'static str;

    fn struct_name_ident(&self) -> syn::Ident {
        format_ident!("{}", self.struct_name())
    }

    fn struct_form_ident(&self) -> syn::Ident {
        format_ident!("{}Form", self.struct_name())
    }

    fn struct_form_components_ident(&self) -> syn::Ident {
        format_ident!("{}FormComponents", self.struct_name())
    }

    fn struct_form_fields_ident(&self) -> syn::Ident {
        format_ident!("{}FormFields", self.struct_name())
    }

    fn form_id_literal(&self) -> String {
        format!("{}-form", self.struct_name().to_snake_case())
    }

    fn ftl_label_ident(&self) -> syn::Ident {
        format_ident!("{}LabelVariants", self.struct_name())
    }

    fn ftl_description_ident(&self) -> syn::Ident {
        format_ident!("{}DescriptionVariants", self.struct_name())
    }
}

impl ShapeIdentities for GpuiFormShape {
    fn struct_name(&self) -> &'static str {
        self.struct_name
    }
}

pub fn generate_entity_creation(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> ComponentCreation {
    let form_components_struct_ident = component.struct_form_components_ident();
    let component_ident = field.field_ident().clone();

    ComponentCreation::new(component_ident, form_components_struct_ident)
}

pub fn generate_entity_field_initializer(field: &ResolvedField<'_>) -> FieldInitializer {
    let field_var_name_ident = field.field_ident();
    FieldInitializer::new(field_var_name_ident.clone())
}

pub fn render_standard_field(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
    child_tokens: TokenStream,
) -> TokenStream {
    let description_fn_tokens = generate_description_fn_tokens(field, component);
    let label_tokens = generate_label_tokens(field, component);

    quote! {
        .child(
            field()
                .label(#label_tokens)
                #description_fn_tokens
                .child(#child_tokens)
        )
    }
}

pub fn generate_label_tokens(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> proc_macro2::TokenStream {
    #[cfg(not(feature = "fluent"))]
    let _ = component;

    #[cfg(feature = "fluent")]
    {
        let ftl_label_ident = component.ftl_label_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        quote! {{
            let message = #ftl_label_ident::#field_name_pascal_case_ident;
            gpui_es_fluent::localize_message(cx, &message)
        }}
    }
    #[cfg(not(feature = "fluent"))]
    {
        use heck::ToTitleCase;
        let title = field.field_name().to_title_case();
        quote! { #title }
    }
}

pub fn generate_description_fn_tokens(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> proc_macro2::TokenStream {
    let field_name_ident = field.field_ident();

    #[cfg(feature = "fluent")]
    let description_tokens = {
        let ftl_description_ident = component.ftl_description_ident();
        let field_name_pascal_case_ident = field.field_ident_pascal();
        quote! {{
            let message = #ftl_description_ident::#field_name_pascal_case_ident;
            gpui_es_fluent::localize_message(cx, &message)
        }}
    };
    #[cfg(not(feature = "fluent"))]
    let description_tokens = {
        use heck::ToTitleCase;
        let title = field.field_name().to_title_case();
        quote! { #title }
    };

    let field_has_validations = !field.validation_rules().is_empty() && component.has_koruma();
    let uses_optional_inner_validation_errors = field.uses_optional_inner_validation_errors();
    let error_tokens = if field_has_validations {
        #[cfg(feature = "fluent")]
        let conversion_tokens = quote! {
            gpui_es_fluent::localize_message(cx, &v)
        };
        #[cfg(not(feature = "fluent"))]
        let conversion_tokens = quote! { v.as_str().to_string() };

        if uses_optional_inner_validation_errors {
            quote! {
                validation_errors
                    .as_ref()
                    .and_then(|e| e.#field_name_ident())
                    .and_then(|inner_error| {
                        let errors = inner_error
                            .all()
                            .map(|v| #conversion_tokens)
                            .collect::<Vec<_>>();
                        if errors.is_empty() {
                            None
                        } else {
                            Some(errors.join("\n"))
                        }
                    })
            }
        } else {
            quote! {{
                validation_errors.as_ref().and_then(|e| {
                    let errors = e
                        .#field_name_ident()
                        .all()
                        .map(|v| #conversion_tokens)
                        .collect::<Vec<_>>();
                    if errors.is_empty() {
                        None
                    } else {
                        Some(errors.join("\n"))
                    }
                })
            }}
        }
    } else {
        quote! {{ None }}
    };
    let error_color_tokens = quote! { cx.theme().danger };

    if !field_has_validations {
        quote! {
            .description_fn({
                let description = #description_tokens;
                move |_, _| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().child(description.clone()))
                }
            })
        }
    } else {
        quote! {
            .description_fn({
                let description = #description_tokens;
                let error = #error_tokens;
                let error_color = #error_color_tokens;
                move |_, _| {
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().child(description.clone()))
                        .when(error.is_some(), |this| {
                            this.child(
                                div()
                                    .text_color(error_color)
                                    .child(error.clone().unwrap_or_default()),
                            )
                        })
                }
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ResolvedField, generate_description_fn_tokens};
    use gpui_form_schema::registry::{
        FieldValuePresence, FieldValueSpec, FieldVariant, GpuiFormShape, RustPath, RustType,
        ValidationRuleId,
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[cfg(feature = "fluent")]
    fn validation_message_conversion() -> &'static str {
        "gpui_es_fluent::localize_message(cx,&v)"
    }

    #[cfg(not(feature = "fluent"))]
    fn validation_message_conversion() -> &'static str {
        "v.as_str().to_string()"
    }

    const fn hidden_field_with_validations(
        field_name: &'static str,
        value_type: &'static str,
        value_presence: FieldValuePresence,
        validations: &'static [ValidationRuleId],
    ) -> FieldVariant {
        let value_type = RustType::from_macro_tokens_unchecked(value_type);
        FieldVariant::hidden(
            field_name,
            FieldValueSpec::new(value_type, value_type, value_presence)
                .with_validations(validations),
        )
    }

    #[test]
    fn description_uses_direct_all_for_non_optional_newtype_errors() {
        const VALIDATIONS: &[ValidationRuleId] = &[ValidationRuleId::Newtype];
        const FIELDS: [FieldVariant; 1] = [hidden_field_with_validations(
            "index",
            "Age",
            FieldValuePresence::RequiresValue,
            VALIDATIONS,
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            true,
        );

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains(&format!(
                "leterrors=e.index().all().map(|v|{}).collect::<Vec<_>>();",
                validation_message_conversion(),
            )),
            "non-optional newtype errors should keep direct .all() access: {compact}"
        );
        assert!(
            !compact.contains("e.index().and_then(|inner_error|"),
            "non-optional newtype errors should not be treated as optional: {compact}"
        );
    }

    #[test]
    fn description_unwraps_optional_newtype_inner_errors_before_all() {
        const VALIDATIONS: &[ValidationRuleId] = &[ValidationRuleId::Newtype];
        const FIELDS: [FieldVariant; 1] = [hidden_field_with_validations(
            "age",
            "Age",
            FieldValuePresence::Optional,
            VALIDATIONS,
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            true,
        );

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains(&format!(
                "validation_errors.as_ref().and_then(|e|e.age()).and_then(|inner_error|{{leterrors=inner_error.all().map(|v|{}).collect::<Vec<_>>();",
                validation_message_conversion(),
            )),
            "optional newtype errors should map/filter the inner errors before rendering: {compact}"
        );
    }

    #[test]
    fn description_unwraps_optional_nested_inner_errors_before_all() {
        const VALIDATIONS: &[ValidationRuleId] = &[ValidationRuleId::Nested];
        const FIELDS: [FieldVariant; 1] = [hidden_field_with_validations(
            "address",
            "Address",
            FieldValuePresence::Optional,
            VALIDATIONS,
        )];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            true,
        );

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains(&format!(
                "validation_errors.as_ref().and_then(|e|e.address()).and_then(|inner_error|{{leterrors=inner_error.all().map(|v|{}).collect::<Vec<_>>();",
                validation_message_conversion(),
            )),
            "optional nested errors should map/filter the inner errors before rendering: {compact}"
        );
    }
}
