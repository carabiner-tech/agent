pub mod operations;
use enum_as_inner::EnumAsInner;
use enum_dispatch::enum_dispatch;
use serde::{Deserialize, Serialize};

use crate::operations::{current_time::CurrentTime, fs::ListFiles};

#[derive(Debug, Serialize, Deserialize)]
pub struct RpcMessage {
    pub id: uuid::Uuid,
    pub op: Op,
}

impl RpcMessage {
    pub fn new(op: Op) -> Self {
        let id = uuid::Uuid::new_v4();
        Self { id, op }
    }
}

#[enum_dispatch(Op)]
pub trait Operation {
    fn process(&self) -> Self;
}

#[derive(Serialize, Deserialize, Debug, EnumAsInner)]
#[enum_dispatch]
pub enum Op {
    CurrentTime(CurrentTime),
    ListFiles(ListFiles),
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;

    use super::*;

    #[test]
    fn test_simple_dispatch() {
        // Serialize a CurrentTime request, deserialize it, and dispatch it.
        let msg = CurrentTime::make_request();
        let msg_se = serde_json::to_string(&msg).unwrap();

        let msg_de: RpcMessage = serde_json::from_str(&msg_se).unwrap();
        let resp = msg_de.op.process();

        assert_matches!(resp, Op::CurrentTime(_));
    }

    #[test]
    fn test_invalid_op() {
        // Show what / where the error is when deserializing a message with an unknown operation
        let json_str = r#"{"id":"d0e9e0a0-0e1e-4e1e-8e1e-0e1e0e1e0e1e","op":{"MadeUpOperation":{"request":null,"response":null}}}"#;
        let err = serde_json::from_str::<RpcMessage>(json_str).unwrap_err();
        let err_msg = err.to_string();
        assert!(err_msg.contains("unknown variant"));
    }
}
