//! Original source argument order and fixed props; renderer selection remains admin-owned.
use super::{ComponentDefinition, PropDefinition, PropType};
use serde_json::Value;

/// The original 44 contracts, including import provenance and required positional arguments.
pub(super) fn components() -> Vec<ComponentDefinition> {
    vec![
        // attributes.ts:1272 -> frui/form/CodeEditor
        (
            "field.code",
            vec![
                ("className", PropType::String, false),
                ("extensions", PropType::String, false),
                ("language", PropType::String, false),
                ("numbers", PropType::String, false),
                ("setup", PropType::String, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:1327 -> frui/form/ColorInput
        (
            "field.color",
            vec![
                ("className", PropType::String, false),
                ("input", PropType::Json, false),
                ("picker", PropType::Json, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:1377 -> frui/form/CountrySelect
        (
            "field.country",
            vec![
                ("className", PropType::String, false),
                ("display", PropType::Json, false),
                ("dropdown", PropType::Json, false),
                ("option", PropType::Json, false),
                ("placeholder", PropType::String, false),
                ("searchable", PropType::Json, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:1457 -> frui/form/CurrencySelect
        (
            "field.currency",
            vec![
                ("className", PropType::String, false),
                ("display", PropType::Json, false),
                ("dropdown", PropType::Json, false),
                ("option", PropType::Json, false),
                ("placeholder", PropType::String, false),
                ("searchable", PropType::Json, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:1697 -> frui/form/Input
        (
            "field.email",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"type": "email"}),
        ),
        // attributes.ts:1728 -> frui/form/FileInput
        (
            "field.file",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
                ("uploading", PropType::String, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:1806 -> frui/form/ImageInput
        (
            "field.image",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:1970 -> frui/form/CodeEditor
        (
            "field.json",
            vec![
                ("className", PropType::String, false),
                ("extensions", PropType::String, false),
                ("numbers", PropType::String, false),
            ],
            serde_json::json!({"language": "json", "setup": "basic"}),
        ),
        // attributes.ts:2012 -> frui/form/MarkdownEditor
        (
            "field.markdown",
            vec![
                ("className", PropType::String, false),
                ("rows", PropType::Number, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:2051 -> frui/form/MaskInput
        (
            "field.mask",
            vec![
                ("className", PropType::String, false),
                ("mask", PropType::String, true),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:2511 -> frui/form/Radio
        (
            "field.radio",
            vec![
                ("blue", PropType::Boolean, false),
                ("check", PropType::Boolean, false),
                ("checked", PropType::Boolean, false),
                ("circle", PropType::Boolean, false),
                ("className", PropType::String, false),
                ("defaultChecked", PropType::String, false),
                ("label", PropType::String, false),
                ("orange", PropType::String, false),
                ("rounded", PropType::Boolean, false),
                ("square", PropType::Boolean, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:3052 -> frui/form/NumberInput
        (
            "field.small",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"max": 9, "min": 0, "step": 1}),
        ),
        // attributes.ts:3132 -> frui/form/SuggestInput
        (
            "field.suggest",
            vec![
                ("className", PropType::String, false),
                ("options", PropType::Json, false),
                ("remote", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:1568 -> frui/form/TextList
        (
            "field.datelist",
            vec![
                ("add", PropType::String, false),
                ("className", PropType::String, false),
                ("placeholder", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"type": "date"}),
        ),
        // attributes.ts:1648 -> frui/form/TextList
        (
            "field.datetimelist",
            vec![
                ("add", PropType::String, false),
                ("className", PropType::String, false),
                ("placeholder", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"type": "datetime"}),
        ),
        // attributes.ts:1767 -> frui/form/FileList
        (
            "field.filelist",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
                ("uploading", PropType::String, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:1837 -> frui/form/ImageList
        (
            "field.imagelist",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:2248 -> frui/form/TextList
        (
            "field.numberlist",
            vec![
                ("add", PropType::String, false),
                ("className", PropType::String, false),
                ("placeholder", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"type": "number"}),
        ),
        // attributes.ts:3733 -> frui/form/TextList
        (
            "field.timelist",
            vec![
                ("add", PropType::String, false),
                ("className", PropType::String, false),
                ("placeholder", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"type": "time"}),
        ),
        // attributes.ts:3816 -> Fieldset
        ("field.fieldset", vec![], serde_json::json!({})),
        // attributes.ts:4008 -> frui/view/TextOverflow
        (
            "view.chars",
            vec![
                ("length", PropType::Json, false),
                ("hellip", PropType::Boolean, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4205 -> frui/view/Spread
        (
            "view.comma",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"separator": ", "}),
        ),
        // attributes.ts:4236 -> frui/view/Country
        (
            "view.country",
            vec![
                ("className", PropType::String, false),
                ("flag", PropType::Boolean, false),
                ("lg", PropType::Boolean, false),
                ("md", PropType::Boolean, false),
                ("sm", PropType::Boolean, false),
                ("style", PropType::Json, false),
                ("text", PropType::Boolean, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4307 -> frui/view/Currency
        (
            "view.currency",
            vec![
                ("className", PropType::String, false),
                ("flag", PropType::Boolean, false),
                ("lg", PropType::Boolean, false),
                ("md", PropType::Boolean, false),
                ("sm", PropType::Boolean, false),
                ("style", PropType::Json, false),
                ("text", PropType::Boolean, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4409 -> frui/view/EmailLink
        (
            "view.email",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
                ("target", PropType::String, false),
                ("title", PropType::String, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4456 -> frui/view/Formula
        (
            "view.formula",
            vec![("formula", PropType::String, false)],
            serde_json::json!({}),
        ),
        // attributes.ts:4668 -> frui/view/Spread
        (
            "view.line",
            vec![
                ("className", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"separator": "line"}),
        ),
        // attributes.ts:5025 -> frui/view/DateFormat
        (
            "view.relative",
            vec![("locale", PropType::String, false)],
            serde_json::json!({"format": "ago"}),
        ),
        // attributes.ts:5235 -> frui/view/TextTransform
        (
            "view.transform",
            vec![("format", PropType::String, false)],
            serde_json::json!({}),
        ),
        // attributes.ts:5286 -> frui/view/TextOverflow
        (
            "view.words",
            vec![
                ("length", PropType::Json, false),
                ("hellip", PropType::Boolean, false),
            ],
            serde_json::json!({"words": true}),
        ),
        // attributes.ts:3913 -> frui/view/ImageCarousel
        (
            "view.carousel",
            vec![
                ("auto", PropType::Boolean, false),
                ("className", PropType::String, false),
                ("defaultIndex", PropType::Number, false),
                ("film", PropType::Json, false),
                ("frame", PropType::Json, false),
                ("hidden", PropType::Boolean, false),
                ("image", PropType::Json, false),
                ("repeat", PropType::Boolean, false),
                ("scroll", PropType::Boolean, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4039 -> frui/view/Code
        (
            "view.code",
            vec![
                ("addDefaultStyles", PropType::Boolean, false),
                ("className", PropType::String, false),
                ("langClassName", PropType::String, false),
                ("langStyle", PropType::Json, false),
                ("language", PropType::String, true),
                ("numbers", PropType::Boolean, false),
                ("showLanguage", PropType::Boolean, false),
                ("showLineNumbers", PropType::Boolean, false),
                ("startingLineNumber", PropType::Number, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4134 -> frui/view/Color
        (
            "view.color",
            vec![
                ("box", PropType::Boolean, false),
                ("className", PropType::String, false),
                ("lg", PropType::Boolean, false),
                ("md", PropType::Boolean, false),
                ("sm", PropType::Boolean, false),
                ("style", PropType::Json, false),
                ("text", PropType::Boolean, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4621 -> frui/view/ImageFilm
        (
            "view.film",
            vec![
                ("className", PropType::String, false),
                ("frame", PropType::Json, false),
                ("image", PropType::Json, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4532 -> frui/view/Code
        (
            "view.json",
            vec![
                ("addDefaultStyles", PropType::Boolean, false),
                ("className", PropType::String, false),
                ("langClassName", PropType::String, false),
                ("langStyle", PropType::Json, false),
                ("numbers", PropType::Boolean, false),
                ("showLanguage", PropType::Boolean, false),
                ("showLineNumbers", PropType::Boolean, false),
                ("startingLineNumber", PropType::Number, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({"language": "json"}),
        ),
        // attributes.ts:4783 -> frui/view/Markdown
        ("view.markdown", vec![], serde_json::json!({})),
        // attributes.ts:4963 -> frui/view/Rating
        (
            "view.rating",
            vec![
                ("max", PropType::Number, false),
                ("remainder", PropType::Boolean, false),
                ("round", PropType::String, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:4746 -> frui/view/List
        (
            "view.list",
            vec![("ordered", PropType::Boolean, false)],
            serde_json::json!({}),
        ),
        // attributes.ts:4875 -> frui/view/List
        ("view.ol", vec![], serde_json::json!({"ordered": true})),
        // attributes.ts:5048 -> frui/view/Spread
        (
            "view.spread",
            vec![
                ("className", PropType::String, false),
                ("separator", PropType::String, false),
                ("style", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:5087 -> frui/view/Tabular
        (
            "view.tabular",
            vec![
                ("style", PropType::Json, false),
                ("className", PropType::String, false),
                ("stripes", PropType::Json, false),
            ],
            serde_json::json!({}),
        ),
        // attributes.ts:5258 -> frui/view/List
        ("view.ul", vec![], serde_json::json!({})),
        // attributes.ts:5351 -> Fieldset
        ("view.fieldset", vec![], serde_json::json!({})),
        // attributes.ts:5002 -> frui/view/DateFormat
        (
            "view.rel",
            vec![("locale", PropType::String, false)],
            serde_json::json!({"format": "a"}),
        ),
    ]
    .into_iter()
    .map(|(name, props, fixed)| definition(name, props, fixed))
    .collect()
}

/// Build one metadata-only source definition without repeating revision/public/fixed policies.
fn definition(name: &str, props: Vec<(&str, PropType, bool)>, fixed: Value) -> ComponentDefinition {
    let mut props = props
        .into_iter()
        .map(|(name, value_type, required)| PropDefinition {
            name: name.into(),
            value_type,
            required,
            public: true,
        })
        .collect::<Vec<_>>();
    // Registry validation also checks fixed attributes. Append their signatures after
    // the complete original positional list, so no authored source argument shifts.
    for (name, value) in fixed.as_object().expect("literal source fixed props") {
        if !props.iter().any(|prop| prop.name == *name) {
            let value_type = match value {
                Value::Bool(_) => PropType::Boolean,
                Value::String(_) => PropType::String,
                Value::Number(_) => PropType::Number,
                _ => PropType::Json,
            };
            props.push(PropDefinition {
                name: name.clone(),
                value_type,
                required: false,
                public: true,
            });
        }
    }
    ComponentDefinition {
        name: name.into(),
        key: format!("stackpress.source.{name}"),
        revision: "source-1".into(),
        props,
        fixed: serde_json::from_value(fixed).expect("literal source fixed props"),
        available: false,
    }
}
