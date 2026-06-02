#[test]
fn gpui_form_reports_invalid_field_attributes() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/gpui_form_direct_storage_*.rs");
    tests.compile_fail("tests/ui/gpui_form_duplicate_*.rs");
    tests.compile_fail("tests/ui/gpui_form_malformed_koruma_*.rs");
    tests.compile_fail("tests/ui/gpui_form_malformed_attribute_*.rs");
    tests.compile_fail("tests/ui/gpui_form_missing_*.rs");
    tests.compile_fail("tests/ui/gpui_form_skip_*.rs");
    tests.compile_fail("tests/ui/gpui_form_value_type_option.rs");
    tests.compile_fail("tests/ui/gpui_form_wrong_*.rs");
}

#[test]
fn gpui_form_accepts_generic_shape_backed_forms() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/gpui_form_generic_shape_pass.rs");
    tests.pass("tests/ui/gpui_form_plain_generic_pass.rs");
    tests.pass("tests/ui/gpui_form_partial_generic_component_pass.rs");
}

#[test]
fn gpui_form_accepts_required_shape_non_default_values() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/gpui_form_required_shape_non_default_pass.rs");
}

#[test]
fn gpui_form_accepts_value_override_passes() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/gpui_form_optional_value_override_pass.rs");
}

#[test]
fn gpui_form_rejects_required_shape_infallible_conversion() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/gpui_form_required_shape_into_original.rs");
}

#[test]
fn gpui_form_compiles_koruma_direct_attrs_end_to_end() {
    let tests = trybuild::TestCases::new();
    tests.pass("tests/ui/koruma_direct_attrs_pass.rs");
}
