//! The GVariant shapes of the IBus objects that torpedo sends.
//!
//! Each IBus object is a struct that starts with `(s type_name, a{sv}
//! attachments)` and travels wrapped in a `v`. In zvariant a `Value` that is a
//! tuple field or an element of a `Vec<Value>` is already encoded as `v`. So
//! these helpers never call `Value::new()` on a nested object: that would give
//! `v<v<..>>`, which makes libibus fail on the nested attribute list.

use std::collections::HashMap;

use zbus::zvariant::Value;

type Attachments = HashMap<String, Value<'static>>;

/// Attribute type UNDERLINE.
pub const ATTR_TYPE_UNDERLINE: u32 = 1;
/// Underline value SINGLE.
pub const ATTR_UNDERLINE_SINGLE: u32 = 1;
/// UpdatePreeditText mode: the client drops the preedit on focus-out.
pub const PREEDIT_CLEAR: u32 = 0;
/// UpdatePreeditText mode: the daemon commits a visible preedit on focus-out,
/// reset, disable and engine switch.
pub const PREEDIT_COMMIT: u32 = 1;
/// Lookup table orientation: vertical.
pub const ORIENTATION_VERTICAL: i32 = 1;

/// An IBusAttribute: `(sa{sv}uuuu)`.
pub fn attribute(kind: u32, value: u32, start: u32, end: u32) -> Value<'static> {
    Value::from((
        "IBusAttribute".to_string(),
        Attachments::new(),
        kind,
        value,
        start,
        end,
    ))
}

/// An IBusAttrList: `(sa{sv}av)`.
pub fn attr_list(attrs: Vec<Value<'static>>) -> Value<'static> {
    Value::from(("IBusAttrList".to_string(), Attachments::new(), attrs))
}

/// An IBusText with attributes: `(sa{sv}sv)`.
pub fn text_with_attrs(text: &str, attrs: Vec<Value<'static>>) -> Value<'static> {
    Value::from((
        "IBusText".to_string(),
        Attachments::new(),
        text.to_string(),
        attr_list(attrs),
    ))
}

/// An IBusText with no attributes.
pub fn text(text: &str) -> Value<'static> {
    text_with_attrs(text, vec![])
}

/// An IBusText underlined from start to end.
pub fn underlined(text: &str) -> Value<'static> {
    let end = text.chars().count() as u32;
    text_with_attrs(
        text,
        vec![attribute(
            ATTR_TYPE_UNDERLINE,
            ATTR_UNDERLINE_SINGLE,
            0,
            end,
        )],
    )
}

/// An IBusLookupTable: `(sa{sv}uubbiavav)`.
pub fn lookup_table(
    candidates: &[String],
    page_size: u32,
    cursor_pos: u32,
    orientation: i32,
) -> Value<'static> {
    let c: Vec<Value<'static>> = candidates.iter().map(|t| text(t)).collect();
    let labels: Vec<Value<'static>> = Vec::new();
    Value::from((
        "IBusLookupTable".to_string(),
        Attachments::new(),
        page_size,
        cursor_pos,
        true,
        false,
        orientation,
        c,
        labels,
    ))
}

/// IBusProperty type NORMAL: a menu item that the user activates.
pub const PROP_TYPE_NORMAL: u32 = 0;
/// IBusProperty state UNCHECKED.
pub const PROP_STATE_UNCHECKED: u32 = 0;

/// An IBusPropList: `(sa{sv}av)`.
pub fn prop_list(props: Vec<Value<'static>>) -> Value<'static> {
    Value::from(("IBusPropList".to_string(), Attachments::new(), props))
}

/// An IBusProperty of type NORMAL, sensitive, visible and unchecked, with no
/// sub-properties: `(sa{sv}suvsvbbuvv)`. The fields are key, type, label,
/// icon, tooltip, sensitive, visible, state, sub-properties and symbol.
pub fn property(key: &str, label: &str, icon: &str, tooltip: &str) -> Value<'static> {
    Value::from((
        "IBusProperty".to_string(),
        Attachments::new(),
        key.to_string(),
        PROP_TYPE_NORMAL,
        text(label),
        icon.to_string(),
        text(tooltip),
        true,
        true,
        PROP_STATE_UNCHECKED,
        prop_list(vec![]),
        text(""),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The nesting of a value: `v<..>` for a variant, `(..)` for a struct.
    fn tree(v: &Value) -> String {
        match v {
            Value::Value(inner) => format!("v<{}>", tree(inner)),
            Value::Structure(s) => {
                format!(
                    "({})",
                    s.fields().iter().map(tree).collect::<Vec<_>>().join(",")
                )
            }
            Value::Array(a) => format!("[{}]", a.iter().map(tree).collect::<Vec<_>>().join(",")),
            other => other.value_signature().to_string(),
        }
    }

    #[test]
    fn attribute_has_the_ibus_signature() {
        assert_eq!(
            attribute(1, 1, 0, 2).value_signature().to_string(),
            "(sa{sv}uuuu)"
        );
    }

    #[test]
    fn attr_list_has_the_ibus_signature() {
        assert_eq!(
            attr_list(vec![]).value_signature().to_string(),
            "(sa{sv}av)"
        );
    }

    #[test]
    fn text_has_the_ibus_signature() {
        assert_eq!(text("x").value_signature().to_string(), "(sa{sv}sv)");
    }

    #[test]
    fn lookup_table_has_the_ibus_signature() {
        let t = lookup_table(&["🔥 fire".to_string()], 9, 0, ORIENTATION_VERTICAL);
        assert_eq!(t.value_signature().to_string(), "(sa{sv}uubbiavav)");
    }

    #[test]
    fn nested_attributes_are_wrapped_once() {
        assert_eq!(
            tree(&underlined(":a")),
            "(s,a{sv},s,v<(s,a{sv},[v<(s,a{sv},u,u,u,u)>])>)"
        );
    }

    #[test]
    fn underline_spans_characters_not_bytes() {
        let Value::Structure(s) = underlined("::é") else {
            panic!()
        };
        let Value::Value(list) = &s.fields()[3] else {
            panic!()
        };
        let Value::Structure(list) = list.as_ref() else {
            panic!()
        };
        let Value::Array(attrs) = &list.fields()[2] else {
            panic!()
        };
        let Value::Value(attr) = &attrs.inner()[0] else {
            panic!()
        };
        let Value::Structure(attr) = attr.as_ref() else {
            panic!()
        };
        assert_eq!(attr.fields()[5], Value::U32(3));
    }

    #[test]
    fn property_has_the_ibus_signature() {
        let p = property("k", "Label", "icon", "tip");
        assert_eq!(p.value_signature().to_string(), "(sa{sv}suvsvbbuvv)");
        let list = prop_list(vec![p]);
        assert_eq!(list.value_signature().to_string(), "(sa{sv}av)");
        assert_eq!(
            tree(&list),
            "(s,a{sv},[v<(s,a{sv},s,u,v<(s,a{sv},s,v<(s,a{sv},[])>)>,s,v<(s,a{sv},s,v<(s,a{sv},[])>)>,b,b,u,v<(s,a{sv},[])>,v<(s,a{sv},s,v<(s,a{sv},[])>)>)>])"
        );
    }
}
