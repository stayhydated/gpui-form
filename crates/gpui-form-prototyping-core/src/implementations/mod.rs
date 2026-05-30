pub mod shape;

use gpui_form_schema::{
    registry::{FieldVariant, GpuiFormShape},
    resolved::ResolvedField,
};
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::imports::ImportItem;

static SHAPE_GENERATOR: shape::ShapeCodeGenerator = shape::ShapeCodeGenerator;

pub fn field_generator() -> &'static dyn FieldCodeGenerator {
    &SHAPE_GENERATOR
}

#[derive(Default)]
pub struct GeneratedSubscription {
    pub calls: Vec<TokenStream>,
    pub handlers: Vec<TokenStream>,
}

impl GeneratedSubscription {
    pub fn is_empty(&self) -> bool {
        self.calls.is_empty() && self.handlers.is_empty()
    }
}

pub trait FieldCodeGenerator {
    fn generate_imports(&self, _field: &FieldVariant) -> Vec<ImportItem> {
        vec![]
    }

    fn generate_cx_new_call(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream;

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream;

    fn generate_render_child(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> TokenStream;

    fn generate_subscription(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<GeneratedSubscription>;

    fn generate_post_subscription_initialization(
        &self,
        _field: &ResolvedField<'_>,
        _component: &GpuiFormShape,
    ) -> TokenStream {
        TokenStream::new()
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
) -> TokenStream {
    let form_components_struct_ident = component.struct_form_components_ident();
    let var_name_ident = field.field_ident_with_component_suffix().clone();
    let fn_name_ident = var_name_ident.clone();

    quote! {
        let #var_name_ident =
            cx.new(|cx| #form_components_struct_ident::#fn_name_ident(window, cx));
    }
}

pub fn generate_entity_field_initializer(field: &ResolvedField<'_>) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_component_suffix();
    quote! { #field_var_name_ident, }
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
            gpui_es_fluent::localize_message(cx, v)
        };
        #[cfg(not(feature = "fluent"))]
        let conversion_tokens = quote! { v.as_str().to_string() };

        if uses_optional_inner_validation_errors {
            quote! {
                validation_errors
                    .as_ref()
                    .and_then(|e| e.#field_name_ident())
                    .map(|inner_error| inner_error.all())
                    .filter(|errs| !errs.is_empty())
                    .map(|errs| {
                        errs.iter()
                            .map(|v| #conversion_tokens)
                            .collect::<Vec<_>>()
                            .join("\n")
                    })
            }
        } else {
            quote! {{
                validation_errors.as_ref().and_then(|e| {
                    let errs = e.#field_name_ident().all();
                    if errs.is_empty() {
                        None
                    } else {
                        Some(
                            errs.iter()
                                .map(|v| #conversion_tokens)
                                .collect::<Vec<_>>()
                            .join("\n"),
                        )
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
        FieldValuePresence, FieldVariant, GpuiFormShape, RustPath, RustType, ValidationRuleId,
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

    #[test]
    fn description_uses_direct_all_for_non_optional_newtype_errors() {
        const VALIDATIONS: &[ValidationRuleId] = &[ValidationRuleId::Newtype];
        const FIELDS: [FieldVariant; 1] =
            [
                hidden_field("index", "Age", FieldValuePresence::RequiresValue)
                    .with_validations(VALIDATIONS),
            ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            true,
        );

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains("leterrs=e.index().all();"),
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
        const FIELDS: [FieldVariant; 1] =
            [hidden_field("age", "Age", FieldValuePresence::Optional)
                .with_validations(VALIDATIONS)];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            true,
        );

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains(
                "validation_errors.as_ref().and_then(|e|e.age()).map(|inner_error|inner_error.all()).filter(|errs|!errs.is_empty()).map(|errs|{"
            ),
            "optional newtype errors should map/filter the inner errors before rendering: {compact}"
        );
    }

    #[test]
    fn description_unwraps_optional_nested_inner_errors_before_all() {
        const VALIDATIONS: &[ValidationRuleId] = &[ValidationRuleId::Nested];
        const FIELDS: [FieldVariant; 1] =
            [
                hidden_field("address", "Address", FieldValuePresence::Optional)
                    .with_validations(VALIDATIONS),
            ];
        const SHAPE: GpuiFormShape = GpuiFormShape::new(
            "Demo",
            &FIELDS,
            RustPath::from_macro_tokens_unchecked("demo"),
            true,
        );

        let field = ResolvedField::new(&FIELDS[0]).expect("field metadata should parse");
        let compact = compact(&generate_description_fn_tokens(&field, &SHAPE).to_string());

        assert!(
            compact.contains(
                "validation_errors.as_ref().and_then(|e|e.address()).map(|inner_error|inner_error.all()).filter(|errs|!errs.is_empty()).map(|errs|{"
            ),
            "optional nested errors should map/filter the inner errors before rendering: {compact}"
        );
    }
}
