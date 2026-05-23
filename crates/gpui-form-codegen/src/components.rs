use darling::{Error as DarlingError, FromMeta, util::Flag};
use gpui_form_schema::components::{
    ComponentKind, ComponentsBehaviour, InfiniteSelectBehaviour, SelectBehaviour,
};
use proc_macro2::TokenStream;
use quote::{ToTokens as _, quote};
use syn::{Expr, Lit, Path, Token, parse::Parser as _, punctuated::Punctuated};

use crate::implementations::ComponentLayout as _;

pub trait ComponentOption {}

pub struct FieldInformation<T: ComponentOption> {
    pub options: T,
    pub name: String,
    pub r#type: syn::Type,
}

impl<T: ComponentOption> FieldInformation<T> {
    pub fn new(options: T, name: String, r#type: syn::Type) -> Self {
        Self {
            options,
            name,
            r#type,
        }
    }
}

pub struct GeneratedFieldLayout {
    pub field_structure_tokens: TokenStream,
    pub field_base_declarations_tokens: TokenStream,
    pub requires_value: bool,
}

#[derive(Clone, Debug)]
pub struct ComponentMethod {
    pub method: syn::Ident,
    pub args: Vec<Expr>,
}

#[derive(Clone, Debug)]
pub struct CustomOptions {
    /// Path to a type implementing `gpui_form_component::custom::CustomComponentShape`.
    pub shape: syn::Path,
    /// UI component type path (e.g. `TagsInput`).
    /// When provided, the prototyping code generator emits `Component::new(&entity)`.
    pub component: Option<syn::Path>,
    /// Whether non-optional source fields should reject a missing holder value.
    /// This is inferred from known shape types for the component expression syntax.
    pub requires_value: bool,
    /// User-facing component behavior metadata carried by this shape.
    pub behaviour: ComponentsBehaviour,
    /// Field-level default value expression, used by known shapes that can seed
    /// component state from the model default.
    pub field_default: Option<syn::Expr>,
    /// Whether prototyping code should wire this custom component through
    /// `CustomComponentValueAdapter`.
    pub value_binding: Option<bool>,
    /// Optional explicit generated field/helper suffix for prototyping output.
    pub field_suffix: Option<String>,
    component_methods: Vec<ComponentMethod>,
}

#[derive(Debug, Default, FromMeta)]
struct CustomOptionsMeta {
    #[darling(default)]
    shape: Option<syn::Path>,
    #[darling(default)]
    state: Option<syn::Path>,
    #[darling(default)]
    component: Option<syn::Path>,
    #[darling(default)]
    requires_value: Option<bool>,
    #[darling(default)]
    value_binding: Flag,
    #[darling(default)]
    field_suffix: Option<String>,
}

impl CustomOptions {
    fn from_shape(shape: Path) -> Self {
        let shape = normalize_shape_path(shape);
        let (behaviour, requires_value, field_suffix) = inferred_shape_defaults(&shape);

        Self {
            shape,
            component: None,
            requires_value,
            behaviour,
            field_default: None,
            value_binding: None,
            field_suffix: field_suffix.map(str::to_owned),
            component_methods: Vec::new(),
        }
    }

    fn from_meta(meta: CustomOptionsMeta) -> darling::Result<Self> {
        let CustomOptionsMeta {
            shape,
            state,
            component,
            requires_value,
            value_binding,
            field_suffix,
        } = meta;

        let shape = normalize_shape_path(match (shape, state) {
            (Some(shape), None) | (None, Some(shape)) => shape,
            (Some(_), Some(_)) => {
                return Err(DarlingError::custom(
                    "custom component may specify only one of `shape` or `state`",
                ));
            },
            (None, None) => {
                return Err(DarlingError::custom(
                    "custom component requires `shape = ...` or `state = ...`",
                ));
            },
        });

        let mut options = Self::from_shape(shape);
        options.component = component;
        if let Some(requires_value) = requires_value {
            options.requires_value = requires_value;
        }
        options.value_binding = value_binding.is_present().then_some(true);
        options.field_suffix = field_suffix;

        Ok(options)
    }

    fn from_component_expr(expr: &Expr) -> darling::Result<Self> {
        let (shape, methods) = analyze_component_expr(expr)?;
        let mut options = Self::from_shape(shape);
        let mut behavior_seen = false;
        let mut metadata_seen = false;

        for method in &methods {
            let method_name = method.method.to_string();
            if method_name == "builder" {
                return Err(DarlingError::custom(
                    "component behavior chains start setters directly; use \
                     `Select::<_>::searchable(true)` instead of calling `builder()`",
                )
                .with_span(&method.method));
            }

            let is_behavior = is_component_behavior_method(&method.method);
            if is_behavior && metadata_seen {
                return Err(DarlingError::custom(
                    "component behavior setters must start the component expression",
                )
                .with_span(&method.method));
            }
            if !is_behavior && behavior_seen {
                return Err(DarlingError::custom(
                    "component behavior chains only accept behavior setters",
                )
                .with_span(&method.method));
            }

            options.apply_component_method(&method.method, &method.args)?;
            behavior_seen |= is_behavior;
            metadata_seen |= !is_behavior;
        }

        options.component_methods = methods;
        Ok(options)
    }

    fn from_component_meta_list(meta_list: &syn::MetaList) -> darling::Result<Self> {
        let mut shape = meta_list.path.clone();
        let method = pop_component_method(&mut shape)?;
        let args = parse_expr_args(meta_list.tokens.clone())
            .map_err(|err| DarlingError::custom(err.to_string()).with_span(meta_list))?;

        let mut options = Self::from_shape(shape);
        options.apply_component_method(&method, &args)?;
        Ok(options)
    }

    fn apply_component_method(
        &mut self,
        method: &syn::Ident,
        args: &[Expr],
    ) -> darling::Result<()> {
        match method.to_string().as_str() {
            "value_binding" => {
                self.value_binding = Some(expect_optional_bool_arg(method, args)?);
            },
            "component" => {
                self.component = Some(expect_path_arg(method, args)?);
            },
            "field_suffix" => {
                self.field_suffix = Some(expect_string_arg(method, args)?);
            },
            "searchable" => {
                let searchable = expect_bool_arg(method, args)?;
                match &mut self.behaviour {
                    ComponentsBehaviour::Select(behaviour) => {
                        behaviour.searchable = searchable;
                    },
                    ComponentsBehaviour::InfiniteSelect(behaviour) => {
                        behaviour.searchable = searchable;
                    },
                    _ => {
                        return Err(DarlingError::custom(
                            "`searchable` is only supported by gpui_form_collection::select::Select and gpui_form_component::infinite_select::InfiniteSelect",
                        )
                        .with_span(method));
                    },
                }
            },
            "partial" => {
                let partial = expect_bool_arg(method, args)?;
                match &mut self.behaviour {
                    ComponentsBehaviour::Select(behaviour) => {
                        behaviour.partial = partial;
                    },
                    _ => {
                        return Err(DarlingError::custom(
                            "`partial` is only supported by gpui_form_collection::select::Select",
                        )
                        .with_span(method));
                    },
                }
            },
            "max_depth" => {
                let max_depth = expect_usize_arg(method, args)?;
                match &mut self.behaviour {
                    ComponentsBehaviour::InfiniteSelect(behaviour) => {
                        behaviour.max_depth = Some(max_depth);
                    },
                    _ => {
                        return Err(DarlingError::custom(
                            "`max_depth` is only supported by gpui_form_component::infinite_select::InfiniteSelect",
                        )
                        .with_span(method));
                    },
                }
            },
            _ => {
                return Err(DarlingError::custom(format!(
                    "unknown component behavior `{method}`; supported methods are \
                     `value_binding`, `component`, `field_suffix`, `searchable`, \
                     `partial`, and `max_depth`"
                ))
                .with_span(method));
            },
        }

        Ok(())
    }

    pub fn resolved_shape(&self, field_type: &syn::Type) -> syn::Path {
        substitute_infer_in_path(&self.shape, field_type)
    }

    pub fn runtime_shape(&self, field_type: &syn::Type) -> syn::Path {
        let shape = self.resolved_shape(field_type);

        match &self.behaviour {
            ComponentsBehaviour::Select(behaviour) if behaviour.searchable => {
                searchable_select_shape(shape, field_type)
            },
            ComponentsBehaviour::InfiniteSelect(behaviour) if behaviour.searchable => {
                searchable_infinite_select_shape(shape)
            },
            _ => shape,
        }
    }

    pub fn with_field_type(mut self, field_type: &syn::Type) -> Self {
        self.shape = self.resolved_shape(field_type);
        self
    }

    pub fn with_field_default(mut self, field_default: Option<syn::Expr>) -> Self {
        self.field_default = field_default;
        self
    }

    pub fn component_suffix(&self, field_name: &str) -> String {
        if let Some(field_suffix) = &self.field_suffix {
            return gpui_form_schema::registry::custom_component_suffix_from_suffix(
                field_name,
                field_suffix,
            )
            .unwrap_or_else(|| ComponentKind::Custom.component_name().to_string());
        }

        let shape = self.shape.to_token_stream().to_string();
        gpui_form_schema::registry::custom_component_suffix_from_shape(field_name, &shape)
            .unwrap_or_else(|| ComponentKind::Custom.component_name().to_string())
    }

    pub fn constructor_tokens(&self, field_type: &syn::Type) -> TokenStream {
        let shape = self.runtime_shape(field_type);
        let constructor_shape = turbofish_shape_path(shape.clone());

        match &self.behaviour {
            ComponentsBehaviour::Select(behaviour) => {
                let constructor = if let Some(default_expr) = self.field_default.as_ref() {
                    quote! {
                        #constructor_shape::new_with_initial(
                            {
                                let __gpui_form_default = #default_expr;
                                __gpui_form_default
                            },
                            window,
                            cx,
                        )
                    }
                } else {
                    quote! {
                        <#shape as ::gpui_form_component::custom::CustomComponentShape>::new(
                            window,
                            cx,
                        )
                    }
                };

                if behaviour.searchable {
                    quote! {
                        #constructor.searchable(true)
                    }
                } else {
                    constructor
                }
            },
            ComponentsBehaviour::InfiniteSelect(behaviour) => {
                let options = infinite_select_options_tokens(behaviour);
                let initial_value = if let Some(default_expr) = self.field_default.as_ref() {
                    quote! {
                        {
                            let __gpui_form_default = #default_expr;
                            __gpui_form_default
                        }
                    }
                } else {
                    quote! { ::core::default::Default::default() }
                };
                quote! {
                    #constructor_shape::new_with_options(
                        #initial_value,
                        #options,
                        window,
                        cx,
                    )
                }
            },
            _ => {
                quote! {
                    <#shape as ::gpui_form_component::custom::CustomComponentShape>::new(window, cx)
                }
            },
        }
    }

    pub fn type_check_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        if self.component_methods.is_empty()
            || !self
                .component_methods
                .iter()
                .all(|method| is_component_behavior_method(&method.method))
        {
            return None;
        }

        let shape = turbofish_shape_path(self.resolved_shape(field_type));
        let mut methods = self.component_methods.iter();
        let first = methods.next()?;
        let first_method_name = &first.method;
        let first_args = &first.args;
        let setter_calls = methods.map(|method| {
            let method_name = &method.method;
            let args = &method.args;
            quote! { .#method_name(#(#args),*) }
        });

        Some(quote! {
            let _ = #shape::#first_method_name(#(#first_args),*) #(#setter_calls)* .build();
        })
    }
}

impl FromMeta for CustomOptions {
    fn from_word() -> darling::Result<Self> {
        Err(DarlingError::custom(
            "custom component requires `shape = ...` or `state = ...`",
        ))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let meta = CustomOptionsMeta::from_list(items)?;
        Self::from_meta(meta)
    }
}

fn analyze_component_expr(expr: &Expr) -> darling::Result<(Path, Vec<ComponentMethod>)> {
    match expr {
        Expr::Group(group) => analyze_component_expr(&group.expr),
        Expr::Paren(paren) => analyze_component_expr(&paren.expr),
        Expr::MethodCall(method_call) => {
            let (shape, mut methods) = analyze_component_expr(&method_call.receiver)?;
            methods.push(ComponentMethod {
                method: method_call.method.clone(),
                args: method_call.args.iter().cloned().collect(),
            });
            Ok((shape, methods))
        },
        Expr::Call(call) => analyze_component_call_expr(call),
        Expr::Path(path) => Ok((path.path.clone(), Vec::new())),
        Expr::Lit(expr_lit) => {
            if let Lit::Str(value) = &expr_lit.lit {
                return value
                    .parse::<Path>()
                    .map(|shape| (shape, Vec::new()))
                    .map_err(|err| DarlingError::custom(err.to_string()).with_span(value));
            }

            Err(DarlingError::unexpected_lit_type(&expr_lit.lit))
        },
        _ => Err(DarlingError::custom(
            "component syntax expects a shape path or shape behavior expression",
        )
        .with_span(expr)),
    }
}

fn analyze_component_call_expr(
    call: &syn::ExprCall,
) -> darling::Result<(Path, Vec<ComponentMethod>)> {
    let func = match &*call.func {
        Expr::Group(group) => &group.expr,
        Expr::Paren(paren) => &paren.expr,
        other => other,
    };

    let Expr::Path(path_expr) = func else {
        return Err(DarlingError::custom(
            "component call must be an associated function on the shape path",
        )
        .with_span(func));
    };

    let mut shape = path_expr.path.clone();
    let method = pop_component_method(&mut shape)?;
    Ok((
        shape,
        vec![ComponentMethod {
            method,
            args: call.args.iter().cloned().collect(),
        }],
    ))
}

fn pop_component_method(path: &mut Path) -> darling::Result<syn::Ident> {
    let method = path
        .segments
        .last()
        .map(|segment| segment.ident.clone())
        .ok_or_else(|| DarlingError::custom("component expression requires a shape path"))?;

    path.segments.pop();
    path.segments.pop_punct();
    if path.segments.is_empty() {
        return Err(DarlingError::custom(
            "component expression requires a shape path before the behavior method",
        )
        .with_span(&method));
    }

    Ok(method)
}

fn normalize_shape_path(mut path: Path) -> Path {
    for segment in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
            args.colon2_token = None;
        }
    }

    path
}

fn turbofish_shape_path(mut path: Path) -> Path {
    for segment in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
            args.colon2_token = Some(Default::default());
        }
    }

    path
}

fn parse_expr_args(tokens: TokenStream) -> syn::Result<Vec<Expr>> {
    if tokens.is_empty() {
        return Ok(Vec::new());
    }

    Punctuated::<Expr, Token![,]>::parse_terminated
        .parse2(tokens)
        .map(|args| args.into_iter().collect())
}

fn is_component_behavior_method(method: &syn::Ident) -> bool {
    matches!(
        method.to_string().as_str(),
        "searchable" | "partial" | "max_depth"
    )
}

fn expect_bool_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<bool> {
    let [arg] = args else {
        return Err(DarlingError::custom(format!(
            "`{method}` expects exactly one boolean argument"
        ))
        .with_span(method));
    };

    match arg {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Bool(value) => Ok(value.value),
            lit => Err(DarlingError::unexpected_lit_type(lit).with_span(arg)),
        },
        _ => Err(DarlingError::unexpected_expr_type(arg).with_span(arg)),
    }
}

fn expect_optional_bool_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<bool> {
    match args {
        [] => Ok(true),
        [_] => expect_bool_arg(method, args),
        _ => Err(DarlingError::custom(format!(
            "`{method}` expects zero arguments or one boolean argument"
        ))
        .with_span(method)),
    }
}

fn expect_path_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<Path> {
    let [arg] = args else {
        return Err(
            DarlingError::custom(format!("`{method}` expects exactly one path argument"))
                .with_span(method),
        );
    };

    match arg {
        Expr::Path(path) => Ok(path.path.clone()),
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(value) => value
                .parse::<Path>()
                .map_err(|err| DarlingError::custom(err.to_string()).with_span(value)),
            lit => Err(DarlingError::unexpected_lit_type(lit).with_span(arg)),
        },
        _ => Err(DarlingError::unexpected_expr_type(arg).with_span(arg)),
    }
}

fn expect_string_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<String> {
    let [arg] = args else {
        return Err(DarlingError::custom(format!(
            "`{method}` expects exactly one string literal argument"
        ))
        .with_span(method));
    };

    match arg {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Str(value) => Ok(value.value()),
            lit => Err(DarlingError::unexpected_lit_type(lit).with_span(arg)),
        },
        _ => Err(DarlingError::unexpected_expr_type(arg).with_span(arg)),
    }
}

fn expect_usize_arg(method: &syn::Ident, args: &[Expr]) -> darling::Result<usize> {
    let [arg] = args else {
        return Err(DarlingError::custom(format!(
            "`{method}` expects exactly one integer argument"
        ))
        .with_span(method));
    };

    match arg {
        Expr::Lit(expr_lit) => match &expr_lit.lit {
            Lit::Int(value) => value
                .base10_parse::<usize>()
                .map_err(|err| DarlingError::custom(err.to_string()).with_span(value)),
            lit => Err(DarlingError::unexpected_lit_type(lit).with_span(arg)),
        },
        _ => Err(DarlingError::unexpected_expr_type(arg).with_span(arg)),
    }
}

fn inferred_shape_defaults(shape: &Path) -> (ComponentsBehaviour, bool, Option<&'static str>) {
    if is_collection_shape(shape, "input", "Input") {
        (ComponentsBehaviour::Input, true, Some("input"))
    } else if is_collection_shape(shape, "checkbox", "Checkbox") {
        (ComponentsBehaviour::Checkbox, false, Some("checkbox"))
    } else if is_collection_shape(shape, "switch", "Switch") {
        (ComponentsBehaviour::Switch, false, Some("switch"))
    } else if is_collection_shape(shape, "select", "Select") {
        (
            ComponentsBehaviour::Select(SelectBehaviour::default()),
            false,
            Some("select"),
        )
    } else if is_infinite_select_shape(shape, "InfiniteSelect") {
        (
            ComponentsBehaviour::InfiniteSelect(InfiniteSelectBehaviour::default()),
            false,
            Some("infinite_select"),
        )
    } else if is_infinite_select_shape(shape, "SearchableInfiniteSelect") {
        (
            ComponentsBehaviour::InfiniteSelect(InfiniteSelectBehaviour {
                searchable: true,
                max_depth: None,
            }),
            false,
            Some("infinite_select"),
        )
    } else {
        (ComponentsBehaviour::Custom, true, None)
    }
}

fn is_collection_shape(shape: &Path, module: &str, ident: &str) -> bool {
    path_ends_with(shape, &["gpui_form_collection", module, ident])
}

fn is_infinite_select_shape(shape: &Path, ident: &str) -> bool {
    path_ends_with(shape, &["gpui_form_component", "infinite_select", ident])
}

fn path_ends_with(path: &Path, expected: &[&str]) -> bool {
    let actual = path
        .segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>();

    actual.len() >= expected.len()
        && actual
            .iter()
            .rev()
            .zip(expected.iter().rev())
            .all(|(actual, expected)| actual == expected)
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

fn searchable_select_shape(mut shape: Path, field_type: &syn::Type) -> Path {
    let existing_type_args = type_arg_count(&shape);
    if !is_collection_shape(&shape, "select", "Select") || existing_type_args > 1 {
        return shape;
    }

    let selected_type = field_type.clone();
    let delegate_type: syn::Type = syn::parse_quote! {
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
    if !is_infinite_select_shape(&shape, "InfiniteSelect") || type_arg_count(&shape) > 1 {
        return shape;
    }

    if let Some(segment) = shape.segments.last_mut() {
        segment.ident = syn::Ident::new("SearchableInfiniteSelect", segment.ident.span());
    }

    shape
}

fn infinite_select_options_tokens(behaviour: &InfiniteSelectBehaviour) -> TokenStream {
    let searchable = behaviour.searchable;
    let max_depth = match behaviour.max_depth {
        Some(max_depth) => quote! { Some(#max_depth) },
        None => quote! { None },
    };

    quote! {
        ::gpui_form_component::infinite_select::InfiniteSelectOptions::new(
            #searchable,
            #max_depth,
        )
    }
}

fn behaviour_tokens(behaviour: &ComponentsBehaviour) -> TokenStream {
    match behaviour {
        ComponentsBehaviour::Input => {
            quote! { ::gpui_form::schema::components::ComponentsBehaviour::Input }
        },
        ComponentsBehaviour::NumberInput(options) => {
            let validation_type = match options.validation_type {
                Some(value) => quote! { Some(#value) },
                None => quote! { None },
            };
            let kind = match options.kind {
                gpui_form_schema::components::NumberInputKind::Float => {
                    quote! { ::gpui_form::schema::components::NumberInputKind::Float }
                },
                gpui_form_schema::components::NumberInputKind::SignedInteger => {
                    quote! { ::gpui_form::schema::components::NumberInputKind::SignedInteger }
                },
                gpui_form_schema::components::NumberInputKind::UnsignedInteger => {
                    quote! { ::gpui_form::schema::components::NumberInputKind::UnsignedInteger }
                },
                gpui_form_schema::components::NumberInputKind::Custom => {
                    quote! { ::gpui_form::schema::components::NumberInputKind::Custom }
                },
            };

            quote! {
                ::gpui_form::schema::components::ComponentsBehaviour::NumberInput(
                    ::gpui_form::schema::components::NumberInputBehaviour {
                        validation_type: #validation_type,
                        kind: #kind,
                    }
                )
            }
        },
        ComponentsBehaviour::Checkbox => {
            quote! { ::gpui_form::schema::components::ComponentsBehaviour::Checkbox }
        },
        ComponentsBehaviour::Switch => {
            quote! { ::gpui_form::schema::components::ComponentsBehaviour::Switch }
        },
        ComponentsBehaviour::Select(options) => {
            let searchable = options.searchable;
            let partial = options.partial;
            quote! {
                ::gpui_form::schema::components::ComponentsBehaviour::Select(
                    ::gpui_form::schema::components::SelectBehaviour {
                        partial: #partial,
                        searchable: #searchable,
                    }
                )
            }
        },
        ComponentsBehaviour::InfiniteSelect(options) => {
            let searchable = options.searchable;
            let max_depth = match options.max_depth {
                Some(max_depth) => quote! { Some(#max_depth) },
                None => quote! { None },
            };
            quote! {
                ::gpui_form::schema::components::ComponentsBehaviour::InfiniteSelect(
                    ::gpui_form::schema::components::InfiniteSelectBehaviour {
                        searchable: #searchable,
                        max_depth: #max_depth,
                    }
                )
            }
        },
        ComponentsBehaviour::Custom => {
            quote! { ::gpui_form::schema::components::ComponentsBehaviour::Custom }
        },
        ComponentsBehaviour::DatePicker => {
            quote! { ::gpui_form::schema::components::ComponentsBehaviour::DatePicker }
        },
        ComponentsBehaviour::FilePicker => {
            quote! { ::gpui_form::schema::components::ComponentsBehaviour::FilePicker }
        },
    }
}

fn substitute_infer_in_type(ty: &syn::Type, replacement: &syn::Type) -> syn::Type {
    match ty {
        syn::Type::Infer(_) => replacement.clone(),
        syn::Type::Path(type_path) => {
            let mut type_path = type_path.clone();
            type_path.path = substitute_infer_in_path(&type_path.path, replacement);
            syn::Type::Path(type_path)
        },
        syn::Type::Tuple(tuple) => {
            let mut tuple = tuple.clone();
            tuple.elems = tuple
                .elems
                .iter()
                .map(|ty| substitute_infer_in_type(ty, replacement))
                .collect();
            syn::Type::Tuple(tuple)
        },
        syn::Type::Paren(paren) => {
            let mut paren = paren.clone();
            paren.elem = Box::new(substitute_infer_in_type(&paren.elem, replacement));
            syn::Type::Paren(paren)
        },
        syn::Type::Group(group) => {
            let mut group = group.clone();
            group.elem = Box::new(substitute_infer_in_type(&group.elem, replacement));
            syn::Type::Group(group)
        },
        syn::Type::Reference(reference) => {
            let mut reference = reference.clone();
            reference.elem = Box::new(substitute_infer_in_type(&reference.elem, replacement));
            syn::Type::Reference(reference)
        },
        _ => ty.clone(),
    }
}

fn substitute_infer_in_path(path: &syn::Path, replacement: &syn::Type) -> syn::Path {
    let mut path = path.clone();

    for segment in &mut path.segments {
        if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
            for arg in &mut args.args {
                match arg {
                    syn::GenericArgument::Type(ty) => {
                        *ty = substitute_infer_in_type(ty, replacement);
                    },
                    syn::GenericArgument::AssocType(assoc_type) => {
                        assoc_type.ty = substitute_infer_in_type(&assoc_type.ty, replacement);
                    },
                    _ => {},
                }
            }
        }
    }

    path
}

impl ComponentOption for CustomOptions {}

pub struct CustomComponent(pub FieldInformation<CustomOptions>);

impl CustomComponent {
    pub fn component_name() -> &'static str {
        ComponentKind::Custom.component_name()
    }
}

#[derive(Clone, Debug)]
pub enum Components {
    Custom(CustomOptions),
}

impl FromMeta for Components {
    fn from_word() -> darling::Result<Self> {
        Err(DarlingError::custom(
            "component requires a shape expression, for example \
             `component = my::Shape`",
        ))
    }

    fn from_expr(expr: &Expr) -> darling::Result<Self> {
        CustomOptions::from_component_expr(expr).map(Self::Custom)
    }

    fn from_string(value: &str) -> darling::Result<Self> {
        if let Ok(expr) = syn::parse_str::<Expr>(value)
            && let Ok(component) = Self::from_expr(&expr)
        {
            return Ok(component);
        }

        syn::parse_str::<Path>(value)
            .map(CustomOptions::from_shape)
            .map(Self::Custom)
            .map_err(|err| DarlingError::custom(err.to_string()))
    }

    fn from_list(items: &[darling::ast::NestedMeta]) -> darling::Result<Self> {
        let [item] = items else {
            return Err(DarlingError::custom(
                "component expects one shape expression or one `custom(...)` block",
            ));
        };

        match item {
            darling::ast::NestedMeta::Meta(syn::Meta::List(meta_list))
                if meta_list.path.is_ident("custom") =>
            {
                <CustomOptions as FromMeta>::from_meta(&syn::Meta::List(meta_list.clone()))
                    .map(Self::Custom)
            },
            darling::ast::NestedMeta::Meta(syn::Meta::Path(path)) => {
                Ok(Self::Custom(CustomOptions::from_shape(path.clone())))
            },
            darling::ast::NestedMeta::Meta(syn::Meta::List(meta_list)) => {
                CustomOptions::from_component_meta_list(meta_list).map(Self::Custom)
            },
            darling::ast::NestedMeta::Lit(Lit::Str(value)) => value
                .parse::<Path>()
                .map(CustomOptions::from_shape)
                .map(Self::Custom)
                .map_err(|err| DarlingError::custom(err.to_string()).with_span(value)),
            darling::ast::NestedMeta::Lit(lit) => Err(DarlingError::unexpected_lit_type(lit)),
            darling::ast::NestedMeta::Meta(meta) => Err(DarlingError::custom(
                "unsupported component metadata; use a shape expression or `custom(...)`",
            )
            .with_span(meta)),
        }
    }
}

impl Components {
    pub const fn kind(&self) -> ComponentKind {
        match self {
            Self::Custom(options) => options.behaviour.kind(),
        }
    }

    pub fn requires_value(&self) -> bool {
        match self {
            Self::Custom(options) => options.requires_value,
        }
    }

    pub fn generate_field_layout(
        &self,
        field_name: String,
        field_type: syn::Type,
        field_default: Option<syn::Expr>,
    ) -> GeneratedFieldLayout {
        let mut field_structure_tokens = TokenStream::new();
        let mut field_base_declarations_tokens = TokenStream::new();

        match self {
            Self::Custom(options) => {
                let options = options
                    .clone()
                    .with_field_default(field_default)
                    .with_field_type(&field_type);
                let component =
                    CustomComponent(FieldInformation::new(options, field_name, field_type));
                component.field_tokens(
                    &mut field_structure_tokens,
                    &mut field_base_declarations_tokens,
                );
            },
        }

        GeneratedFieldLayout {
            field_structure_tokens,
            field_base_declarations_tokens,
            requires_value: self.requires_value(),
        }
    }

    pub fn behaviour_tokens(&self, _field_type: &syn::Type) -> TokenStream {
        match self {
            Self::Custom(options) => behaviour_tokens(&options.behaviour),
        }
    }

    pub fn custom_component_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Custom(options) = self;

        let shape = options.resolved_shape(field_type);
        if let Some(component) = options.component.as_ref() {
            let component_str = component.to_token_stream().to_string();
            Some(quote! { .with_custom_component(#component_str) })
        } else {
            Some(quote! {
                .with_custom_component_opt(
                    <#shape as ::gpui_form_component::custom::CustomComponentShape>::COMPONENT_PATH
                )
            })
        }
    }

    pub fn custom_shape_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Custom(options) = self;

        let shape = options
            .resolved_shape(field_type)
            .to_token_stream()
            .to_string();
        Some(quote! { .with_custom_shape(#shape) })
    }

    pub fn custom_value_binding_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Custom(options) = self;

        let shape = options.resolved_shape(field_type);

        Some(match options.value_binding {
            Some(true) => quote! { .with_custom_value_binding(true) },
            Some(false) => quote! { .with_custom_value_binding(false) },
            None => {
                quote! {
                    .with_custom_value_binding(
                        <#shape as ::gpui_form_component::custom::CustomComponentShape>::VALUE_BINDING
                    )
                }
            },
        })
    }

    pub fn custom_prototyping_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        let Self::Custom(options) = self;

        if let Some(field_suffix) = &options.field_suffix {
            let field_suffix = syn::LitStr::new(field_suffix, proc_macro2::Span::call_site());
            Some(quote! {
                .with_custom_prototyping_field_suffix(Some(#field_suffix))
            })
        } else {
            let shape = options.resolved_shape(field_type);

            Some(quote! {
                .with_custom_prototyping_field_suffix(
                    <#shape as ::gpui_form_component::custom::CustomComponentShape>::PROTOTYPING
                        .field_suffix
                )
            })
        }
    }

    pub fn type_check_tokens(&self, field_type: &syn::Type) -> Option<TokenStream> {
        match self {
            Self::Custom(options) => options.type_check_tokens(field_type),
        }
    }
}
