#[cfg(feature = "derive")]
pub use gpui_form::GpuiForm;

fn main() {
    assert!(gpui_form::core::numeric::validate_signed_numeric::<i32>(
        "42", true
    ));
    #[cfg(feature = "runtime")]
    {
        let _ = core::any::type_name::<gpui_form::runtime::shape::DirectValueStorage>();
    }
    #[cfg(feature = "mcp")]
    {
        use gpui_form::mcp::McpJsonSchema as _;
        let _ = i32::json_schema();
        #[cfg(feature = "chrono")]
        let _ = chrono::NaiveDate::json_schema();
        #[cfg(feature = "rust_decimal")]
        let _ = rust_decimal::Decimal::json_schema();
    }
}

#[cfg(all(feature = "derive", feature = "runtime"))]
#[derive(GpuiForm)]
#[gpui_form(no_inventory)]
struct ConsumerForm {
    #[gpui_form(hidden)]
    value: String,
}
