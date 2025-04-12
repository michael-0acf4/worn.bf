use super::ast::WithPos;
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::{line_ending, multispace1},
    combinator::{opt, rest},
    multi::many0,
    sequence::{delimited, preceded, terminated},
};
use nom_locate::LocatedSpan;

pub type Span<'a> = LocatedSpan<&'a str>;

pub fn with_position_mut<'a, O>(
    mut parser: impl FnMut(Span<'a>) -> IResult<Span<'a>, O>,
) -> impl FnMut(Span<'a>) -> IResult<Span<'a>, WithPos<O>> {
    move |input: Span<'a>| {
        let start = input.location_offset();
        let (next_input, value) = parser(input)?;
        let end = next_input.location_offset();
        Ok((next_input, WithPos { value, start, end }))
    }
}

pub fn inline_comment(input: Span) -> IResult<Span, LocatedSpan<&str>> {
    preceded(
        tag("//"),
        // must terminate on \n or eof
        terminated(alt((take_until("\n"), rest)), opt(line_ending)),
    )(input)
}

pub fn multiline_comment(input: Span) -> IResult<Span, LocatedSpan<&str>> {
    delimited(tag("/*"), take_until("*/"), tag("*/"))(input)
}

/// 0 or more `space`, `inline_comment` or `multiline_comment`
pub fn skippable0(input: Span) -> IResult<Span, Vec<LocatedSpan<&str>>> {
    let noop = alt((inline_comment, multiline_comment, multispace1));
    many0(noop)(input)
}
