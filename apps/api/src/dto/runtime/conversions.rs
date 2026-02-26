use qryvanta_domain::RuntimeRecord;

use super::types::RuntimeRecordResponse;

impl From<RuntimeRecord> for RuntimeRecordResponse {
    fn from(value: RuntimeRecord) -> Self {
        Self {
            record_id: value.record_id().as_str().to_owned(),
            entity_logical_name: value.entity_logical_name().as_str().to_owned(),
            data: value.data().clone(),
        }
    }
}
