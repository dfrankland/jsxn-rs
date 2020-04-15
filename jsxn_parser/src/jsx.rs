use crate::{
    json::{json_string, json_value, JsonValue},
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
use serde::{Serialize, Serializer};
use std::collections::BTreeMap;

/// A JSX-specific value
#[derive(Debug, PartialEq, Clone)]
pub enum JsxValue {
    /// A JSX Element containing an element type, props, and array of children
    JsxElement(JsxElement),

    /// A JSX Fragment which only contains an array of children
    JsxFragment(JsxFragment),

    /// A JSON value
    JsonValue(JsonValue),

    /// A JSX Expression containing a JSON value, a JSX Element, or a JSX
    /// Fragment
    JsxExpression(Box<JsxValue>),
}

impl Serialize for JsxValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            JsxValue::JsxElement(jsx_element) => jsx_element.serialize(serializer),
            JsxValue::JsxFragment(jsx_fragment) => jsx_fragment.serialize(serializer),
            JsxValue::JsonValue(json_value) => json_value.serialize(serializer),
            JsxValue::JsxExpression(jsx_expression) => jsx_expression.serialize(serializer),
        }
    }
}

/// JSX Element that correlates to the arguments for `React.createElement`.
/// (type, props, children)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JsxElement {
    r#type: String,
    props: BTreeMap<String, JsxValue>,
    children: Vec<JsxValue>,
}

impl JsxElement {
    /// Create a new JSX Element
    pub fn new(
        r#type: String,
        props: BTreeMap<String, JsxValue>,
        children: Vec<JsxValue>,
    ) -> JsxElement {
        JsxElement {
            r#type,
            props,
            children,
        }
    }
}

/// JSX Element that correlates to the arguments for `React.createElement`.
/// (type, props, children)
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct JsxFragment {
    r#type: String,
    children: Vec<JsxValue>,
}

impl JsxFragment {
    /// Create a new JSX Fragment
    pub fn new(children: Vec<JsxValue>) -> JsxFragment {
        JsxFragment {
            r#type: String::from("Fragment"),
            children,
        }
    }
}

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
                    alt((
                        map(json_value, |json| {
                            if let JsonValue::JsxValue(jsx) = json {
                                return *jsx;
                            }
                            JsxValue::JsonValue(json)
                        }),
                        root,
                    )),
                    |jsx| JsxValue::JsxExpression(Box::new(jsx)),
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
) -> IResult<&'a str, (String, BTreeMap<String, JsxValue>), E> {
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
                                        map(json_string, |s| {
                                            JsxValue::JsonValue(JsonValue::Str(s))
                                        }),
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
                        BTreeMap::new(),
                        |mut acc: BTreeMap<_, _>, (key, value)| {
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
    Ok((input_remainder, JsxElement::new(tag, props, vec![])))
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

    Ok((input_remainder, JsxElement::new(tag, props, children)))
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
            |x| JsxValue::JsxFragment(JsxFragment::new(x)),
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
