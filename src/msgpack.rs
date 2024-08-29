use crate::MsgPackExtractorFuture;

use super::{MsgPackConfig, MsgPackError, MsgPackMessage, DEFAULT_CONFIG};
use actix_web::{
	body::BoxBody, dev::Payload, error::Error, FromRequest, HttpRequest, HttpResponse, Responder,
};
use mime::APPLICATION_MSGPACK;
use serde::{de::DeserializeOwned, Serialize};
use std::{
	fmt,
	ops::{Deref, DerefMut},
};

pub struct MsgPack<T>(pub T);

impl<T> Deref for MsgPack<T> {
	type Target = T;

	fn deref(&self) -> &T {
		&self.0
	}
}

impl<T> DerefMut for MsgPack<T> {
	fn deref_mut(&mut self) -> &mut T {
		&mut self.0
	}
}

impl<T> fmt::Debug for MsgPack<T>
where
	T: fmt::Debug,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "MsgPack: {:?}", self.0)
	}
}

impl<T> fmt::Display for MsgPack<T>
where
	T: fmt::Display,
{
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt::Display::fmt(&self.0, f)
	}
}

impl<T: DeserializeOwned> FromRequest for MsgPack<T>
where
	T: 'static,
{
	type Error = Error;
	type Future = MsgPackExtractorFuture<T>;

	#[inline]
	fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
		let config = req.app_data::<MsgPackConfig>().unwrap_or(&DEFAULT_CONFIG);
		let limit = config.limit;
		let err_handler = config.error_handler.clone();
		let content_type = config.content_type.clone();

		MsgPackExtractorFuture {
			req: req.clone(),
			fut: MsgPackMessage::new(req, payload, content_type).limit(limit),
			err_handler,
		}
	}
}

impl<T: Serialize> Responder for MsgPack<T> {
	type Body = BoxBody;

	fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
		match rmp_serde::to_vec_named(&self.0) {
			Ok(body) => {
				match HttpResponse::Ok().content_type(APPLICATION_MSGPACK).message_body(body) {
					Ok(response) => response.map_into_boxed_body(),
					Err(err) => HttpResponse::from_error(err),
				}
			},
			Err(err) => HttpResponse::from_error(MsgPackError::Serialize(err)),
		}
	}
}
