use super::MsgPackError;
use actix_web::{error::Error, HttpRequest};
use std::sync::Arc;

pub type ErrorHandler = Arc<dyn Fn(MsgPackError, &HttpRequest) -> Error + Send + Sync>;
