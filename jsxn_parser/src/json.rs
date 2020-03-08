use crate::shared::sp;
use nom::{
    branch::alt,
    bytes::complete::{escaped, is_not, tag},
    character::complete::{char, hex_digit1, one_of},
    combinator::{cut, map, opt, value, verify},
    error::{context, ParseError},
    multi::separated_list,
    number::complete::double,
    sequence::{delimited, preceded, separated_pair, terminated},
    IResult,
};
use std::{collections::HashMap, str};

/// A JSON value
#[derive(Debug, PartialEq, Clone)]
pub enum JsonValue {
    /// A JSON string
    Str(String),

    /// A JSON boolean
    Boolean(bool),

    /// A JSON number
    Num(f64),

    /// A JSON array
    Array(Vec<JsonValue>),

    /// A JSON object
    Object(HashMap<String, JsonValue>),

    /// A JSON null value
    Null,
}

fn boolean<'a, E: ParseError<&'a str>>(input: &'a str) -> IResult<&'a str, bool, E> {
    let parse_true = value(true, tag("true"));
    let parse_false = value(false, tag("false"));
    alt((parse_true, parse_false))(input)
}

fn unicode_sequence<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context(
        "unicode sequence",
        verify(preceded(char('u'), hex_digit1), |hex_digits: &str| {
            hex_digits.len() == 4
        }),
    )(i)
}

pub(crate) fn string<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, String, E> {
    context(
        "string",
        map(
            delimited(
                char('\"'),
                escaped(
                    is_not("\""),
                    '\\',
                    alt((
                        map(one_of("\"\\/bfnrt"), |_| ()),
                        map(unicode_sequence, |_| ()),
                    )),
                ),
                char('\"'),
            ),
            String::from,
        ),
    )(i)
}

fn array<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<JsonValue>, E> {
    context(
        "array",
        preceded(
            char('['),
            cut(terminated(
                separated_list(preceded(sp, char(',')), json_value),
                preceded(sp, char(']')),
            )),
        ),
    )(i)
}

fn key_value<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (String, JsonValue), E> {
    separated_pair(
        preceded(sp, string),
        cut(preceded(sp, char(':'))),
        json_value,
    )(i)
}

fn hash<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, HashMap<String, JsonValue>, E> {
    context(
        "map",
        preceded(
            char('{'),
            cut(terminated(
                map(
                    separated_list(preceded(sp, char(',')), key_value),
                    |tuple_vec| tuple_vec.into_iter().collect(),
                ),
                preceded(sp, char('}')),
            )),
        ),
    )(i)
}

pub(crate) fn json_value<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsonValue, E> {
    preceded(
        sp,
        alt((
            map(hash, JsonValue::Object),
            map(array, JsonValue::Array),
            map(string, JsonValue::Str),
            map(double, JsonValue::Num),
            map(boolean, JsonValue::Boolean),
            map(tag("null"), |_| JsonValue::Null),
        )),
    )(i)
}

/// The root JSON object or array of a JSON tree
pub fn root<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsonValue, E> {
    delimited(
        sp,
        alt((map(hash, JsonValue::Object), map(array, JsonValue::Array))),
        opt(sp),
    )(i)
}
