use jsxn::{json, jsx};
use nom::{
    error::{convert_error, ErrorKind, VerboseError},
    Err,
};
use std::collections::BTreeMap;

const VALID_JSON: &str = r#"
    {
        "a": 42,
        "b": ["this is a \"string\"", "this is too 👍 \u2605", 12],
        "c": { "hello" : "world", "empty string":"" },
        "d": null,
        "e": <Element prop="value" />
    }
"#;

const INVALID_JSON: &str = r#"
    {
        "a": 42,
        "b": ["x", "y", 12],
        "c": { 1"hello" : "world" }
    }
"#;

#[test]
fn parse_valid_json() {
    assert_eq!(
        json::root::<(&str, ErrorKind)>(VALID_JSON),
        Ok((
            "",
            json::JsonValue::Object({
                let mut object = BTreeMap::new();
                object.insert(String::from("a"), json::JsonValue::Num(42.0));
                object.insert(
                    String::from("b"),
                    json::JsonValue::Array({
                        let mut array = vec![];
                        array.push(json::JsonValue::Str(String::from(
                            "this is a \\\"string\\\"",
                        )));
                        array.push(json::JsonValue::Str(String::from("this is too 👍 \\u2605")));
                        array.push(json::JsonValue::Num(12.0));
                        array
                    }),
                );
                object.insert(
                    String::from("c"),
                    json::JsonValue::Object({
                        let mut object = BTreeMap::new();
                        object.insert(
                            String::from("hello"),
                            json::JsonValue::Str(String::from("world")),
                        );
                        object.insert(
                            String::from("empty string"),
                            json::JsonValue::Str(String::from("")),
                        );
                        object
                    }),
                );
                object.insert(String::from("d"), json::JsonValue::Null);
                object.insert(
                    String::from("e"),
                    json::JsonValue::JsxValue(Box::new(jsx::JsxValue::JsxElement(
                        jsx::JsxElement::new(
                            String::from("Element"),
                            {
                                let mut props = BTreeMap::new();
                                props.insert(
                                    String::from("prop"),
                                    jsx::JsxValue::JsonValue(json::JsonValue::Str(String::from(
                                        "value",
                                    ))),
                                );
                                props
                            },
                            vec![],
                        ),
                    ))),
                );
                object
            })
        ))
    )
}

#[test]
fn parse_invalid_json() {
    assert_eq!(
        json::root::<(&str, ErrorKind)>(INVALID_JSON),
        Err(Err::Failure((
            "1\"hello\" : \"world\" }\n    }\n",
            ErrorKind::Char,
        )))
    )
}

#[test]
fn parse_invalid_json_verbose_trace() {
    if let Err(Err::Error(e)) | Err(Err::Failure(e)) =
        json::root::<VerboseError<&str>>(INVALID_JSON)
    {
        assert_eq!(
            convert_error(INVALID_JSON, e),
            r#"0: at line 5:
        "c": { 1"hello" : "world" }
               ^
expected '}', found 1

1: at line 5, in json object:
        "c": { 1"hello" : "world" }
             ^

2: at line 4, in json key value:
        "b": ["x", "y", 12],
                            ^

3: at line 2, in json object:
    {
    ^

"#
        )
    }
}

#[test]
fn serialize_json() {
    assert_eq!(
        serde_json::to_string_pretty(&json::root::<(&str, ErrorKind)>(VALID_JSON).unwrap().1)
            .unwrap(),
        String::from(
            r#"{
  "a": 42.0,
  "b": [
    "this is a \\\"string\\\"",
    "this is too 👍 \\u2605",
    12.0
  ],
  "c": {
    "empty string": "",
    "hello": "world"
  },
  "d": null,
  "e": {
    "type": "Element",
    "props": {
      "prop": "value"
    },
    "children": []
  }
}"#
        )
    )
}
