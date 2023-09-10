use serde::{Deserialize, Serialize};

use crate::{Op, Operation, RpcMessage};

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentTimeRequest {}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentTimeResponse {
    pub time: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CurrentTime {
    request: Option<CurrentTimeRequest>,
    pub response: Option<CurrentTimeResponse>,
}

impl CurrentTime {
    #[allow(dead_code)]
    pub fn make_request() -> RpcMessage {
        let req = CurrentTimeRequest {};
        let op = Op::CurrentTime(Self {
            request: Some(req),
            response: None,
        });
        RpcMessage {
            id: uuid::Uuid::new_v4(),
            op,
        }
    }
}

impl Operation for CurrentTime {
    fn process(&self) -> Self {
        let resp = CurrentTimeResponse {
            time: chrono::Utc::now().to_string(),
        };
        Self {
            request: None,
            response: Some(resp),
        }
    }
}
