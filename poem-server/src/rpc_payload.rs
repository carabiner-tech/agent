//! This is basically a copy of the poem-openapi JSON payload implementation
//! https://github.com/poem-web/poem/blob/master/poem-openapi/src/payload/json.rs
//!
//! The key change is that the content type is application/json, without charset=utf-8
//! OpenAI seems to have a bug when the OpenAPI schema has a POST body request with that mediatype
//! https://community.openai.com/t/application-json-charset-utf-8-in-openapi-schema-leads-to-unrecognizedkwargserror/378477
use poem::{FromRequest, IntoResponse, Request, RequestBody, Response, Result};
use poem_openapi::impl_apirequest_for_payload;
use serde_json::Value;
use std::ops::{Deref, DerefMut};

use poem_openapi::{
    error::ParseRequestPayloadError,
    payload::{ParsePayload, Payload},
    registry::{MetaMediaType, MetaResponse, MetaResponses, MetaSchemaRef, Registry},
    types::{ParseFromJSON, ToJSON, Type},
    ApiResponse,
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RpcPayload<T>(pub T);

impl<T> Deref for RpcPayload<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for RpcPayload<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Type> Payload for RpcPayload<T> {
    const CONTENT_TYPE: &'static str = "application/json";

    // don't bother checking content types
    fn check_content_type(_content_type: &str) -> bool {
        true
    }

    fn schema_ref() -> MetaSchemaRef {
        T::schema_ref()
    }

    #[allow(unused_variables)]
    fn register(registry: &mut Registry) {
        T::register(registry);
    }
}

#[poem::async_trait]
impl<T: ParseFromJSON> ParsePayload for RpcPayload<T> {
    const IS_REQUIRED: bool = true;

    async fn from_request(request: &Request, body: &mut RequestBody) -> Result<Self> {
        let data: Vec<u8> = FromRequest::from_request(request, body).await?;
        let value = if data.is_empty() {
            Value::Null
        } else {
            serde_json::from_slice(&data).map_err(|err| ParseRequestPayloadError {
                reason: err.to_string(),
            })?
        };

        let value = T::parse_from_json(Some(value)).map_err(|err| ParseRequestPayloadError {
            reason: err.into_message(),
        })?;
        Ok(Self(value))
    }
}

impl<T: ToJSON> IntoResponse for RpcPayload<T> {
    fn into_response(self) -> Response {
        poem::web::Json(self.0.to_json()).into_response()
    }
}

impl<T: ToJSON> ApiResponse for RpcPayload<T> {
    fn meta() -> MetaResponses {
        MetaResponses {
            responses: vec![MetaResponse {
                description: "",
                status: Some(200),
                content: vec![MetaMediaType {
                    content_type: Self::CONTENT_TYPE,
                    schema: Self::schema_ref(),
                }],
                headers: vec![],
            }],
        }
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
    }
}

impl_apirequest_for_payload!(RpcPayload<T>, T: ParseFromJSON);
