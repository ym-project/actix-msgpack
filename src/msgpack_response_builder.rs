use crate::MsgPackError;
use actix_web::http::header::ContentType;
use actix_web::web::Bytes;
use actix_web::{HttpResponse, HttpResponseBuilder};
use mime::APPLICATION_MSGPACK;
use serde::Serialize;

pub trait MsgPackResponseBuilder {
	/// MessagePack Responder
	fn msgpack<T: Serialize>(&mut self, value: T) -> HttpResponse;
}

impl MsgPackResponseBuilder for HttpResponseBuilder {
	fn msgpack<T: Serialize>(&mut self, value: T) -> HttpResponse {
		match rmp_serde::to_vec_named(&value) {
			Ok(body) => {
				self.insert_header(ContentType(APPLICATION_MSGPACK));
				self.body(Bytes::from(body))
			},
			Err(err) => HttpResponse::from_error(MsgPackError::Serialize(err)),
		}
	}
}
