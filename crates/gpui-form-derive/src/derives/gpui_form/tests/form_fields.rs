use super::support::*;

#[test]
fn generated_form_field_enum_contains_only_component_fields() {
    let derive_input: DeriveInput = syn::parse_quote! {
        #[derive(GpuiForm)]
        #[gpui_form(no_inventory)]
        struct UserProfile {
            #[gpui_form(component(crate::Input))]
            display_name: String,
            #[gpui_form(hidden)]
            internal_note: String,
            #[gpui_form(skip)]
            database_id: u64,
        }
    };

    let expanded = expansion::expand_gpui_form(
        derive_input,
        structs::GpuiFormOptions {
            generate_shape: false,
            generate_mcp: false,
        },
    );
    let file = syn::parse2::<syn::File>(expanded.clone()).expect("expansion should parse");
    let form_field = file
        .items
        .iter()
        .find_map(|item| match item {
            syn::Item::Enum(item) if item.ident == "UserProfileFormField" => Some(item),
            _ => None,
        })
        .expect("generated form-field enum");

    let variants = form_field
        .variants
        .iter()
        .map(|variant| variant.ident.to_string())
        .collect::<Vec<_>>();
    assert_eq!(variants, ["DisplayName"]);

    let compact = compact_tokens(&expanded.to_string());
    assert!(
        compact.contains("#[strum(to_string=\"display_name\")]DisplayName"),
        "form-field enum should retain the exact source field name: {compact}"
    );
    assert!(
        compact.contains("impl::gpui_form::core::FormFieldforUserProfileFormField"),
        "form-field enum should implement the facade contract: {compact}"
    );
    assert!(
        compact.contains("pubconstfnname(self)->&'staticstr{self.into_str()}"),
        "form-field enum should expose a const static name: {compact}"
    );
}

#[test]
fn test_structured_field_attribute_grammar() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(
                crate::DatePickerState,
                value(
                    type = chrono::NaiveDate,
                    from_source = |ts| to_form(ts),
                    into_source = |dt| to_model(dt)
                ),
                default = Timestamp::now()
            ))]
            birth_date: Timestamp,

            #[gpui_form(hidden(
                value(type = String, from_source = |id| id.to_string(), into_source = |id| id.parse().unwrap()),
                default = 1_u64
            ))]
            account_id: u64,
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
        compact.contains("RustType::from_macro_tokens_unchecked(\"chrono::NaiveDate\")")
            && compact.contains("pubaccount_id:String"),
        "structured value(type = ...) should drive component metadata and hidden holder storage: {compact}"
    );
    assert!(
        compact.contains("to_form") && compact.contains("to_model"),
        "structured from_source/into_source conversions should be emitted: {compact}"
    );
    assert!(
        compact.contains(".with_default(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"Timestamp::now()\"))")
            && compact.contains("::core::convert::Into::into(1_u64)"),
        "structured default options should be emitted for inventory and value-holder defaults: {compact}"
    );
    assert!(
        compact.contains("FieldVariant::hidden_for_field(\"account_id\",::gpui_form::schema::registry::FieldValueSpec::new(::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"String\"),::gpui_form::schema::registry::RustType::from_macro_tokens_unchecked(\"u64\"),::gpui_form::schema::registry::FieldValuePresence::DirectStorage)")
            && compact.contains("with_conversions(::gpui_form::schema::registry::ConversionMetadata::new(Some(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"|id|id.to_string()\")),Some(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"|id|id.parse().unwrap()\"))))")
            && compact.contains(".with_default(::gpui_form::schema::registry::RustExpr::from_macro_tokens_unchecked(\"1_u64\"))"),
        "hidden fields should be emitted into inventory with source/form types, conversions, direct storage, and defaults: {compact}"
    );
}

#[test]
fn test_gpui_form_rejects_duplicate_field_options() {
    let tokens = quote! {
        #[derive(GpuiForm)]
        struct TestForm {
            #[gpui_form(component(crate::Input, value(type = String, type = std::string::String)))]
            name: String,

            #[gpui_form(skip, skip)]
            hidden: bool,
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
        compact.contains("duplicate`type`option")
            && compact.contains("removetheduplicate`type`entry"),
        "duplicate type should produce an actionable error: {compact}"
    );
}
