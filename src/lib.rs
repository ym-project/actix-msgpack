mod constants;
mod msgpack;
mod msgpack_config;
mod msgpack_error;
mod msgpack_message;
mod msgpack_response_builder;

pub(crate) use constants::DEFAULT_PAYLOAD_LIMIT;
pub use msgpack::MsgPack;
pub use msgpack_config::MsgPackConfig;
pub use msgpack_error::MsgPackError;
pub use msgpack_message::MsgPackMessage;
pub use msgpack_response_builder::MsgPackResponseBuilder;

#[cfg(test)]
mod tests {
	use super::*;
	use actix_web::body::MessageBody;
	use actix_web::http::{header, StatusCode};
	use actix_web::test::TestRequest;
	use actix_web::web::Bytes;
	use actix_web::{HttpRequest, HttpResponse};
	use mime::{APPLICATION_JSON, APPLICATION_MSGPACK};
	use serde::{Deserialize, Serialize};

	impl PartialEq for MsgPackError {
		fn eq(&self, other: &MsgPackError) -> bool {
			match *self {
				MsgPackError::Overflow => {
					matches!(*other, MsgPackError::Overflow)
				},
				MsgPackError::ContentType => {
					matches!(*other, MsgPackError::ContentType)
				},
				_ => false,
			}
		}
	}

	#[derive(Debug, Serialize, Deserialize, PartialEq)]
	pub struct Data {
		payload: bool,
	}

	#[actix_web::test]
	async fn check_content_type() {
		// Pass empty Content-Type
		let (req, mut payload) = TestRequest::default().to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload).await;

		assert_eq!(msgpack.err().unwrap(), MsgPackError::ContentType);

		// Pass non-msgpack Content-Type
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_JSON))
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload).await;

		assert_eq!(msgpack.err().unwrap(), MsgPackError::ContentType);

		// Pass correct Content-Type
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload).await;

		assert_ne!(msgpack.err().unwrap(), MsgPackError::ContentType);
	}

	#[actix_web::test]
	async fn check_default_limit() {
		// Pass min limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 0))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload).await;

		assert_ne!(msgpack.err().unwrap(), MsgPackError::Overflow);

		// Pass max limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, DEFAULT_PAYLOAD_LIMIT))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload).await;

		assert_ne!(msgpack.err().unwrap(), MsgPackError::Overflow);

		// Pass more than default limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, DEFAULT_PAYLOAD_LIMIT + 1))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload).await;

		assert_eq!(msgpack.err().unwrap(), MsgPackError::Overflow);
	}

	#[actix_web::test]
	async fn check_custom_limit() {
		const LIMIT: usize = 10;

		// Pass max limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, LIMIT))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload).limit(LIMIT).await;

		assert_ne!(msgpack.err().unwrap(), MsgPackError::Overflow);

		// Pass more than limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, LIMIT + 1))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload).limit(LIMIT).await;

		assert_eq!(msgpack.err().unwrap(), MsgPackError::Overflow);
	}

	#[actix_web::test]
	async fn check_body() {
		// Pass empty body
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.to_http_parts();

		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload).await;

		assert!(matches!(msgpack.err().unwrap(), MsgPackError::Payload(..)));

		// Pass invalid body
		let data = Bytes::from_static(&[0x81]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 1))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload).await;

		assert!(matches!(msgpack.err().unwrap(), MsgPackError::Deserialize(..)));

		// Pass correct body
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 10))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload).await;

		assert_eq!(msgpack.ok().unwrap(), Data { payload: true })
	}

	#[actix_web::test]
	async fn check_body_limit() {
		// Pass body length == Content-Length value header
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 10))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload).await;

		assert_eq!(msgpack.ok().unwrap(), Data { payload: true });

		// Pass body length < Content-Length value header
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 11))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload).await;

		assert_eq!(msgpack.ok().unwrap(), Data { payload: true });

		// Pass body length > Content-Length value header
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 1))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload).await;

		assert_eq!(msgpack.ok().unwrap(), Data { payload: true });

		// Pass body and don't pass Content-Length header
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload).await;

		assert_eq!(msgpack.ok().unwrap(), Data { payload: true });
	}

	#[actix_web::test]
	async fn check_responses() {
		// Response with msgpack responder
		async fn service(_: HttpRequest) -> HttpResponse {
			let payload = Data { payload: true };
			HttpResponse::Ok().msgpack(payload)
		}

		let request = TestRequest::post()
			.uri("/")
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.to_http_request();
		let response = service(request).await;

		assert_eq!(response.status(), StatusCode::OK);
		assert_eq!(
			response.into_body().try_into_bytes().unwrap(),
			vec![0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]
		);
	}
}
