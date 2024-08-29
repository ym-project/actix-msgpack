mod constants;
mod content_type_handler;
mod error_handler;
mod msgpack;
mod msgpack_config;
mod msgpack_error;
mod msgpack_extractor_future;
mod msgpack_message;
mod msgpack_response_builder;

pub(crate) use constants::DEFAULT_PAYLOAD_LIMIT;
pub use content_type_handler::ContentTypeHandler;
pub use error_handler::ErrorHandler;
pub use msgpack::MsgPack;
pub use msgpack_config::MsgPackConfig;
pub(crate) use msgpack_config::DEFAULT_CONFIG;
pub use msgpack_error::MsgPackError;
pub use msgpack_extractor_future::MsgPackExtractorFuture;
pub use msgpack_message::MsgPackMessage;
pub use msgpack_response_builder::MsgPackResponseBuilder;

#[cfg(test)]
mod tests {
	use super::*;
	use actix_web::{
		body::MessageBody,
		error::InternalError,
		http::{header, Method, StatusCode},
		test::{call_service, init_service, TestRequest},
		web::{self, Bytes},
		App, HttpRequest, HttpResponse, Responder,
	};
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
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload, None).await;

		assert_eq!(msgpack.err().unwrap(), MsgPackError::ContentType);

		// Pass non-msgpack Content-Type
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_JSON))
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload, None).await;

		assert_eq!(msgpack.err().unwrap(), MsgPackError::ContentType);

		// Pass correct Content-Type
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload, None).await;

		assert_ne!(msgpack.err().unwrap(), MsgPackError::ContentType);
	}

	#[actix_web::test]
	async fn check_default_limit() {
		// Pass min limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 0))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload, None).await;

		assert_ne!(msgpack.err().unwrap(), MsgPackError::Overflow);

		// Pass max limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, DEFAULT_PAYLOAD_LIMIT))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload, None).await;

		assert_ne!(msgpack.err().unwrap(), MsgPackError::Overflow);

		// Pass more than default limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, DEFAULT_PAYLOAD_LIMIT + 1))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload, None).await;

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
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload, None).limit(LIMIT).await;

		assert_ne!(msgpack.err().unwrap(), MsgPackError::Overflow);

		// Pass more than limit
		let (req, mut payload) = TestRequest::default()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, LIMIT + 1))
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload, None).limit(LIMIT).await;

		assert_eq!(msgpack.err().unwrap(), MsgPackError::Overflow);
	}

	#[actix_web::test]
	async fn check_body() {
		// Pass empty body
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.to_http_parts();

		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload, None).await;

		assert!(matches!(msgpack.err().unwrap(), MsgPackError::Payload(..)));

		// Pass invalid body
		let data = Bytes::from_static(&[0x81]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 1))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<()>::new(&req, &mut payload, None).await;

		assert!(matches!(msgpack.err().unwrap(), MsgPackError::Deserialize(..)));

		// Pass correct body
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 10))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload, None).await;

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
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload, None).await;

		assert_eq!(msgpack.ok().unwrap(), Data { payload: true });

		// Pass body length < Content-Length value header
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 11))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload, None).await;

		assert_eq!(msgpack.ok().unwrap(), Data { payload: true });

		// Pass body length > Content-Length value header
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.insert_header((header::CONTENT_LENGTH, 1))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload, None).await;

		assert_eq!(msgpack.ok().unwrap(), Data { payload: true });

		// Pass body and don't pass Content-Length header
		let data =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);
		let (req, mut payload) = TestRequest::post()
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.set_payload(data)
			.to_http_parts();
		let msgpack = MsgPackMessage::<Data>::new(&req, &mut payload, None).await;

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

	#[actix_web::test]
	async fn check_responder() {
		// Pass correct payload
		let request = TestRequest::default().to_http_request();
		let payload = MsgPack(Bytes::from_static(&[
			0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3,
		]));
		let response = payload.clone().respond_to(&request);

		assert_eq!(response.status(), StatusCode::OK);
		assert_eq!(
			response.into_body().try_into_bytes().unwrap(),
			vec![0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]
		);
	}

	#[actix_web::test]
	async fn check_empty_error_handler() {
		async fn service(_: MsgPack<Data>) -> HttpResponse {
			HttpResponse::Ok().finish()
		}

		let app = init_service(
			App::new().app_data(MsgPackConfig::default()).route("/", web::post().to(service)),
		)
		.await;
		let request = TestRequest::default()
			.method(Method::POST)
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.to_request();
		let response = call_service(&app, request).await;

		assert_eq!(response.status(), StatusCode::BAD_REQUEST);
	}

	#[actix_web::test]
	async fn check_error_handler() {
		async fn service(_: MsgPack<Data>) -> HttpResponse {
			HttpResponse::Ok().finish()
		}

		let mut config = MsgPackConfig::default();
		config.error_handler(|err, _req| {
			InternalError::from_response(err, HttpResponse::NotAcceptable().finish()).into()
		});
		let app =
			init_service(App::new().app_data(config).route("/", web::post().to(service))).await;

		let request = TestRequest::default()
			.method(Method::POST)
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.to_request();
		let response = call_service(&app, request).await;

		assert_eq!(response.status(), StatusCode::NOT_ACCEPTABLE);
	}

	#[actix_web::test]
	async fn check_custom_content_type() {
		async fn service(_: MsgPack<Data>) -> HttpResponse {
			HttpResponse::Ok().finish()
		}

		let payload =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);

		let mut config = MsgPackConfig::default();
		config.content_type(|mime_type| mime_type == APPLICATION_JSON);

		let app =
			init_service(App::new().app_data(config).route("/", web::post().to(service))).await;

		let request = TestRequest::default()
			.method(Method::POST)
			.set_payload(payload)
			.insert_header((header::CONTENT_TYPE, APPLICATION_JSON))
			.to_request();
		let response = call_service(&app, request).await;

		assert_eq!(response.status(), StatusCode::OK);
	}

	#[actix_web::test]
	async fn check_default_content_type() {
		async fn service(_: MsgPack<Data>) -> HttpResponse {
			HttpResponse::Ok().finish()
		}

		let payload =
			Bytes::from_static(&[0x81, 0xa7, 0x70, 0x61, 0x79, 0x6c, 0x6f, 0x61, 0x64, 0xc3]);

		let config = MsgPackConfig::default();

		let app =
			init_service(App::new().app_data(config).route("/", web::post().to(service))).await;

		let request = TestRequest::default()
			.method(Method::POST)
			.set_payload(payload)
			.insert_header((header::CONTENT_TYPE, APPLICATION_MSGPACK))
			.to_request();
		let response = call_service(&app, request).await;

		assert_eq!(response.status(), StatusCode::OK);
	}
}
