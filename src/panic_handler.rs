//! Contains a middlware that automatically responds with an internal server
//! error whenever the current thread is panicking.
//!
//! #### Example
//! ```
//! use poem::{middleware::CatchPanic, EndpointExt, Route};
//! use poem_ext::panic_handler::PanicHandler;
//! use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
//!
//! struct Api;
//!
//! #[OpenApi]
//! impl Api {
//!     #[oai(path = "/test", method = "get")]
//!     async fn test(&self) -> PlainText<&'static str> {
//!         // status = 500, content = {"error":"internal_server_error"}
//!         panic!("at the disco")
//!     }
//! }
//!
//! let api_service = OpenApiService::new(Api, "Test", "0.1.0");
//! let app = Route::new()
//!     .nest("/", api_service)
//!     .with(PanicHandler::middleware());
//! ```

use poem::middleware::CatchPanic;

use crate::responses::{make_internal_server_error, ErrorResponse};

/// Custom panic handler.
#[derive(Debug, Clone)]
pub struct PanicHandler;

impl PanicHandler {
    /// Creates a [`CatchPanic`] middlware that uses this panic handler.
    pub fn middleware() -> CatchPanic<Self> {
        CatchPanic::new().with_handler(Self)
    }
}

impl poem::middleware::PanicHandler for PanicHandler {
    type Response = ErrorResponse;

    fn get_response(&self, _err: Box<dyn std::any::Any + Send + 'static>) -> Self::Response {
        make_internal_server_error()
    }
}
