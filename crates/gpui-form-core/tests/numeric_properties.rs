use gpui_form_core::numeric::{validate_signed_numeric, validate_unsigned_numeric};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(128))]

    #[test]
    fn canonical_integer_text_is_accepted(signed in any::<i64>(), unsigned in any::<u64>()) {
        prop_assert!(validate_signed_numeric::<i64>(&signed.to_string(), true));
        prop_assert!(validate_unsigned_numeric::<u64>(&unsigned.to_string(), true));
    }

    #[test]
    fn parse_required_checks_integer_range(signed in -1000_i64..1001, unsigned in 0_u64..1001) {
        let signed_text = signed.to_string();
        let unsigned_text = unsigned.to_string();
        // Mathematical representability is independent of the parsing code.
        let signed_fits = (i64::from(i8::MIN)..=i64::from(i8::MAX)).contains(&signed);
        let unsigned_fits = unsigned <= u64::from(u8::MAX);
        prop_assert_eq!(validate_signed_numeric::<i8>(&signed_text, true), signed_fits);
        prop_assert_eq!(validate_unsigned_numeric::<u8>(&unsigned_text, true), unsigned_fits);
        prop_assert!(validate_signed_numeric::<i8>(&signed_text, false));
        prop_assert!(validate_unsigned_numeric::<u8>(&unsigned_text, false));
    }

    #[test]
    fn oversized_decimal_text_is_editable_but_does_not_parse(
        digits in "[1-9][0-9]{20,63}",
    ) {
        // At least 21 decimal digits exceeds u64 and i64, even after shrinking.
        prop_assert!(validate_unsigned_numeric::<u64>(&digits, false));
        prop_assert!(!validate_unsigned_numeric::<u64>(&digits, true));
        prop_assert!(validate_signed_numeric::<i64>(&digits, false));
        prop_assert!(!validate_signed_numeric::<i64>(&digits, true));
        let negative = format!("-{digits}");
        prop_assert!(validate_signed_numeric::<i64>(&negative, false));
        prop_assert!(!validate_signed_numeric::<i64>(&negative, true));
    }

    #[test]
    fn non_ascii_insertions_are_rejected(
        value in any::<u64>(),
        inserted in prop::sample::select(vec!["é", "１", "١", "🧪"]),
        position in any::<u8>(),
    ) {
        let mut text = value.to_string();
        text.insert_str(usize::from(position) % (text.len() + 1), inserted);
        for require_parse in [false, true] {
            prop_assert!(!validate_signed_numeric::<i64>(&text, require_parse));
            prop_assert!(!validate_unsigned_numeric::<u64>(&text, require_parse));
        }
    }

    #[test]
    fn leading_zero_and_misplaced_sign_mutations_are_rejected(value in 1_u64..u64::MAX) {
        let digits = value.to_string();
        for text in [format!("0{digits}"), format!("-0{digits}"), format!("{digits}-"), format!("--{digits}")] {
            for require_parse in [false, true] {
                prop_assert!(!validate_signed_numeric::<i64>(&text, require_parse));
                prop_assert!(!validate_unsigned_numeric::<u64>(&text, require_parse));
            }
        }
    }
}

#[test]
fn parse_required_keeps_incomplete_input_and_range_boundaries_distinct() {
    // These are editing prefixes, not completed numbers, even with parse required.
    for text in ["", "-"] {
        assert!(validate_signed_numeric::<i8>(text, true));
    }
    for text in ["-128", "127"] {
        assert!(validate_signed_numeric::<i8>(text, true));
    }
    for text in ["-129", "128", "1..2", "-."] {
        assert!(validate_signed_numeric::<i8>(text, false));
        assert!(!validate_signed_numeric::<i8>(text, true));
    }
    assert!(validate_unsigned_numeric::<u8>("255", true));
    assert!(!validate_unsigned_numeric::<u8>("256", true));
}
