use crate::{json, jsx, shared::sp};
use nom::{
    branch::alt,
    combinator::{map, opt},
    error::ParseError,
    sequence::delimited,
    IResult,
};
use serde::{Serialize, Serializer};

/// A JSXN value
#[derive(Debug, PartialEq, Clone)]
pub enum JsxnValue {
    /// A JSX value
    JsxValue(jsx::JsxValue),

    /// A JSON value
    JsonValue(json::JsonValue),
}

impl Serialize for JsxnValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            JsxnValue::JsxValue(json_value) => json_value.serialize(serializer),
            JsxnValue::JsonValue(jsx_value) => jsx_value.serialize(serializer),
        }
    }
}

/// The root JSX or JSON of a JSXN tree
pub fn root<'a, E: ParseError<&'a str>>(i: &'a str) -> IResult<&'a str, JsxnValue, E> {
    delimited(
        sp,
        alt((
            map(json::root, JsxnValue::JsonValue),
            map(jsx::root, JsxnValue::JsxValue),
        )),
        opt(sp),
    )(i)
}
