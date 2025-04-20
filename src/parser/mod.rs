pub mod ast;
pub mod shared;
pub mod string;

use ast::{Instruction, SuperValue, WithPos};
use nom::{
    IResult,
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::char,
    combinator::map,
    multi::{many0, many1, separated_list0},
    sequence::{delimited, preceded},
};
use shared::{Span, skippable0, with_position_mut};
use string::parse_string;

fn parse_token(input: Span) -> IResult<Span, WithPos<String>> {
    map(
        with_position_mut(take_while1(|c: char| c.is_alphanumeric() || c == '_')),
        |s| s.transfer(s.value.to_string()),
    )(input)
}

fn parse_string_value(input: Span) -> IResult<Span, WithPos<SuperValue>> {
    map(with_position_mut(parse_string), |s| {
        s.transfer(SuperValue::String(s.value.clone()))
    })(input)
}

fn parse_literal_value(input: Span) -> IResult<Span, WithPos<SuperValue>> {
    map(parse_token, |s| {
        s.transfer(SuperValue::Literal(s.value.clone()))
    })(input)
}

fn parse_number_value(input: Span) -> IResult<Span, WithPos<SuperValue>> {
    map(
        with_position_mut(take_while1(|c: char| c.is_digit(10))),
        |s| {
            s.transfer(SuperValue::Integer(
                s.value.clone().to_owned().parse::<u32>().unwrap(),
            ))
        },
    )(input)
}

fn parse_value(input: Span) -> IResult<Span, WithPos<SuperValue>> {
    alt((
        parse_string_value,
        parse_super_call,
        parse_number_value,
        parse_literal_value,
    ))(input)
}

fn parse_super_call(input: Span) -> IResult<Span, WithPos<SuperValue>> {
    let start = input.location_offset();
    let (next_input, callee) = preceded(skippable0, parse_token)(input)?;
    let sep = delimited(skippable0, char(','), skippable0);
    let (next_input, args) = delimited(
        preceded(skippable0, char('(')),
        separated_list0(sep, preceded(skippable0, parse_instr)),
        preceded(skippable0, char(')')),
    )(next_input)?;

    Ok((
        next_input,
        WithPos {
            start,
            end: next_input.location_offset(),
            value: SuperValue::SuperCall { callee, args },
        },
    ))
}

fn parse_inline_value(input: Span) -> IResult<Span, WithPos<Instruction>> {
    map(parse_value, |v| {
        v.transfer(Instruction::InlineValue(v.value.clone()))
    })(input)
}

fn parse_add(input: Span) -> IResult<Span, WithPos<Instruction>> {
    map(
        with_position_mut(alt((many1(char('+')), many1(char('-'))))),
        |s| {
            s.transfer(Instruction::Add(
                s.value.len() as i32 * (if s.value[0] == '+' { 1 } else { -1 }),
            ))
        },
    )(input)
}

fn parse_loop(input: Span) -> IResult<Span, WithPos<Instruction>> {
    let body = delimited(
        preceded(skippable0, char('[')),
        many0(parse_instr),
        preceded(skippable0, char(']')),
    );

    map(with_position_mut(body), |s| {
        s.transfer(Instruction::Loop {
            body: s.value.clone(),
        })
    })(input)
}

fn parse_io(input: Span) -> IResult<Span, WithPos<Instruction>> {
    map(
        with_position_mut(alt((many1(char('.')), many1(char(','))))),
        |s| {
            let instr = if s.value[0] == '.' {
                Instruction::Put(s.value.len() as u32)
            } else {
                Instruction::Get(s.value.len() as u32)
            };

            s.transfer(instr)
        },
    )(input)
}

fn parse_move(input: Span) -> IResult<Span, WithPos<Instruction>> {
    map(
        with_position_mut(alt((many1(char('>')), many1(char('<'))))),
        |s| {
            s.transfer(Instruction::Move(
                s.value.len() as i32 * (if s.value[0] == '>' { 1 } else { -1 }),
            ))
        },
    )(input)
}

fn parse_super(input: Span) -> IResult<Span, WithPos<Instruction>> {
    let start = input.location_offset();
    let (next_input, _) = preceded(skippable0, tag("super"))(input)?;
    let (next_input, name) = preceded(skippable0, parse_token)(next_input)?;

    let sep = delimited(skippable0, char(','), skippable0);
    let (next_input, args) = delimited(
        preceded(skippable0, char('(')),
        separated_list0(sep, preceded(skippable0, parse_token)),
        preceded(skippable0, char(')')),
    )(next_input)?;

    let (next_input, body) = delimited(
        preceded(skippable0, char('{')),
        many1(parse_instr),
        preceded(skippable0, char('}')),
    )(next_input)?;

    Ok((
        next_input,
        WithPos {
            start,
            end: next_input.location_offset(),
            value: Instruction::SuperFunction { name, args, body },
        },
    ))
}

fn parse_instr(input: Span) -> IResult<Span, WithPos<Instruction>> {
    preceded(
        skippable0,
        alt((
            parse_add,
            parse_move,
            parse_io,
            parse_loop,
            parse_super,
            parse_inline_value,
        )),
    )(input)
}

pub fn parse_program(input: &str) -> Result<Vec<WithPos<Instruction>>, String> {
    let (_, instructions) = many0(parse_instr)(input.into()).map_err(|e| e.to_string())?;
    Ok(instructions)
}

#[test]
pub fn test_units() {
    // greedy test with take_till1
    assert_eq!(
        parse_token("a ".into()).map(|v| (v.0.to_string(), v.1.value)),
        Ok((" ".to_string(), "a".to_string()))
    );
    assert_eq!(
        parse_token("abc cd".into()).map(|v| (v.0.to_string(), v.1.value)),
        Ok((" cd".to_string(), "abc".to_string()))
    );
}
