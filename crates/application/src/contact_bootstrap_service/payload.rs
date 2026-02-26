use serde_json::{Map, Value};

use super::{
    DISPLAY_NAME_FIELD_LOGICAL_NAME, EMAIL_FIELD_LOGICAL_NAME, SUBJECT_FIELD_LOGICAL_NAME,
};

pub(super) fn build_contact_payload(
    subject: &str,
    display_name: &str,
    email: Option<&str>,
) -> Value {
    let mut payload = Map::new();
    payload.insert(
        SUBJECT_FIELD_LOGICAL_NAME.to_owned(),
        Value::String(subject.to_owned()),
    );
    payload.insert(
        DISPLAY_NAME_FIELD_LOGICAL_NAME.to_owned(),
        Value::String(display_name.to_owned()),
    );
    if let Some(address) = email.filter(|value| !value.trim().is_empty()) {
        payload.insert(
            EMAIL_FIELD_LOGICAL_NAME.to_owned(),
            Value::String(address.to_owned()),
        );
    }

    Value::Object(payload)
}
