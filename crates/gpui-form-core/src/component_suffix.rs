/// Returns whether a component/prototyping suffix is a non-empty ASCII
/// identifier suffix.
pub const fn is_valid_ascii_identifier_suffix(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.is_empty() || (bytes.len() == 1 && bytes[0] == b'_') {
        return false;
    }
    if !is_ascii_ident_start(bytes[0]) {
        return false;
    }

    let mut idx = 1;
    while idx < bytes.len() {
        if !is_ascii_ident_continue(bytes[idx]) {
            return false;
        }
        idx += 1;
    }

    true
}

const fn is_ascii_ident_start(byte: u8) -> bool {
    byte == b'_' || byte >= b'a' && byte <= b'z' || byte >= b'A' && byte <= b'Z'
}

const fn is_ascii_ident_continue(byte: u8) -> bool {
    is_ascii_ident_start(byte) || byte >= b'0' && byte <= b'9'
}
