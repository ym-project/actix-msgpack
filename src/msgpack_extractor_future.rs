use crate::{ErrorHandler, MsgPack, MsgPackMessage};
use actix_web::{error::Error, HttpRequest};
use serde::de::DeserializeOwned;
use std::{
	future::Future,
	pin::Pin,
	task::{Context, Poll},
};

pub struct MsgPackExtractorFuture<T> {
	pub req: HttpRequest,
	pub fut: MsgPackMessage<T>,
	pub err_handler: Option<ErrorHandler>,
}

impl<T: DeserializeOwned + 'static> Future for MsgPackExtractorFuture<T> {
	type Output = Result<MsgPack<T>, Error>;

	fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
		let this = self.get_mut();
		let res = match Pin::new(&mut this.fut).poll(ctx) {
			Poll::Ready(data) => data,
			Poll::Pending => return Poll::Pending,
		};

		let res = match res {
			Err(err) => {
				if let Some(err_handler) = this.err_handler.as_ref() {
					Err((*err_handler)(err, &this.req))
				} else {
					Err(err.into())
				}
			},
			Ok(data) => Ok(MsgPack(data)),
		};

		Poll::Ready(res)
	}
}
