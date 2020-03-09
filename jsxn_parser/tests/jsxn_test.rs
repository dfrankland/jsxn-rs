use jsxn_parser::{json, jsx, jsxn};
use nom::error::ErrorKind;
use std::collections::HashMap;

const VALID_JSX_ELEMENT: &str = r#"
    <Element prop="value" />
"#;

const VALID_JSON: &str = r#"
    { "key": "value" }
"#;

#[test]
fn parse_valid_jsxn() {
    assert_eq!(
        jsxn::root::<(&str, ErrorKind)>(VALID_JSX_ELEMENT),
        Ok((
            "",
            jsxn::JsxnValue::JsxValue(jsx::JsxValue::JsxElement((
                String::from("Element"),
                {
                    let mut props = HashMap::new();
                    props.insert(
                        String::from("prop"),
                        jsx::JsxValue::JsonValue(json::JsonValue::Str(String::from("value"))),
                    );
                    props
                },
                vec![],
            )))
        ))
    );
    assert_eq!(
        jsxn::root::<(&str, ErrorKind)>(VALID_JSON),
        Ok((
            "",
            jsxn::JsxnValue::JsonValue(json::JsonValue::Object({
                let mut object = HashMap::new();
                object.insert(
                    String::from("key"),
                    json::JsonValue::Str(String::from("value")),
                );
                object
            }))
        ))
    );
}
