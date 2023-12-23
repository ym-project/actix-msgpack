# actix-msgpack

> [Msgpack](https://msgpack.org) payload extractor for [Actix Web](https://actix.rs).

## Installation

```bash
cargo add actix-msgpack
```

## Documentation

- [API Documentation](https://docs.rs/actix-msgpack)

## Example

```rust
use actix_msgpack::MsgPack;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Data {
    payload: String,
}
#[post("/")]
async fn index(data: MsgPack<Data>) -> impl Responder {
    println!("payload: {}", data.payload);
    HttpResponse::Ok().finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new().service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

#### You can set custom limit (default is 256kb):
```rust
use actix_msgpack::MsgPackConfig;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        let mut msgpack_config = MsgPackConfig::default();
        msgpack_config.limit(1024); // 1kb

        App::new().app_data(msgpack_config).service(index)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

#### You can use responder:

```rust
use actix_msgpack::MsgPackResponseBuilder;

#[derive(Serialize)]
struct Data {
    payload: bool,
}

#[post("/")]
async fn index() -> HttpResponse {
    let payload = Data { payload: true };
    HttpResponse::Ok().msgpack(payload)
}
```

## License

This project is licensed under of MIT license ([LICENSE](LICENSE) or [https://opensource.org/licenses/MIT](https://opensource.org/licenses/MIT))
