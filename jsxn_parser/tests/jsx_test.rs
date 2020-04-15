use jsxn::{json, jsx};
use nom::error::ErrorKind;
use std::collections::BTreeMap;

const VALID_JSX_ELEMENT: &str = r#"
    <Hello friend="World" count={1}>
        <Goodbye signOff />
        You can put text here too.
        {{
            "Is it okay to put JSON values here?": true
        }}
        {< ExpressionInception
            nullProp={null}
            objectProp={{ "cool": true }}
            arrayProp={[0, "1", true, null]}
        / >}
        <>
            < >
                F R A G M E N T S
            < / >
        </>
        Text is fine here too.
    </Hello>
"#;

#[test]
fn parse_valid_jsx_element() {
    assert_eq!(
        jsx::root::<(&str, ErrorKind)>(VALID_JSX_ELEMENT),
        Ok((
            "",
            jsx::JsxValue::JsxElement(jsx::JsxElement::new(
                String::from("Hello"),
                {
                    let mut props = BTreeMap::new();
                    props.insert(
                        String::from("friend"),
                        jsx::JsxValue::JsonValue(json::JsonValue::Str(String::from("World"))),
                    );
                    props.insert(
                        String::from("count"),
                        jsx::JsxValue::JsxExpression(Box::new(jsx::JsxValue::JsonValue(
                            json::JsonValue::Num(1.0),
                        ))),
                    );
                    props
                },
                {
                    let mut children = vec![];
                    children.push(jsx::JsxValue::JsxElement(jsx::JsxElement::new(
                        String::from("Goodbye"),
                        {
                            let mut props = BTreeMap::new();
                            props.insert(
                                String::from("signOff"),
                                jsx::JsxValue::JsonValue(json::JsonValue::Boolean(true)),
                            );
                            props
                        },
                        vec![],
                    )));
                    children.push(jsx::JsxValue::JsonValue(json::JsonValue::Str(
                        String::from("You can put text here too."),
                    )));
                    children.push(jsx::JsxValue::JsxExpression(Box::new(
                        jsx::JsxValue::JsonValue(json::JsonValue::Object({
                            let mut object = BTreeMap::new();
                            object.insert(
                                String::from("Is it okay to put JSON values here?"),
                                json::JsonValue::Boolean(true),
                            );
                            object
                        })),
                    )));
                    children.push(jsx::JsxValue::JsxExpression(Box::new(
                        jsx::JsxValue::JsxElement(jsx::JsxElement::new(
                            String::from("ExpressionInception"),
                            {
                                let mut props = BTreeMap::new();
                                props.insert(
                                    String::from("nullProp"),
                                    jsx::JsxValue::JsxExpression(Box::new(
                                        jsx::JsxValue::JsonValue(json::JsonValue::Null),
                                    )),
                                );
                                props.insert(String::from("objectProp"), {
                                    let mut object = BTreeMap::new();
                                    object.insert(
                                        String::from("cool"),
                                        json::JsonValue::Boolean(true),
                                    );
                                    jsx::JsxValue::JsxExpression(Box::new(
                                        jsx::JsxValue::JsonValue(json::JsonValue::Object(object)),
                                    ))
                                });
                                props.insert(String::from("arrayProp"), {
                                    let mut array = vec![];
                                    array.push(json::JsonValue::Num(0.0));
                                    array.push(json::JsonValue::Str(String::from("1")));
                                    array.push(json::JsonValue::Boolean(true));
                                    array.push(json::JsonValue::Null);
                                    jsx::JsxValue::JsxExpression(Box::new(
                                        jsx::JsxValue::JsonValue(json::JsonValue::Array(array)),
                                    ))
                                });
                                props
                            },
                            vec![],
                        )),
                    )));
                    children.push(jsx::JsxValue::JsxFragment(jsx::JsxFragment::new({
                        let mut children = vec![];
                        children.push(jsx::JsxValue::JsxFragment(jsx::JsxFragment::new({
                            let mut children = vec![];
                            children.push(jsx::JsxValue::JsonValue(json::JsonValue::Str(
                                String::from("F R A G M E N T S"),
                            )));
                            children
                        })));
                        children
                    })));
                    children.push(jsx::JsxValue::JsonValue(json::JsonValue::Str(
                        String::from("Text is fine here too."),
                    )));
                    children
                },
            )),
        ))
    )
}

#[test]
fn serialize_jsx_element() {
    assert_eq!(
        serde_json::to_string_pretty(&jsx::root::<(&str, ErrorKind)>(VALID_JSX_ELEMENT).unwrap().1)
            .unwrap(),
        String::from(
            r#"{
  "type": "Hello",
  "props": {
    "count": 1.0,
    "friend": "World"
  },
  "children": [
    {
      "type": "Goodbye",
      "props": {
        "signOff": true
      },
      "children": []
    },
    "You can put text here too.",
    {
      "Is it okay to put JSON values here?": true
    },
    {
      "type": "ExpressionInception",
      "props": {
        "arrayProp": [
          0.0,
          "1",
          true,
          null
        ],
        "nullProp": null,
        "objectProp": {
          "cool": true
        }
      },
      "children": []
    },
    {
      "type": "Fragment",
      "children": [
        {
          "type": "Fragment",
          "children": [
            "F R A G M E N T S"
          ]
        }
      ]
    },
    "Text is fine here too."
  ]
}"#
        )
    )
}
