use super::support::*;

#[test]
fn test_mcp_attribute_requires_mcp_feature() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(mcp)]
        struct TestForm {
            #[gpui_form(hidden)]
            value: String,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: false,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("requiresthe`gpui-form/mcp`feature"),
        "mcp attribute should require the mcp feature: {compact}"
    );
}

#[test]
fn test_mcp_attribute_rejects_no_inventory() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(no_inventory, mcp)]
        struct TestForm {
            #[gpui_form(hidden)]
            value: String,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: true,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("cannotbecombinedwith") && compact.contains("gpui_form(no_inventory)"),
        "mcp attribute should reject no_inventory: {compact}"
    );
}

#[test]
fn test_mcp_attribute_rejects_generic_forms() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        #[gpui_form(mcp)]
        struct TestForm<T> {
            #[gpui_form(hidden)]
            value: T,
        }
    };

    let derive_input: DeriveInput = syn::parse2(tokens).unwrap();
    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: true,
            generate_mcp: true,
        },
    );

    let compact = compact_tokens(&expanded.to_string());

    assert!(
        compact.contains("doesnotsupportgenericforms"),
        "mcp attribute should reject generic forms: {compact}"
    );
}
