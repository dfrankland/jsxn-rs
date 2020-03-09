use crate::{jsx::{root as jsx_value, JsxValue}, shared::sp};
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

    /// A JSX value
    JsxValue(Box<JsxValue>),
}

fn json_boolean<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, bool, E> {
    let parse_true = value(true, tag("true"));
    let parse_false = value(false, tag("false"));
    context("json boolean", alt((parse_true, parse_false)))(i)
}

fn json_unicode_sequence<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context(
        "json unicode sequence",
        verify(preceded(char('u'), hex_digit1), |hex_digits: &str| {
            hex_digits.len() == 4
        }),
    )(i)
}

pub(crate) fn json_string<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, String, E> {
    context(
        "json string",
        map(
            delimited(
                char('\"'),
                escaped(
                    is_not("\""),
                    '\\',
                    alt((
                        map(one_of("\"\\/bfnrt"), |_| ()),
                        map(json_unicode_sequence, |_| ()),
                    )),
                ),
                char('\"'),
            ),
            String::from,
        ),
    )(i)
}

fn json_array<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<JsonValue>, E> {
    context(
        "json array",
        preceded(
            char('['),
            cut(terminated(
                separated_list(preceded(sp, char(',')), json_value),
                preceded(sp, char(']')),
            )),
        ),
    )(i)
}

fn json_key_value<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (String, JsonValue), E> {
    context("json key value", separated_pair(
        preceded(sp, json_string),
        cut(preceded(sp, char(':'))),
        json_value,
    ))(i)
}

fn json_object<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, HashMap<String, JsonValue>, E> {
    context(
        "json object",
        preceded(
            char('{'),
            cut(terminated(
                map(
                    separated_list(preceded(sp, char(',')), json_key_value),
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
            map(json_object, JsonValue::Object),
            map(json_array, JsonValue::Array),
            map(json_string, JsonValue::Str),
            map(double, JsonValue::Num),
            map(json_boolean, JsonValue::Boolean),
            map(tag("null"), |_| JsonValue::Null),
            map(jsx_value, |jsx| {
                JsonValue::JsxValue(Box::new(jsx))
            }),
        )),
    )(i)
}

/// The root JSON object or array of a JSON tree
pub fn root<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsonValue, E> {
    delimited(
        sp,
        alt((map(json_object, JsonValue::Object), map(json_array, JsonValue::Array))),
        opt(sp),
    )(i)
}
