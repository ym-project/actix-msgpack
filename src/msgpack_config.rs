use super::{ErrorHandler, DEFAULT_PAYLOAD_LIMIT};
use crate::MsgPackError;
use actix_web::{error::Error, HttpRequest};
use std::sync::Arc;

#[derive(Clone)]
pub struct MsgPackConfig {
	pub(crate) limit: usize,
	pub(crate) error_handler: Option<ErrorHandler>,
}

pub const DEFAULT_CONFIG: MsgPackConfig =
	MsgPackConfig { limit: DEFAULT_PAYLOAD_LIMIT, error_handler: None };

impl MsgPackConfig {
	/// Set maximum accepted payload size in bytes. The default limit is 256KiB.
	pub fn limit(&mut self, limit: usize) -> &mut Self {
		self.limit = limit;
		self
	}

	pub fn error_handler<F>(&mut self, handler: F) -> &mut Self
	where
		F: Fn(MsgPackError, &HttpRequest) -> Error + Send + Sync + 'static,
	{
		self.error_handler = Some(Arc::new(handler));
		self
	}
}

impl Default for MsgPackConfig {
	fn default() -> Self {
		DEFAULT_CONFIG
	}
}
