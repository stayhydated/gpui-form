pub mod custom;

use gpui_form_schema::{
    components::{ComponentsBehaviour, InfiniteSelectBehaviour, SelectBehaviour},
    registry::{FieldVariant, GpuiFormShape},
};
use heck::{ToPascalCase as _, ToSnakeCase as _};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, Path, Type};

use crate::{
    error::{PrototypingError, PrototypingResult},
    imports::ImportItem,
};

static CUSTOM_GENERATOR: custom::CustomCodeGenerator = custom::CustomCodeGenerator;

pub fn field_generator(behaviour: &ComponentsBehaviour) -> &'static dyn FieldCodeGenerator {
    match behaviour {
        ComponentsBehaviour::Input
        | ComponentsBehaviour::NumberInput(_)
        | ComponentsBehaviour::Checkbox
        | ComponentsBehaviour::Switch
        | ComponentsBehaviour::Select(_)
        | ComponentsBehaviour::InfiniteSelect(_)
        | ComponentsBehaviour::Custom
        | ComponentsBehaviour::DatePicker
        | ComponentsBehaviour::FilePicker => &CUSTOM_GENERATOR,
    }
}

pub struct ResolvedField<'a> {
    field: &'a FieldVariant,
    field_ident: Ident,
    field_ident_pascal: Ident,
    field_ident_with_behaviour: Ident,
    value_type: Type,
    component_ident: Ident,
    custom_shape_path: Option<Path>,
    custom_component_path: Option<Path>,
}

impl<'a> ResolvedField<'a> {
    pub fn new(field: &'a FieldVariant) -> PrototypingResult<Self> {
        let value_type = syn::parse_str::<Type>(field.value_type).map_err(|error| {
            PrototypingError::InvalidType {
                field_name: field.field_name.to_string(),
                value: field.value_type.to_string(),
                error: error.to_string(),
            }
        })?;

        let custom_component_path = match field.custom_component {
            Some(component_path) => {
                Some(syn::parse_str::<Path>(component_path).map_err(|error| {
                    PrototypingError::InvalidPath {
                        kind: "custom component path",
                        value: component_path.to_string(),
                        error: error.to_string(),
                    }
                })?)
            },
            None => None,
        };

        let custom_shape_path = match field.custom_shape {
            Some(shape_path) => Some(syn::parse_str::<Path>(shape_path).map_err(|error| {
                PrototypingError::InvalidPath {
                    kind: "custom component shape path",
                    value: shape_path.to_string(),
                    error: error.to_string(),
                }
            })?),
            None => None,
        };

        Ok(Self {
            field,
            field_ident: format_ident!("{}", field.field_name),
            field_ident_pascal: format_ident!("{}", field.field_name_pascal()),
            field_ident_with_behaviour: format_ident!("{}", field.field_name_with_behaviour()),
            value_type,
            component_ident: format_ident!("{}", field.behaviour.component_name().to_pascal_case()),
            custom_shape_path,
            custom_component_path,
        })
    }

    pub fn raw(&self) -> &FieldVariant {
        self.field
    }

    pub fn behaviour(&self) -> &ComponentsBehaviour {
        &self.field.behaviour
    }

    pub fn field_name(&self) -> &'a str {
        self.field.field_name
    }

    pub fn field_ident(&self) -> &Ident {
        &self.field_ident
    }

    pub fn field_ident_pascal(&self) -> &Ident {
        &self.field_ident_pascal
    }

    pub fn field_ident_with_behaviour(&self) -> &Ident {
        &self.field_ident_with_behaviour
    }

    pub fn value_type(&self) -> &Type {
        &self.value_type
    }

    pub fn component_ident(&self) -> &Ident {
        &self.component_ident
    }

    pub fn optional(&self) -> bool {
        self.field.optional
    }

    pub fn value_holder_wraps_in_option(&self) -> bool {
        self.field.value_holder_wraps_in_option()
    }

    pub fn custom_value_binding(&self) -> bool {
        self.field.custom_value_binding
    }

    pub fn custom_component(&self) -> Option<&'a str> {
        self.field.custom_component
    }

    pub fn custom_shape_path(&self) -> Option<&Path> {
        self.custom_shape_path.as_ref()
    }

    pub fn custom_runtime_shape_path(&self) -> Option<Path> {
        self.custom_shape_path
            .clone()
            .map(|shape| runtime_shape_for_behaviour(shape, self.behaviour(), self.value_type()))
    }

    pub fn custom_component_path(&self) -> Option<&Path> {
        self.custom_component_path.as_ref()
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
        format_ident!("{}_{}", self.field.field_name, suffix)
    }

    pub fn prefixed_ident(&self, prefix: &str) -> Ident {
        format_ident!("{}_{}", prefix, self.field.field_name)
    }

    pub fn component_event_handler_ident(&self) -> Ident {
        format_ident!("on_{}_event", self.field_ident_with_behaviour)
    }
}

fn runtime_shape_for_behaviour(
    shape: Path,
    behaviour: &ComponentsBehaviour,
    field_type: &Type,
) -> Path {
    match behaviour {
        ComponentsBehaviour::Select(SelectBehaviour {
            searchable: true, ..
        }) => searchable_select_shape(shape, field_type),
        ComponentsBehaviour::InfiniteSelect(InfiniteSelectBehaviour {
            searchable: true, ..
        }) => searchable_infinite_select_shape(shape),
        _ => shape,
    }
}

fn shape_ident(shape: &Path) -> Option<String> {
    shape
        .segments
        .last()
        .map(|segment| segment.ident.to_string())
}

fn type_arg_count(path: &Path) -> usize {
    path.segments
        .last()
        .and_then(|segment| match &segment.arguments {
            syn::PathArguments::AngleBracketed(args) => Some(
                args.args
                    .iter()
                    .filter(|arg| matches!(arg, syn::GenericArgument::Type(_)))
                    .count(),
            ),
            _ => None,
        })
        .unwrap_or_default()
}

fn searchable_select_shape(mut shape: Path, field_type: &Type) -> Path {
    let existing_type_args = type_arg_count(&shape);
    if shape_ident(&shape).as_deref() != Some("SelectShape") || existing_type_args > 1 {
        return shape;
    }

    let selected_type = field_type.clone();
    let delegate_type: Type = syn::parse_quote! {
        ::gpui_component::select::SearchableVec<#selected_type>
    };
    let Some(segment) = shape.segments.last_mut() else {
        return shape;
    };

    match &mut segment.arguments {
        syn::PathArguments::AngleBracketed(args) => {
            if existing_type_args == 0 {
                args.args
                    .push(syn::GenericArgument::Type(field_type.clone()));
            }
            args.args.push(syn::GenericArgument::Type(delegate_type));
        },
        _ => {
            let args: syn::AngleBracketedGenericArguments =
                syn::parse_quote!(<#field_type, #delegate_type>);
            segment.arguments = syn::PathArguments::AngleBracketed(args);
        },
    }

    shape
}

fn searchable_infinite_select_shape(mut shape: Path) -> Path {
    if shape_ident(&shape).as_deref() != Some("InfiniteSelectState") || type_arg_count(&shape) > 1 {
        return shape;
    }

    if let Some(segment) = shape.segments.last_mut() {
        segment.ident = syn::Ident::new("SearchableInfiniteSelectState", segment.ident.span());
    }

    shape
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

    fn generate_focusable_cycle(
        &self,
        field: &ResolvedField<'_>,
        component: &GpuiFormShape,
    ) -> Option<TokenStream>;

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
    let var_name_ident = field.field_ident_with_behaviour().clone();
    let fn_name_ident = var_name_ident.clone();

    quote! {
        let #var_name_ident =
            cx.new(|cx| #form_components_struct_ident::#fn_name_ident(window, cx));
    }
}

pub fn generate_entity_field_initializer(field: &ResolvedField<'_>) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    quote! { #field_var_name_ident, }
}

pub fn generate_entity_focus(field: &ResolvedField<'_>) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    quote! {
        self.fields.#field_var_name_ident.focus_handle(cx),
    }
}

pub fn generate_text_value_prefill(field: &ResolvedField<'_>) -> TokenStream {
    let field_var_name_ident = field.field_ident_with_behaviour();
    let field_name_ident = field.field_ident();

    if field.value_holder_wraps_in_option() {
        quote! {
            if let Some(value) = current_data.#field_name_ident.as_ref() {
                #field_var_name_ident.update(cx, |state, cx| {
                    state.set_value(value.to_string(), window, cx);
                });
            }
        }
    } else {
        quote! {
            #field_var_name_ident.update(cx, |state, cx| {
                state.set_value(current_data.#field_name_ident.to_string(), window, cx);
            });
        }
    }
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

pub fn render_component_entity_field(
    field: &ResolvedField<'_>,
    component: &GpuiFormShape,
) -> TokenStream {
    let component_gpui_type = field.component_ident();
    let field_in_struct_name_ident = field.field_ident_with_behaviour();

    render_standard_field(
        field,
        component,
        quote! { #component_gpui_type::new(&self.fields.#field_in_struct_name_ident) },
    )
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
            localize(cx, &message)
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
            localize(cx, &message)
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
            localize(cx, v)
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
    use gpui_form_schema::{
        components::ComponentsBehaviour,
        registry::{FieldVariant, GpuiFormShape},
    };

    fn compact(input: &str) -> String {
        input.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn description_uses_direct_all_for_non_optional_newtype_errors() {
        const VALIDATIONS: &[&str] = &["NewtypeValidation"];
        const FIELDS: [FieldVariant; 1] =
            [
                FieldVariant::new("index", "Age", false, ComponentsBehaviour::Custom)
                    .with_validations(VALIDATIONS),
            ];
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
        const FIELDS: [FieldVariant; 1] =
            [
                FieldVariant::new("age", "Age", true, ComponentsBehaviour::Custom)
                    .with_validations(VALIDATIONS),
            ];
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
        const FIELDS: [FieldVariant; 1] =
            [
                FieldVariant::new("address", "Address", true, ComponentsBehaviour::Custom)
                    .with_validations(VALIDATIONS),
            ];
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
