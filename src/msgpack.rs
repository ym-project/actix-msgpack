use super::constants::DEFAULT_PAYLOAD_LIMIT;
use super::MsgPackConfig;
use super::MsgPackMessage;
use actix_web::dev::Payload;
use actix_web::error::Error;
use actix_web::{FromRequest, HttpRequest};
use futures_util::future::LocalBoxFuture;
use futures_util::FutureExt;
use serde::de::DeserializeOwned;
use std::fmt;
use std::ops::{Deref, DerefMut};

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
	type Future = LocalBoxFuture<'static, Result<Self, Error>>;

	#[inline]
	fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
		let data = req.app_data::<MsgPackConfig>();
		let limit = data.map(|item| item.limit).unwrap_or(DEFAULT_PAYLOAD_LIMIT);

		MsgPackMessage::new(req, payload)
			.limit(limit)
			.map(move |res| match res {
				Err(e) => Err(e.into()),
				Ok(item) => Ok(MsgPack(item)),
			})
			.boxed_local()
	}
}
