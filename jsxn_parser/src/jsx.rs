use crate::{
    json::{json_value, string, JsonValue},
    shared::sp,
};
use nom::{
    branch::alt,
    character::complete::{alphanumeric1 as alphanumeric, anychar, char},
    combinator::{cut, map, opt, peek, verify},
    error::{context, ParseError},
    multi::{fold_many0, many0, many_till},
    sequence::{delimited, pair, preceded, separated_pair, terminated, tuple},
    IResult,
};
use std::collections::HashMap;

/// A JSX-specific value
#[derive(Debug, PartialEq, Clone)]
pub enum JsxValue {
    /// A JSX Element containing an element type, props, and array of children
    JsxElement(JsxElement),

    /// A JSX Fragment which only contains an array of children
    JsxFragment(Vec<JsxValue>),

    /// A JSON value
    JsonValue(JsonValue),

    /// A JSX Expression containing a JSON value, a JSX Element, or a JSX
    /// Fragment
    JsxExpression(Box<JsxValue>),
}

/// JSX Element that correlates to the arguments for `React.createElement`.
/// (type, props, children)
pub type JsxElement = (String, HashMap<String, JsxValue>, Vec<JsxValue>);

fn jsx_text<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsxValue, E> {
    let (input_remainder, (chars, ..)) = context(
        "jsx text",
        verify(
            many_till(
                anychar,
                peek(preceded(
                    sp,
                    alt((
                        map(jsx_expression, |_| ()),
                        map(root, |_| ()),
                        map(jsx_element_closing_tag, |_| ()),
                        jsx_fragement_closing_tag,
                    )),
                )),
            ),
            |result| !result.0.is_empty(),
        ),
    )(i)?;
    Ok((
        input_remainder,
        JsxValue::JsonValue(JsonValue::Str(chars.into_iter().collect())),
    ))
}

fn jsx_expression<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsxValue, E> {
    context(
        "jsx expression",
        preceded(
            char('{'),
            cut(terminated(
                map(
                    alt((map(json_value, JsxValue::JsonValue), root)),
                    |jsx_value| JsxValue::JsxExpression(Box::new(jsx_value)),
                ),
                preceded(sp, char('}')),
            )),
        ),
    )(i)
}

fn jsx_children<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, Vec<JsxValue>, E> {
    context(
        "jsx children",
        many0(preceded(sp, alt((jsx_expression, root, jsx_text)))),
    )(i)
}

fn jsx_element_opening_tag<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, (String, HashMap<String, JsxValue>), E> {
    context(
        "jsx element opening tag",
        preceded(
            char('<'),
            pair(
                preceded(sp, map(alphanumeric, String::from)),
                preceded(
                    sp,
                    fold_many0(
                        alt((
                            separated_pair(
                                preceded(sp, map(alphanumeric, String::from)),
                                preceded(sp, char('=')),
                                preceded(
                                    sp,
                                    alt((
                                        jsx_expression,
                                        map(string, |s| JsxValue::JsonValue(JsonValue::Str(s))),
                                    )),
                                ),
                            ),
                            preceded(
                                sp,
                                map(alphanumeric, |prop| {
                                    (
                                        String::from(prop),
                                        JsxValue::JsonValue(JsonValue::Boolean(true)),
                                    )
                                }),
                            ),
                        )),
                        HashMap::new(),
                        |mut acc: HashMap<_, _>, (key, value)| {
                            acc.insert(key, value);
                            acc
                        },
                    ),
                ),
            ),
        ),
    )(i)
}

fn jsx_element_closing_tag<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, &'a str, E> {
    context(
        "jsx element closing tag",
        delimited(
            sp,
            delimited(
                preceded(char('<'), preceded(sp, char('/'))),
                preceded(sp, alphanumeric),
                preceded(sp, char('>')),
            ),
            opt(sp),
        ),
    )(i)
}

fn jsx_fragement_closing_tag<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, (), E> {
    context(
        "jsx fragment closing tag",
        delimited(
            sp,
            map(
                preceded(
                    char('<'),
                    preceded(opt(sp), preceded(char('/'), preceded(opt(sp), char('>')))),
                ),
                |_| (),
            ),
            opt(sp),
        ),
    )(i)
}

fn jsx_element_self_closing<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, JsxElement, E> {
    let (input_remainder, (tag, props)) = context(
        "jsx element self closing",
        terminated(
            jsx_element_opening_tag,
            preceded(sp, preceded(char('/'), preceded(sp, char('>')))),
        ),
    )(i)?;
    Ok((input_remainder, (tag, props, vec![])))
}

fn jsx_element_with_children<'a, E: ParseError<&'a str>>(
    i: &'a str,
) -> IResult<&'a str, JsxElement, E> {
    let (input_remainder, ((tag, props), children, ..)) = context(
        "jsx element with children",
        verify(
            tuple((
                terminated(jsx_element_opening_tag, preceded(sp, char('>'))),
                jsx_children,
                jsx_element_closing_tag,
            )),
            |((tag, ..), .., closing_tag)| tag == closing_tag,
        ),
    )(i)?;

    Ok((input_remainder, (tag, props, children)))
}

fn jsx_fragment<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsxValue, E> {
    context(
        "jsx fragment",
        map(
            delimited(
                preceded(char('<'), preceded(opt(sp), char('>'))),
                jsx_children,
                jsx_fragement_closing_tag,
            ),
            JsxValue::JsxFragment,
        ),
    )(i)
}

fn jsx_element<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsxValue, E> {
    context(
        "jsx element",
        map(
            alt((jsx_element_with_children, jsx_element_self_closing)),
            JsxValue::JsxElement,
        ),
    )(i)
}

/// The root JSX Element or JSX Fragment of a JSX tree
pub fn root<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsxValue, E> {
    delimited(sp, alt((jsx_element, jsx_fragment)), opt(sp))(i)
}
