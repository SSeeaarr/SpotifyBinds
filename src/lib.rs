
pub mod hotkeyreg;


#[cfg(test)]
mod key_conversion_tests {
    use rdev::Key;
    use crate::hotkeyreg::str_to_key;

    #[test]
    fn test_str_to_key_valid() {
        assert_eq!(str_to_key("KeyA"), Some(Key::KeyA));
        assert_eq!(str_to_key("KeyB"), Some(Key::KeyB));
        assert_eq!(str_to_key("KeyC"), Some(Key::KeyC));
    }

    #[test]
    fn test_str_to_key_invalid() {
        assert_eq!(str_to_key("InvalidKey"), None);
        assert_eq!(str_to_key(""), None);
    }

    #[test]
    fn test_str_to_key_case_insensitive() {
        assert_eq!(str_to_key("keya"), Some(Key::KeyA));
        assert_eq!(str_to_key("KEYA"), Some(Key::KeyA));
    }
}
