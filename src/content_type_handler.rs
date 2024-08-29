use mime::Mime;
use std::sync::Arc;

pub type ContentTypeHandler = Arc<dyn Fn(Mime) -> bool + Send + Sync>;
