use crate::MsgPackError;
use crate::DEFAULT_PAYLOAD_LIMIT;
use actix_web::dev::Payload;
use actix_web::error::PayloadError;
use actix_web::http::header::CONTENT_LENGTH;
use actix_web::web::BytesMut;
use actix_web::{HttpMessage, HttpRequest};
use futures_util::future::LocalBoxFuture;
use futures_util::stream::StreamExt;
use futures_util::FutureExt;
use mime::APPLICATION_MSGPACK;
use serde::de::DeserializeOwned;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::task::{self, Poll};

pub struct MsgPackMessage<T> {
	limit: usize,
	length: Option<usize>,
	stream: Option<Payload>,
	err: Option<MsgPackError>,
	fut: Option<LocalBoxFuture<'static, Result<T, MsgPackError>>>,
}

impl<T> MsgPackMessage<T> {
	pub fn new(req: &HttpRequest, payload: &mut Payload) -> Self {
		if req.content_type() != APPLICATION_MSGPACK {
			return MsgPackMessage {
				limit: DEFAULT_PAYLOAD_LIMIT,
				length: None,
				stream: None,
				fut: None,
				err: Some(MsgPackError::ContentType),
			};
		}

		let mut length = None;

		if let Some(content_length) = req.headers().get(CONTENT_LENGTH) {
			if let Ok(string) = content_length.to_str() {
				if let Ok(l) = string.parse::<usize>() {
					length = Some(l)
				}
			}
		}

		MsgPackMessage {
			limit: DEFAULT_PAYLOAD_LIMIT,
			length,
			stream: Some(payload.take()),
			fut: None,
			err: None,
		}
	}

	/// Set maximum accepted payload size in bytes
	pub fn limit(mut self, limit: usize) -> Self {
		self.limit = limit;
		self
	}
}

impl<T: DeserializeOwned + 'static> Future for MsgPackMessage<T> {
	type Output = Result<T, MsgPackError>;

	fn poll(mut self: Pin<&mut Self>, task: &mut task::Context<'_>) -> Poll<Self::Output> {
		if let Some(ref mut fut) = self.fut {
			return Pin::new(fut).poll(task);
		}

		if let Some(err) = self.err.take() {
			return Poll::Ready(Err(err));
		}

		let limit = self.limit;

		if let Some(len) = self.length.take() {
			if len > limit {
				return Poll::Ready(Err(MsgPackError::Overflow));
			}
		}

		let mut stream = self.stream.take().expect("MsgPackMessage could not be used second time");

		self.fut = Some(
			async move {
				let mut body = BytesMut::with_capacity(8192);

				while let Some(item) = stream.next().await {
					let chunk = item?;
					if (body.len() + chunk.len()) > limit {
						return Err(MsgPackError::Overflow);
					} else {
						body.extend_from_slice(&chunk);
					}
				}

				if body.len() == 0 {
					return Err(MsgPackError::Payload(PayloadError::Incomplete(Some(
						io::Error::new(io::ErrorKind::InvalidData, "payload is empty"),
					))));
				}

				Ok(rmp_serde::from_slice::<T>(&body)?)
			}
			.boxed_local(),
		);
		self.poll(task)
	}
}
