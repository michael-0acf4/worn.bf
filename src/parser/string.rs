//!
//! (Parsing unicode-safe Strings is hard! :'()
//! Modified/less-cryptic version built based on:
//! https://raw.githubusercontent.com/rust-bakery/nom/refs/heads/main/examples/string.rs

//!
//! A string is:
//!
//! - Enclosed by double quotes
//! - Can contain any raw unescaped code point besides \ and "
//! - Matches the following escape sequences: \b, \f, \n, \r, \t, \", \\, \/
//! - Matches code points like Rust: \u{XXXX}, where XXXX can be up to 6
//!   hex characters
//! - an escape followed by whitespace consumes all whitespace between the
//!   escape and the next non-whitespace character
//!

use super::Span;
use nom::{
    IResult,
    branch::alt,
    bytes::{complete::is_not, complete::take_while_m_n},
    character::complete::{char, multispace1},
    combinator::{map, map_opt, map_res, value, verify},
    multi::fold_many0,
    sequence::{delimited, preceded},
};
use nom_locate::LocatedSpan;

fn parse_unicode(input: Span) -> IResult<Span, char> {
    let parse_hex = take_while_m_n(1, 6, |c: char| c.is_ascii_hexdigit());
    let parse_delimited_hex = preceded(char('u'), delimited(char('{'), parse_hex, char('}')));
    let parse_u32 = map_res(parse_delimited_hex, |hex: Span| {
        u32::from_str_radix(hex.fragment(), 16)
    });

    map_opt(parse_u32, char::from_u32)(input)
}

fn parse_escaped_char(input: Span) -> IResult<Span, char> {
    preceded(
        char('\\'),
        alt((
            parse_unicode,
            value('\n', char('n')),
            value('\r', char('r')),
            value('\t', char('t')),
            value('\u{08}', char('b')),
            value('\u{0C}', char('f')),
            value('\\', char('\\')),
            value('/', char('/')),
            value('"', char('"')),
        )),
    )(input)
}

fn parse_escaped_whitespace(input: Span) -> IResult<Span, LocatedSpan<&str>> {
    preceded(char('\\'), multispace1)(input)
}

fn parse_literal(input: Span) -> IResult<Span, String> {
    let (input, fragment) = verify(is_not("\"\\"), |s: &Span| !s.fragment().is_empty())(input)?;
    Ok((input, fragment.fragment().to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum StringFragment {
    Literal(String),
    EscapedChar(char),
    EscapedWS,
}

fn parse_fragment(input: Span) -> IResult<Span, StringFragment> {
    alt((
        map(parse_literal, StringFragment::Literal),
        map(parse_escaped_char, StringFragment::EscapedChar),
        value(StringFragment::EscapedWS, parse_escaped_whitespace),
    ))(input)
}

pub fn parse_string(input: Span) -> IResult<Span, String> {
    // old:
    // let build_string = fold(0.., parse_fragment, String::new,...
    let build_string = fold_many0(parse_fragment, String::new, |mut string, fragment| {
        match fragment {
            StringFragment::Literal(s) => string.push_str(&s),
            StringFragment::EscapedChar(c) => string.push(c),
            StringFragment::EscapedWS => {}
        }
        string
    });

    delimited(char('"'), build_string, char('"'))(input)
}
