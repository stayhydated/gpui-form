#![deny(warnings, unfulfilled_lint_expectations)]

use gpui_form::GpuiForm;

#[derive(Debug, Eq, GpuiForm, PartialEq)]
#[gpui_form(no_inventory)]
struct SixSkipped {
    #[gpui_form(hidden)]
    value: String,
    #[gpui_form(skip)]
    first: u64,
    #[gpui_form(skip)]
    second: u64,
    #[gpui_form(skip)]
    third: u64,
    #[gpui_form(skip)]
    fourth: u64,
    #[gpui_form(skip)]
    fifth: u64,
    #[gpui_form(skip)]
    sixth: u64,
}

#[derive(Debug, Eq, GpuiForm, PartialEq)]
#[gpui_form(no_inventory)]
struct SevenSkipped {
    #[gpui_form(hidden)]
    value: String,
    #[gpui_form(skip)]
    first: u64,
    #[gpui_form(skip)]
    second: u64,
    #[gpui_form(skip)]
    third: u64,
    #[gpui_form(skip)]
    fourth: u64,
    #[gpui_form(skip)]
    fifth: u64,
    #[gpui_form(skip)]
    sixth: u64,
    #[gpui_form(skip)]
    seventh: u64,
}

fn main() {
    let six = SixSkippedFormValueHolder {
        value: "edited".into(),
    }
    .into_original(1, 2, 3, 4, 5, 6)
    .unwrap();
    assert_eq!(
        six,
        SixSkipped {
            value: "edited".into(),
            first: 1,
            second: 2,
            third: 3,
            fourth: 4,
            fifth: 5,
            sixth: 6
        }
    );
    let seven = SevenSkippedFormValueHolder {
        value: "edited".into(),
    }
    .into_original(1, 2, 3, 4, 5, 6, 7)
    .unwrap();
    assert_eq!(
        seven,
        SevenSkipped {
            value: "edited".into(),
            first: 1,
            second: 2,
            third: 3,
            fourth: 4,
            fifth: 5,
            sixth: 6,
            seventh: 7
        }
    );
}
