use crate::Result;
use std::borrow::Cow;

pub fn encode_param(s: impl AsRef<str>) -> String {
    // TODO: it would be nice not to clone the string when there are no encoding chars.
    let encoded: String = s
        .as_ref()
        .as_bytes()
        .iter()
        .flat_map(|ch| {
            if ch.is_ascii_alphanumeric()
                || *ch as char == '-'
                || *ch as char == '.'
                || *ch as char == '_'
                || *ch as char == '~'
            {
                vec![*ch]
            } else {
                format!("%{ch:X}").as_bytes().to_owned()
            }
        })
        .map(|v| v as char)
        .collect();
    encoded
}

pub fn decode_str(s: impl AsRef<str>) -> String {
    let mut bytes: Vec<u8> = vec![];
    let mut input_bytes = s.as_ref().as_bytes();
    while !input_bytes.is_empty() {
        // unwrap: tail isn't empty, so there will always be a first byte
        let byte = input_bytes.first().unwrap();
        if *byte != b'%' {
            bytes.push(*byte);
            input_bytes = &input_bytes[1..];
        } else {
            let nibble1 = hex_nibble(input_bytes[1] as char);
            let nibble2 = hex_nibble(input_bytes[2] as char);
            bytes.push(nibble1 * 16 + nibble2);
            input_bytes = &input_bytes[3..];
        }
    }
    // TODO: get rid of this unwrap
    String::from_utf8(bytes).unwrap()
}

fn hex_nibble(ch: char) -> u8 {
    // TODO: include a text for is_hex_digit
    if ch.is_ascii_digit() {
        ch as u8 - b'0'
    } else if ch.is_ascii_uppercase() {
        (ch as u8 - b'A') + 10
    } else if ch.is_ascii_lowercase() {
        (ch as u8 - b'a') + 10
    } else {
        panic!()
    }
}

#[cfg(test)]
mod test {
    use crate::pencoding::{encode_param, hex_nibble};

    #[test]
    fn no_escapes() {
        assert_eq!("foobar", encode_param("foobar"));
        assert_eq!("foobar_baz.quux", encode_param("foobar_baz.quux"));
        assert_eq!("foobar-baz~quux", encode_param("foobar-baz~quux"));
    }

    #[test]
    fn simple_escapes() {
        assert_eq!("foobar%21", encode_param("foobar!"));
        // Note that hex is upper-case.
        assert_eq!("Bugs%20Bunny%3F", encode_param("Bugs Bunny?"));
        assert_eq!("%22%2422.51%22", encode_param("\"$22.51\""))
    }

    #[test]
    fn unicode_escapes() {
        assert_eq!("%C2%A1%C2%A2%E2%89%A0%C3%A6", encode_param("¡¢≠æ"));
    }

    #[test]
    fn hex_nibble_test() {
        assert_eq!(0, hex_nibble('0'));
        assert_eq!(5, hex_nibble('5'));
        assert_eq!(9, hex_nibble('9'));
        assert_eq!(10, hex_nibble('a'));
        assert_eq!(12, hex_nibble('c'));
        assert_eq!(15, hex_nibble('f'));
        assert_eq!(10, hex_nibble('A'));
        assert_eq!(12, hex_nibble('C'));
        assert_eq!(15, hex_nibble('F'));
    }
}
