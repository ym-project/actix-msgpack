use actix_web::error::PayloadError;
use actix_web::{HttpResponse, ResponseError};
use derive_more::Display;
use rmp_serde::decode::Error as RmpSerdeDecodeError;
use rmp_serde::encode::Error as RmpSerdeEncodeError;

#[derive(Debug, Display)]
pub enum MsgPackError {
	/// Payload size is bigger than limit
	#[display(fmt = "Payload size is bigger than limit")]
	Overflow,

	/// Content type error
	#[display(fmt = "Content type error")]
	ContentType,

	/// Deserialize error
	#[display(fmt = "Deserialize error: {_0}")]
	Deserialize(RmpSerdeDecodeError),

	/// Serialize error
	#[display(fmt = "Serialize error: {_0}")]
	Serialize(RmpSerdeEncodeError),

	/// Payload error
	#[display(fmt = "Error that occur during reading payload: {_0}")]
	Payload(PayloadError),
}

impl ResponseError for MsgPackError {
	fn error_response(&self) -> HttpResponse {
		match *self {
			MsgPackError::Overflow => HttpResponse::PayloadTooLarge().into(),
			_ => HttpResponse::BadRequest().into(),
		}
	}
}

impl From<PayloadError> for MsgPackError {
	fn from(err: PayloadError) -> MsgPackError {
		MsgPackError::Payload(err)
	}
}

impl From<RmpSerdeDecodeError> for MsgPackError {
	fn from(err: RmpSerdeDecodeError) -> MsgPackError {
		MsgPackError::Deserialize(err)
	}
}
