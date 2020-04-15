use ::jsxn::{json, jsx, jsxn};
use nom::error::ErrorKind;
use std::collections::BTreeMap;

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
            jsxn::JsxnValue::JsxValue(jsx::JsxValue::JsxElement(jsx::JsxElement::new(
                String::from("Element"),
                {
                    let mut props = BTreeMap::new();
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
                let mut object = BTreeMap::new();
                object.insert(
                    String::from("key"),
                    json::JsonValue::Str(String::from("value")),
                );
                object
            }))
        ))
    );
}

#[test]
fn serialize_jsxn() {
    assert_eq!(
        serde_json::to_string_pretty(
            &jsxn::root::<(&str, ErrorKind)>(&format!("[{},{}]", VALID_JSX_ELEMENT, VALID_JSON))
                .unwrap()
                .1
        )
        .unwrap(),
        String::from(
            r#"[
  {
    "type": "Element",
    "props": {
      "prop": "value"
    },
    "children": []
  },
  {
    "key": "value"
  }
]"#
        )
    )
}
