use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    GenericArgument, ImplItem, ItemImpl, PathArguments, Result, ReturnType, Type,
    parse_macro_input, parse_quote,
};

fn component_value_type(item: &ItemImpl) -> Result<Type> {
    let Some((_, trait_path, _)) = &item.trait_ else {
        return Err(syn::Error::new_spanned(
            item,
            "`component_value_binding` must be applied to an impl block for \
             `ComponentValueBinding<T>`",
        ));
    };

    let Some(segment) = trait_path.segments.last() else {
        return Err(syn::Error::new_spanned(
            trait_path,
            "expected `ComponentValueBinding<T>`",
        ));
    };

    if segment.ident != "ComponentValueBinding" {
        return Err(syn::Error::new_spanned(
            &segment.ident,
            "`component_value_binding` must wrap `ComponentValueBinding<T>`",
        ));
    }

    let PathArguments::AngleBracketed(args) = &segment.arguments else {
        return Err(syn::Error::new_spanned(
            segment,
            "expected `ComponentValueBinding<T>`",
        ));
    };

    if args.args.len() != 1 {
        return Err(syn::Error::new_spanned(
            &args.args,
            "`ComponentValueBinding` requires exactly one type argument",
        ));
    }

    match args.args.first().expect("checked len") {
        GenericArgument::Type(ty) => Ok(ty.clone()),
        arg => Err(syn::Error::new_spanned(
            arg,
            "`ComponentValueBinding` argument must be a type",
        )),
    }
}

fn expand(mut item: ItemImpl) -> Result<TokenStream> {
    let value_type = component_value_type(&item)?;

    if let Some((_, trait_path, _)) = &mut item.trait_ {
        *trait_path =
            parse_quote!(::gpui_form_runtime::shape::ComponentStateValueBinding<#value_type>);
    }

    for impl_item in &mut item.items {
        if let ImplItem::Fn(method) = impl_item {
            for arg in &mut method.sig.inputs {
                if let syn::FnArg::Typed(pat_type) = arg {
                    rewrite_self_state_type(&mut pat_type.ty);
                }
            }
            if let ReturnType::Type(_, ty) = &mut method.sig.output {
                rewrite_self_state_type(ty);
            }
        }
    }

    Ok(quote! {
        #item
    })
}

fn rewrite_self_state_type(ty: &mut Type) {
    match ty {
        Type::Path(path) if path.qself.is_none() && path.path.segments.len() == 2 => {
            let mut segments = path.path.segments.iter();
            let first = segments.next().expect("checked len");
            let second = segments.next().expect("checked len");
            if first.ident == "Self" && second.ident == "State" {
                *ty = parse_quote!(Self);
            }
        },
        Type::Path(path) => {
            for segment in &mut path.path.segments {
                if let PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for arg in &mut args.args {
                        if let GenericArgument::Type(ty) = arg {
                            rewrite_self_state_type(ty);
                        }
                    }
                }
            }
        },
        Type::Reference(reference) => rewrite_self_state_type(&mut reference.elem),
        Type::Paren(paren) => rewrite_self_state_type(&mut paren.elem),
        Type::Group(group) => rewrite_self_state_type(&mut group.elem),
        Type::Tuple(tuple) => {
            for elem in &mut tuple.elems {
                rewrite_self_state_type(elem);
            }
        },
        _ => {},
    }
}

pub fn attribute(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let attr: proc_macro2::TokenStream = attr.into();
    if !attr.is_empty() {
        return syn::Error::new_spanned(attr, "`component_value_binding` does not take options")
            .to_compile_error()
            .into();
    }

    let item = parse_macro_input!(item as ItemImpl);

    match expand(item) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
mod tests {
    use super::expand;
    use quote::quote;
    use syn::ItemImpl;

    fn compact_tokens(tokens: &str) -> String {
        tokens.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[test]
    fn rewrites_component_value_binding_to_state_binding() {
        let item: ItemImpl = syn::parse2(quote! {
            impl gpui_form_runtime::shape::ComponentValueBinding<Vec<PathBuf>> for FilePickerState {
                type Event = FilePickerEvent;

                fn form_value_change(
                    _state: &Self::State,
                    _event: &Self::Event,
                ) -> gpui_form_runtime::shape::FormValueChange<Vec<PathBuf>> {
                    gpui_form_runtime::shape::FormValueChange::Unchanged
                }
            }
        })
        .unwrap();

        let expanded = expand(item).unwrap();
        let compact = compact_tokens(&expanded.to_string());

        assert!(
            compact.contains("ComponentStateValueBinding<Vec<PathBuf>>forFilePickerState"),
            "attribute should rewrite the trait path: {compact}"
        );
        assert!(
            compact.contains("_state:&Self"),
            "attribute should rewrite state references to the state type: {compact}"
        );
    }
}
