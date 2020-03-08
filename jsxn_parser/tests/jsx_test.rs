use jsxn_parser::{json, jsx};
use nom::error::ErrorKind;
use std::collections::HashMap;

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
            jsx::JsxValue::JsxElement((
                String::from("Hello"),
                {
                    let mut object = HashMap::new();
                    object.insert(
                        String::from("friend"),
                        jsx::JsxValue::JsonValue(json::JsonValue::Str(String::from("World"))),
                    );
                    object.insert(
                        String::from("count"),
                        jsx::JsxValue::JsxExpression(Box::new(jsx::JsxValue::JsonValue(
                            json::JsonValue::Num(1.0),
                        ))),
                    );
                    object
                },
                {
                    let mut children = vec![];
                    children.push(jsx::JsxValue::JsxElement((
                        String::from("Goodbye"),
                        {
                            let mut object = HashMap::new();
                            object.insert(
                                String::from("signOff"),
                                jsx::JsxValue::JsonValue(json::JsonValue::Boolean(true)),
                            );
                            object
                        },
                        vec![],
                    )));
                    children.push(jsx::JsxValue::JsonValue(json::JsonValue::Str(
                        String::from("You can put text here too."),
                    )));
                    children.push(jsx::JsxValue::JsxExpression(Box::new(
                        jsx::JsxValue::JsonValue(json::JsonValue::Object({
                            let mut object = HashMap::new();
                            object.insert(
                                String::from("Is it okay to put JSON values here?"),
                                json::JsonValue::Boolean(true),
                            );
                            object
                        })),
                    )));
                    children.push(jsx::JsxValue::JsxExpression(Box::new(
                        jsx::JsxValue::JsxElement((
                            String::from("ExpressionInception"),
                            {
                                let mut object = HashMap::new();
                                object.insert(
                                    String::from("nullProp"),
                                    jsx::JsxValue::JsxExpression(Box::new(
                                        jsx::JsxValue::JsonValue(json::JsonValue::Null),
                                    )),
                                );
                                object.insert(String::from("objectProp"), {
                                    let mut object = HashMap::new();
                                    object.insert(
                                        String::from("cool"),
                                        json::JsonValue::Boolean(true),
                                    );
                                    jsx::JsxValue::JsxExpression(Box::new(
                                        jsx::JsxValue::JsonValue(json::JsonValue::Object(object)),
                                    ))
                                });
                                object.insert(String::from("arrayProp"), {
                                    let mut array = vec![];
                                    array.push(json::JsonValue::Num(0.0));
                                    array.push(json::JsonValue::Str(String::from("1")));
                                    array.push(json::JsonValue::Boolean(true));
                                    array.push(json::JsonValue::Null);
                                    jsx::JsxValue::JsxExpression(Box::new(
                                        jsx::JsxValue::JsonValue(json::JsonValue::Array(array)),
                                    ))
                                });
                                object
                            },
                            vec![],
                        )),
                    )));
                    children.push(jsx::JsxValue::JsxFragment({
                        let mut children = vec![];
                        children.push(jsx::JsxValue::JsxFragment({
                            let mut children = vec![];
                            children.push(jsx::JsxValue::JsonValue(json::JsonValue::Str(
                                String::from("F R A G M E N T S"),
                            )));
                            children
                        }));
                        children
                    }));
                    children.push(jsx::JsxValue::JsonValue(json::JsonValue::Str(
                        String::from("Text is fine here too."),
                    )));
                    children
                },
            )),
        ))
    )
}
