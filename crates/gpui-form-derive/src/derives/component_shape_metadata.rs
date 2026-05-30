use proc_macro2::TokenStream;
use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, LitStr, Path, Result, Token, Type, Visibility,
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    spanned::Spanned as _,
};

use gpui_form_codegen::{CratePaths, components::validate_shape_field_suffix};

use super::component_shape_constructor::constructor_body_tokens;

pub(super) const SHAPE_METADATA_OPTIONS: &str = "`new = ...`, `state = ...`, `component = ...`, `value = ...`, `values(...)`, \
     `value_storage = require_value|direct`, `value_binding`, or `field_suffix = ...`";

pub(super) const FUNCTION_SHAPE_OPTIONS: &str = "`new = ...`, `component = ...`, `value = ...`, `values(...)`, \
     `value_storage = require_value|direct`, `value_binding`, or `field_suffix = ...`";

pub(super) mod kw {
    syn::custom_keyword!(component);
    syn::custom_keyword!(field_suffix);
    syn::custom_keyword!(new);
    syn::custom_keyword!(state);
    syn::custom_keyword!(value);
    syn::custom_keyword!(values);
    syn::custom_keyword!(value_storage);
    syn::custom_keyword!(value_binding);
}

fn rust_type_key(ty: &Type) -> String {
    ty.to_token_stream()
        .to_string()
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect()
}

#[derive(Clone, Copy, Debug)]
pub(super) enum ValueStoragePolicy {
    RequiredValueStorage,
    Direct,
}

impl Parse for ValueStoragePolicy {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let ident = input.parse::<Ident>()?;
        if ident == "require_value" {
            Ok(Self::RequiredValueStorage)
        } else if ident == "direct" {
            Ok(Self::Direct)
        } else {
            Err(syn::Error::new_spanned(
                ident,
                "expected `require_value` or `direct` for `value_storage`",
            ))
        }
    }
}

#[derive(Debug, Default)]
pub(super) struct ComponentShapeMetadata {
    new: Option<Expr>,
    /// Optional backing state type when deriving on a render component.
    state: Option<Type>,
    /// Optional UI component type.
    /// When set, generated prototyping code can render the component through
    /// the shape's `RenderComponent` associated type.
    component: Option<Type>,
    /// Explicit form-side value types supported by this shape.
    values: Vec<Type>,
    /// Storage policy for non-optional source fields in generated form value
    /// holders.
    value_storage: Option<ValueStoragePolicy>,
    /// Opt generated prototyping code into ComponentValueBinding by default.
    value_binding: bool,
    /// Preferred generated field/helper suffix for prototyping output.
    field_suffix: Option<LitStr>,
}

#[derive(Debug)]
pub(super) enum ShapeOption {
    New {
        expr: Expr,
        span: proc_macro2::Span,
    },
    State {
        ty: Type,
        span: proc_macro2::Span,
    },
    Component {
        ty: Type,
        span: proc_macro2::Span,
    },
    Value {
        ty: Type,
        span: proc_macro2::Span,
    },
    Values {
        values: Vec<Type>,
        span: proc_macro2::Span,
    },
    ValueStorage {
        policy: ValueStoragePolicy,
        span: proc_macro2::Span,
    },
    ValueBinding {
        span: proc_macro2::Span,
    },
    FieldSuffix {
        suffix: LitStr,
        span: proc_macro2::Span,
    },
}

impl ShapeOption {
    pub(super) fn parse_function(input: ParseStream<'_>) -> Result<Self> {
        if input.peek(kw::new) {
            let key = input.parse::<kw::new>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::New {
                expr: input.parse()?,
                span: key.span,
            });
        }
        if input.peek(kw::component) {
            let key = input.parse::<kw::component>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Component {
                ty: input.parse()?,
                span: key.span,
            });
        }
        if input.peek(kw::value) {
            let key = input.parse::<kw::value>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::Value {
                ty: input.parse()?,
                span: key.span,
            });
        }
        if input.peek(kw::values) {
            let key = input.parse::<kw::values>()?;
            let values_content;
            syn::parenthesized!(values_content in input);
            return Ok(Self::Values {
                values: ComponentShapeMetadata::parse_values(&values_content)?,
                span: key.span,
            });
        }
        if input.peek(kw::value_storage) {
            let key = input.parse::<kw::value_storage>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::ValueStorage {
                policy: input.parse()?,
                span: key.span,
            });
        }
        if input.peek(kw::value_binding) {
            let key = input.parse::<kw::value_binding>()?;
            return Ok(Self::ValueBinding { span: key.span });
        }
        if input.peek(kw::field_suffix) {
            let key = input.parse::<kw::field_suffix>()?;
            input.parse::<Token![=]>()?;
            return Ok(Self::FieldSuffix {
                suffix: input.parse()?,
                span: key.span,
            });
        }

        Err(input.error(format!("expected {FUNCTION_SHAPE_OPTIONS}")))
    }

    pub(super) fn from_nested_meta(meta: &syn::meta::ParseNestedMeta<'_>) -> Result<Self> {
        let span = meta.path.span();
        if meta.path.is_ident("new") {
            Ok(Self::New {
                expr: meta.value()?.parse()?,
                span,
            })
        } else if meta.path.is_ident("state") {
            Ok(Self::State {
                ty: meta.value()?.parse()?,
                span,
            })
        } else if meta.path.is_ident("component") {
            Ok(Self::Component {
                ty: meta.value()?.parse()?,
                span,
            })
        } else if meta.path.is_ident("value") {
            Ok(Self::Value {
                ty: meta.value()?.parse()?,
                span,
            })
        } else if meta.path.is_ident("values") {
            let content;
            syn::parenthesized!(content in meta.input);
            Ok(Self::Values {
                values: ComponentShapeMetadata::parse_values(&content)?,
                span,
            })
        } else if meta.path.is_ident("value_storage") {
            Ok(Self::ValueStorage {
                policy: meta.value()?.parse()?,
                span,
            })
        } else if meta.path.is_ident("field_suffix") {
            Ok(Self::FieldSuffix {
                suffix: meta.value()?.parse()?,
                span,
            })
        } else if meta.path.is_ident("value_binding") {
            Ok(Self::ValueBinding { span })
        } else {
            Err(meta.error(format!(
                "unsupported `gpui_form_shape` option; expected {SHAPE_METADATA_OPTIONS}",
            )))
        }
    }

    pub(super) fn apply(self, shape: &mut ComponentShapeMetadata) -> Result<()> {
        match self {
            Self::New { expr, span } => shape.set_new(expr, span_token("new", span)),
            Self::State { ty, span } => shape.set_state(ty, span_token("state", span)),
            Self::Component { ty, span } => shape.set_component(ty, span_token("component", span)),
            Self::Value { ty, span } => shape.add_value(ty, span_token("value", span)),
            Self::Values { values, span } => shape.add_values(values, span_token("values", span)),
            Self::ValueStorage { policy, span } => {
                shape.set_value_storage(policy, span_token("value_storage", span))
            },
            Self::ValueBinding { span } => {
                shape.enable_value_binding(span_token("value_binding", span))
            },
            Self::FieldSuffix { suffix, span } => {
                shape.set_field_suffix(suffix, span_token("field_suffix", span))
            },
        }
    }
}

fn span_token(name: &str, span: proc_macro2::Span) -> Ident {
    Ident::new(name, span)
}

impl ComponentShapeMetadata {
    pub(super) fn set_new<T: quote::ToTokens>(&mut self, new: Expr, span: T) -> Result<()> {
        set_once(&mut self.new, new, span, "new")
    }

    pub(super) fn set_state<T: quote::ToTokens>(&mut self, state: Type, span: T) -> Result<()> {
        set_once(&mut self.state, state, span, "state")
    }

    pub(super) fn state(&self) -> Option<&Type> {
        self.state.as_ref()
    }

    pub(super) fn set_component<T: quote::ToTokens>(
        &mut self,
        component: Type,
        span: T,
    ) -> Result<()> {
        match &component {
            Type::Path(_) => {},
            Type::Infer(_) => {
                return Err(syn::Error::new_spanned(
                    span,
                    "component type cannot be inferred with bare `_`; use an explicit component type",
                ));
            },
            _ => {
                return Err(syn::Error::new_spanned(
                    span,
                    "component metadata must be a path-like type, such as `my_crate::Input`",
                ));
            },
        }

        set_once(&mut self.component, component, span, "component")
    }

    pub(super) fn component(&self) -> Option<&Type> {
        self.component.as_ref()
    }

    pub(super) fn add_value<T: quote::ToTokens>(&mut self, value: Type, _span: T) -> Result<()> {
        let value_key = rust_type_key(&value);
        if self
            .values
            .iter()
            .any(|existing| rust_type_key(existing) == value_key)
        {
            return Err(syn::Error::new_spanned(
                value,
                "duplicate `value` compatibility metadata; remove the duplicate `value = ...` or `values(...)` entry",
            ));
        }

        self.values.push(value);
        Ok(())
    }

    pub(super) fn add_values<I, T>(&mut self, values: I, span: T) -> Result<()>
    where
        I: IntoIterator<Item = Type>,
        T: quote::ToTokens,
    {
        let mut added = false;
        for value in values {
            self.add_value(value, &span)?;
            added = true;
        }

        if !added {
            return Err(syn::Error::new_spanned(
                span,
                "`values(...)` requires at least one value type",
            ));
        }

        Ok(())
    }

    pub(super) fn parse_values(input: ParseStream<'_>) -> Result<Vec<Type>> {
        Ok(Punctuated::<Type, Token![,]>::parse_terminated(input)?
            .into_iter()
            .collect())
    }

    pub(super) fn first_value(&self) -> Option<&Type> {
        self.values.first()
    }

    pub(super) fn value_impl_tokens(
        &self,
        runtime_crate: &Path,
        ident: &Ident,
        generics: &syn::Generics,
    ) -> TokenStream {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let impls = self.values.iter().map(|value| {
            quote! {
                impl #impl_generics #runtime_crate::shape::ComponentShapeFor<#value>
                    for #ident #ty_generics
                    #where_clause
                {
                }
            }
        });

        quote! { #(#impls)* }
    }

    pub(super) fn set_value_storage<T: quote::ToTokens>(
        &mut self,
        value_storage: ValueStoragePolicy,
        span: T,
    ) -> Result<()> {
        set_once(
            &mut self.value_storage,
            value_storage,
            span,
            "value_storage",
        )
    }

    pub(super) fn enable_value_binding<T: quote::ToTokens>(&mut self, span: T) -> Result<()> {
        if self.value_binding {
            return Err(syn::Error::new_spanned(
                span,
                "duplicate `value_binding` option",
            ));
        }

        self.value_binding = true;
        Ok(())
    }

    pub(super) fn has_value_binding(&self) -> bool {
        self.value_binding
    }

    pub(super) fn set_field_suffix<T: quote::ToTokens>(
        &mut self,
        field_suffix: LitStr,
        span: T,
    ) -> Result<()> {
        validate_shape_field_suffix(&field_suffix.value())
            .map_err(|message| syn::Error::new_spanned(&field_suffix, message))?;
        set_once(&mut self.field_suffix, field_suffix, span, "field_suffix")
    }

    pub(super) fn runtime_crate_path() -> Path {
        CratePaths::resolve().gpui_form_runtime
    }

    pub(super) fn constructor_body_or(&self, default_body: TokenStream) -> TokenStream {
        self.new
            .as_ref()
            .map(constructor_body_tokens)
            .unwrap_or(default_body)
    }

    pub(super) fn render_component_tokens(
        gpui_crate: &Path,
        runtime_crate: &Path,
        vis: &Visibility,
        adapter_ident: &Ident,
        state: &Type,
        component: Option<&Type>,
        generics: &syn::Generics,
    ) -> Result<(TokenStream, TokenStream)> {
        let Some(component) = component else {
            return Ok((
                quote! { type RenderComponent = #runtime_crate::shape::NoRenderComponent; },
                quote! {},
            ));
        };

        let component = component.clone();
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let phantom_type = phantom_type_tokens(generics);

        Ok((
            quote! { type RenderComponent = #adapter_ident #ty_generics; },
            quote! {
                #[doc(hidden)]
                #vis struct #adapter_ident #generics(
                    ::core::marker::PhantomData<fn() -> #phantom_type>
                ) #where_clause;

                impl #impl_generics #runtime_crate::shape::ComponentRender<#state>
                    for #adapter_ident #ty_generics
                    #where_clause
                {
                    const RENDERS: bool = true;

                    fn new(entity: &#gpui_crate::Entity<#state>) -> impl #gpui_crate::IntoElement {
                        <#component>::new(entity)
                    }
                }
            },
        ))
    }

    pub(super) fn impl_items_tokens(
        &self,
        runtime_crate: &Path,
        render_component_assoc: TokenStream,
    ) -> TokenStream {
        let value_storage_policy = match self
            .value_storage
            .unwrap_or(ValueStoragePolicy::RequiredValueStorage)
        {
            ValueStoragePolicy::RequiredValueStorage => {
                quote! { #runtime_crate::shape::RequiredValueStorage }
            },
            ValueStoragePolicy::Direct => quote! { #runtime_crate::shape::DirectValueStorage },
        };
        let value_storage_policy_assoc = quote! {
            type ValueStoragePolicy = #value_storage_policy;
        };
        let value_binding_policy = if self.value_binding {
            quote! { #runtime_crate::shape::InheritedComponentValueBinding }
        } else {
            quote! { #runtime_crate::shape::NoComponentValueBinding }
        };
        let value_binding_policy_assoc = quote! {
            type ValueBindingPolicy = #value_binding_policy;
        };
        let prototyping_const = self.field_suffix.as_ref().map(|field_suffix| {
            quote! {
                const PROTOTYPING: #runtime_crate::shape::ComponentPrototyping =
                    #runtime_crate::shape::ComponentPrototyping::new()
                        .field_suffix(#field_suffix);
            }
        });

        quote! {
            #value_storage_policy_assoc
            #value_binding_policy_assoc
            #render_component_assoc
            #prototyping_const
        }
    }
}

fn phantom_type_tokens(generics: &syn::Generics) -> TokenStream {
    let params: Vec<TokenStream> = generics
        .params
        .iter()
        .filter_map(|param| match param {
            syn::GenericParam::Type(param) => {
                let ident = &param.ident;
                Some(quote! { #ident })
            },
            syn::GenericParam::Lifetime(param) => {
                let lifetime = &param.lifetime;
                Some(quote! { &#lifetime () })
            },
            syn::GenericParam::Const(_) => None,
        })
        .collect();

    if params.is_empty() {
        quote! { () }
    } else {
        quote! { (#(#params),*) }
    }
}

fn set_once<T, S: quote::ToTokens>(
    slot: &mut Option<T>,
    value: T,
    span: S,
    option: &str,
) -> Result<()> {
    if slot.replace(value).is_some() {
        return Err(syn::Error::new_spanned(
            span,
            format!("duplicate `{option}` option"),
        ));
    }

    Ok(())
}
