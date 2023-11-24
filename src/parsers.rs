use oni_comb_parser_rs::prelude::*;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};

fn chars<'a>() -> Parser<'a, u8, String> {
  let special_char = elm_ref(b'\\')
    | elm_ref(b'/')
    | elm_ref(b'"')
    | elm_ref(b'b').map(|_| &b'\x08')
    | elm_ref(b'f').map(|_| &b'\x0C')
    | elm_ref(b'n').map(|_| &b'\n')
    | elm_ref(b'r').map(|_| &b'\r')
    | elm_ref(b't').map(|_| &b'\t');
  let escape_sequence = elm_ref(b'\\') * special_char;
  (none_ref_of(b"\\\"") | escape_sequence)
    .map(Clone::clone)
    .of_many1()
    .map_res(String::from_utf8)
}

fn utf16_chars<'a>() -> Parser<'a, u8, String> {
  let utf16_char = seq(b"\\u")
    * elm_hex_digit()
      .of_count(4)
      .map_res(String::from_utf8)
      .map_res(|digits| u16::from_str_radix(&digits, 16));
  utf16_char.of_many1().map(|chars| {
    decode_utf16(chars)
      .map(|r| r.unwrap_or(REPLACEMENT_CHARACTER))
      .collect::<String>()
  })
}

fn string<'a>() -> Parser<'a, u8, String> {
  let str = surround(elm_ref(b'"'), chars().of_many0(), elm_ref(b'"'));
  str.map(|strings| strings.concat())
}

fn utf16_string<'a>() -> Parser<'a, u8, String> {
  let str = surround(elm_ref(b'"'), (chars() | utf16_chars()).of_many0(), elm_ref(b'"'));
  str.map(|strings| strings.concat())
}

fn name<'a>() -> Parser<'a, u8, String> {
  string()
}

fn caption<'a>() -> Parser<'a, u8, String> {
  utf16_chars()
}
