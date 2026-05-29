pub mod shape;

use gpui_form_schema::registry::{FieldVariant, GpuiFormShape, RustExpr, RustPath, RustType};
use heck::ToSnakeCase as _;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Expr, Ident, Path, Type};

use crate::{
    error::{PrototypingError, PrototypingResult},
    imports::ImportItem,
};

static SHAPE_GENERATOR: shape::ShapeCodeGenerator = shape::ShapeCodeGenerator;

pub fn field_generator() -> &'static dyn FieldCodeGenerator {
    &SHAPE_GENERATOR
}

pub struct ResolvedField<'a> {
    field: &'a FieldVariant,
    field_ident: Ident,
    field_ident_pascal: Ident,
    field_ident_with_component_suffix: Ident,
    value_type: Type,
    shape_path: Option<Path>,
    component_type: Option<Type>,
    default_expr: Option<Expr>,
}

impl<'a> ResolvedField<'a> {
    pub fn new(field: &'a FieldVariant) -> PrototypingResult<Self> {
        let value_type = parse_field_type(field.field_name(), field.value_type())?;

        let component_type = match field.component_type() {
            Some(component_type) => Some(parse_field_type(field.field_name(), component_type)?),
            None => None,
        };

        let shape_path = match field.shape_path() {
            Some(shape_path) => Some(parse_shape_path(shape_path)?),
            None => None,
        };

        let default_expr = match field.default_expr() {
            Some(default_expr) => Some(parse_field_expr(field.field_name(), default_expr)?),
            None => None,
        };

        Ok(Self {
            field,
            field_ident: format_ident!("{}", field.field_name()),
            field_ident_pascal: format_ident!("{}", field.field_name_pascal()),
            field_ident_with_component_suffix: format_ident!(
                "{}",
                field.field_name_with_component_suffix()
            ),
            value_type,
            shape_path,
            component_type,
            default_expr,
        })
    }

    pub fn raw(&self) -> &FieldVariant {
        self.field
    }

    pub fn field_name(&self) -> &'a str {
        self.field.field_name()
    }

    pub fn field_ident(&self) -> &Ident {
        &self.field_ident
    }

    pub fn field_ident_pascal(&self) -> &Ident {
        &self.field_ident_pascal
    }

    pub fn field_ident_with_component_suffix(&self) -> &Ident {
        &self.field_ident_with_component_suffix
    }

    pub fn value_type(&self) -> &Type {
        &self.value_type
    }

    pub fn optional(&self) -> bool {
        self.field.optional()
    }

    pub fn value_holder_uses_option(&self) -> bool {
        self.field.value_holder_uses_option()
    }

    pub fn value_binding(&self) -> bool {
        self.field.value_binding()
    }

    pub fn component_type(&self) -> Option<&'a str> {
        self.field
            .component_type()
            .map(|component| component.as_str())
    }

    pub fn shape_path(&self) -> Option<&Path> {
        self.shape_path.as_ref()
    }

    pub fn runtime_shape_path(&self) -> Option<Path> {
        self.shape_path.clone()
    }

    pub fn component_type_parsed(&self) -> Option<&Type> {
        self.component_type.as_ref()
    }

    pub fn default_expr(&self) -> Option<&Expr> {
        self.default_expr.as_ref()
    }

    pub fn kebab_id(&self) -> String {
        self.field.kebab_id()
    }

    pub fn validation_rules(&self) -> &'static [&'static str] {
        self.field.validation_rules()
    }

    pub fn has_validation_rule(&self, rule: &str) -> bool {
        self.validation_rules().contains(&rule)
    }

    pub fn uses_optional_inner_validation_errors(&self) -> bool {
        self.optional()
            && (self.has_validation_rule("NewtypeValidation")
                || self.has_validation_rule("NestedValidation"))
    }

    pub fn suffixed_ident(&self, suffix: &str) -> Ident {
        format_ident!("{}_{}", self.field.field_name(), suffix)
    }

    pub fn prefixed_ident(&self, prefix: &str) -> Ident {
        format_ident!("{}_{}", prefix, self.field.field_name())
    }

    pub fn component_event_handler_ident(&self) -> Ident {
        format_ident!("on_{}_event", self.field_ident_with_component_suffix)
    }
}

fn parse_field_type(field_name: &str, value: RustType) -> PrototypingResult<Type> {
    value
        .parse()
        .map_err(|error| PrototypingError::InvalidType {
            field_name: field_name.to_string(),
            value: error.value().to_string(),
            error: error.source_error().to_string(),
        })
}

fn parse_shape_path(value: RustPath) -> PrototypingResult<Path> {
    value
        .parse()
        .map_err(|error| PrototypingError::InvalidPath {
            kind: "component shape path",
            value: error.value().to_string(),
            error: error.source_error().to_string(),
        })
}

fn parse_field_expr(field_name: &str, value: RustExpr) -> PrototypingResult<Expr> {
    value
        .parse()
        .map_err(|error| PrototypingError::InvalidExpression {
            field_name: field_name.to_string(),
            value: error.value().to_string(),
            error: error.source_error().to_string(),
        })
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
    ) -> Option<TokenStream>;

    fn generate_field_initializers(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream>;

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
    ) -> Option<TokenStream> {
        None
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
        let conversion_tokens = quote! { v.to_string() };

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
    use gpui_form_schema::registry::{FieldValuePresence, FieldVariant, GpuiFormShape, RustType};

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn description_uses_direct_all_for_non_optional_newtype_errors() {
        const VALIDATIONS: &[&str] = &["NewtypeValidation"];
        const FIELDS: [FieldVariant; 1] = [FieldVariant::hidden(
            "index",
            RustType::new_unchecked("Age"),
            FieldValuePresence::RequiresValue,
        )
        .with_validations(VALIDATIONS)];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", true);

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
        const VALIDATIONS: &[&str] = &["NewtypeValidation"];
        const FIELDS: [FieldVariant; 1] = [FieldVariant::hidden(
            "age",
            RustType::new_unchecked("Age"),
            FieldValuePresence::Optional,
        )
        .with_validations(VALIDATIONS)];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", true);

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
        const VALIDATIONS: &[&str] = &["NestedValidation"];
        const FIELDS: [FieldVariant; 1] = [FieldVariant::hidden(
            "address",
            RustType::new_unchecked("Address"),
            FieldValuePresence::Optional,
        )
        .with_validations(VALIDATIONS)];
        const SHAPE: GpuiFormShape = GpuiFormShape::new("Demo", &FIELDS, "src/demo.rs", true);

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
