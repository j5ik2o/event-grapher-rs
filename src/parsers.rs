use crate::ast::{Arrow, Ast, Line, Name};
use oni_comb_parser_rs::prelude::*;
use std::char::{decode_utf16, REPLACEMENT_CHARACTER};

fn space_with_crlf<'a>() -> Parser<'a, u8, ()> {
  elm_of(b" \t\r\n").of_many0().discard()
}

fn space<'a>() -> Parser<'a, u8, ()> {
  elm_of(b" \t").of_many0().discard()
}

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
  (none_ref_of(b"\\\":-\n") | escape_sequence)
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
  let str2 = chars().of_many0();
  (str2).map(|strings| strings.concat())
}

fn utf16_string<'a>() -> Parser<'a, u8, String> {
  let str = surround(
    elm_ref(b'"'),
    (chars().attempt() | utf16_chars()).of_many0(),
    elm_ref(b'"'),
  );
  str.map(|strings| strings.concat())
}

fn name<'a>() -> Parser<'a, u8, String> {
  space() * string() - space()
}

fn caption_string<'a>() -> Parser<'a, u8, String> {
  space() * utf16_string() - space()
}

fn caption<'a>() -> Parser<'a, u8, String> {
  space() * elm_ref(b':') * caption_string()
}

fn element_parser<'a, F, A>(l: u8, f: F) -> Parser<'a, u8, A>
where
  F: Fn(String, Option<String>) -> A + 'a,
  A: Clone + 'a,
{
  let lp = elm_ref(l) + elm_ref(b':');
  let p = space_with_crlf() * lp * name().debug("name") + caption().debug("caption").opt() - space_with_crlf();
  p.map(move |(n, c)| f(n, c))
}

fn title<'a>() -> Parser<'a, u8, Ast> {
  element_parser(b't', |n, c| Ast::TitleDef(Name::of_title(n, c)))
}

fn user<'a>() -> Parser<'a, u8, Ast> {
  element_parser(b'u', |n, c| Ast::NameDef(Name::of_user(n, c)))
}

fn command<'a>() -> Parser<'a, u8, Ast> {
  element_parser(b'c', |n, c| Ast::NameDef(Name::of_command(n, c)))
}

fn event<'a>() -> Parser<'a, u8, Ast> {
  element_parser(b'e', |n, c| Ast::NameDef(Name::of_event(n, c)))
}

fn aggregate<'a>() -> Parser<'a, u8, Ast> {
  element_parser(b'a', |n, c| Ast::NameDef(Name::of_aggregate(n, c)))
}

fn policy<'a>() -> Parser<'a, u8, Ast> {
  element_parser(b'p', |n, c| Ast::NameDef(Name::of_policy(n, c)))
}

fn read_model<'a>() -> Parser<'a, u8, Ast> {
  element_parser(b'r', |n, c| Ast::NameDef(Name::of_read_model(n, c)))
}

fn element<'a>() -> Parser<'a, u8, Ast> {
  space() * (title() | user() | command() | event() | aggregate() | policy() | read_model()) - space()
}

fn relation_ship_parser<'a, F, A>(b: u8, f: F) -> Parser<'a, u8, A>
where
  F: Fn(String, String, Option<String>) -> A + 'a,
  A: Clone + 'a,
{
  let p =
    space_with_crlf() * name().debug("from") + (elm_ref(b'-') + elm_ref(b)) * name().debug("to") + caption().opt()
      - space_with_crlf();
  p.map(move |((from, to), c)| f(from, to, c))
}

fn arrow<'a>() -> Parser<'a, u8, Ast> {
  relation_ship_parser(b'>', |from, to, c| Ast::Arrow(Arrow::new(from, to, c)))
}

fn line<'a>() -> Parser<'a, u8, Ast> {
  relation_ship_parser(b'-', |from, to, c| Ast::Line(Line::new(from, to, c)))
}

fn relation_ship<'a>() -> Parser<'a, u8, Ast> {
  space() * (line().attempt() | arrow()) - space()
}

fn document<'a>() -> Parser<'a, u8, Ast> {
  space_with_crlf() * (element().attempt() | relation_ship()) - space_with_crlf()
}

pub fn documents<'a>() -> Parser<'a, u8, Ast> {
  document().of_many0().map(|elements| Ast::Documents(elements))
}

pub fn parse<'a>(input: &'a [u8]) -> Result<Ast, ParseError<'a, u8>> {
  documents().parse(input).to_result()
}

#[cfg(test)]
pub mod tests {
  use super::*;
  use crate::dot_writer::Title;
  use std::env;

  pub fn test_parser<'a, A>(parser: Parser<'a, u8, A>, input: &'a [u8], expected: A)
  where
    A: Clone + std::fmt::Debug + PartialEq,
  {
    let result = parser.parse(input).to_result();
    assert_eq!(result, Ok(expected));
  }

  #[test]
  pub fn test_chars() {
    test_parser(chars(), b"abc", "abc".to_string());
  }

  #[test]
  pub fn test_utf16_chars() {
    test_parser(utf16_chars(), b"\\u0041\\u0042\\u0043", "ABC".to_string());
  }

  #[test]
  pub fn test_string() {
    test_parser(string(), b"abc", "abc".to_string());
  }

  #[test]
  pub fn test_utf16_string() {
    test_parser(utf16_string(), "\"ユーザ\"".as_bytes(), "ユーザ".to_string());
  }

  #[test]
  pub fn test_name() {
    test_parser(name(), b"abc", "abc".to_string());
  }

  #[test]
  pub fn test_caption_string() {
    test_parser(caption_string(), "\"ユーザ\"".as_bytes(), "ユーザ".to_string());
  }

  #[test]
  pub fn test_caption() {
    test_parser(caption(), ":\"ユーザ\"".as_bytes(), "ユーザ".to_string());
  }

  #[test]
  pub fn test_user() {
    test_parser(
      user(),
      r#"u:abc"#.as_bytes(),
      Ast::NameDef(Name::of_user("abc".to_string(), None)),
    );
  }

  #[test]
  pub fn test_user_without_double_quote() {
    test_parser(
      user(),
      r#"u:abc"#.as_bytes(),
      Ast::NameDef(Name::of_user("abc".to_string(), None)),
    );
  }

  #[test]
  pub fn test_user_without_double_quote_with_caption() {
    test_parser(
      user(),
      r#"u:abc:"ユーザ""#.as_bytes(),
      Ast::NameDef(Name::of_user("abc".to_string(), Some("ユーザ".to_string()))),
    );
  }

  #[test]
  pub fn test_user_with_caption() {
    test_parser(
      user(),
      r#"u:abc:"ユーザ""#.as_bytes(),
      Ast::NameDef(Name::of_user("abc".to_string(), Some("ユーザ".to_string()))),
    );
  }

  #[test]
  pub fn test_command() {
    test_parser(
      command(),
      "c:abc".as_bytes(),
      Ast::NameDef(Name::of_command("abc".to_string(), None)),
    );
  }

  #[test]
  pub fn test_command_with_caption() {
    test_parser(
      command(),
      r#"c:abc:"ユーザ""#.as_bytes(),
      Ast::NameDef(Name::of_command("abc".to_string(), Some("ユーザ".to_string()))),
    );
  }

  #[test]
  pub fn test_event() {
    test_parser(
      event(),
      "e:abc".as_bytes(),
      Ast::NameDef(Name::of_event("abc".to_string(), None)),
    );
  }

  #[test]
  pub fn test_event_with_caption() {
    test_parser(
      event(),
      r#"e:abc:"ユーザ""#.as_bytes(),
      Ast::NameDef(Name::of_event("abc".to_string(), Some("ユーザ".to_string()))),
    );
  }

  #[test]
  pub fn test_aggregate() {
    test_parser(
      aggregate(),
      "a:abc".as_bytes(),
      Ast::NameDef(Name::of_aggregate("abc".to_string(), None)),
    );
  }

  #[test]
  pub fn test_aggregate_with_caption() {
    test_parser(
      aggregate(),
      r#"a:abc:"ユーザ""#.as_bytes(),
      Ast::NameDef(Name::of_aggregate("abc".to_string(), Some("ユーザ".to_string()))),
    );
  }

  #[test]
  pub fn test_policy() {
    test_parser(
      policy(),
      r#"p:abc"#.as_bytes(),
      Ast::NameDef(Name::of_policy("abc".to_string(), None)),
    );
  }

  #[test]
  pub fn test_policy_with_caption() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    test_parser(
      policy(),
      r#"p:abc:"ユーザ""#.as_bytes(),
      Ast::NameDef(Name::of_policy("abc".to_string(), Some("ユーザ".to_string()))),
    );
  }

  #[test]
  pub fn test_read_model() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    test_parser(
      read_model(),
      "r:abc".as_bytes(),
      Ast::NameDef(Name::of_read_model("abc".to_string(), None)),
    );
  }

  #[test]
  pub fn test_read_model_with_caption() {
    env::set_var("RUST_LOG", "debug");
    env_logger::init();
    test_parser(
      read_model(),
      r#"r:abc:"ユーザ""#.as_bytes(),
      Ast::NameDef(Name::of_read_model("abc".to_string(), Some("ユーザ".to_string()))),
    );
  }

  #[test]
  pub fn test_arrow() {
    test_parser(
      arrow(),
      r#"abc->def"#.as_bytes(),
      Ast::Arrow(Arrow::new("abc".to_string(), "def".to_string(), None)),
    );
  }

  #[test]
  pub fn test_arrow_with_caption() {
    test_parser(
      arrow(),
      r#"abc->def:"ユーザ""#.as_bytes(),
      Ast::Arrow(Arrow::new(
        "abc".to_string(),
        "def".to_string(),
        Some("ユーザ".to_string()),
      )),
    );
  }

  #[test]
  pub fn test_line() {
    test_parser(
      line(),
      r#"abc--def"#.as_bytes(),
      Ast::Line(Line::new("abc".to_string(), "def".to_string(), None)),
    );
  }

  #[test]
  pub fn test_line_with_caption() {
    test_parser(
      line(),
      r#"abc--def:"ユーザ""#.as_bytes(),
      Ast::Line(Line::new(
        "abc".to_string(),
        "def".to_string(),
        Some("ユーザ".to_string()),
      )),
    );
  }

  #[test]
  pub fn test_documents() {
    test_parser(
      documents(),
      r#"
        t:G:"title"
        u:abc:"ユーザ"
        c:abc:"ユーザ"
        e:abc:"ユーザ"
        a:abc:"ユーザ"
        p:abc:"ユーザ"
        r:abc:"ユーザ"
        abc->def:"ユーザ"
        abc->def:"ユーザ"
        "#
      .as_bytes(),
      Ast::Documents(vec![
        Ast::TitleDef(Name::of_title("G".to_string(), Some("title".to_string()))),
        Ast::NameDef(Name::of_user("abc".to_string(), Some("ユーザ".to_string()))),
        Ast::NameDef(Name::of_command("abc".to_string(), Some("ユーザ".to_string()))),
        Ast::NameDef(Name::of_event("abc".to_string(), Some("ユーザ".to_string()))),
        Ast::NameDef(Name::of_aggregate("abc".to_string(), Some("ユーザ".to_string()))),
        Ast::NameDef(Name::of_policy("abc".to_string(), Some("ユーザ".to_string()))),
        Ast::NameDef(Name::of_read_model("abc".to_string(), Some("ユーザ".to_string()))),
        Ast::Arrow(Arrow::new(
          "abc".to_string(),
          "def".to_string(),
          Some("ユーザ".to_string()),
        )),
        Ast::Arrow(Arrow::new(
          "abc".to_string(),
          "def".to_string(),
          Some("ユーザ".to_string()),
        )),
      ]),
    );
  }
}
