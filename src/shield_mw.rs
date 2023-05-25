//! Contains a middleware that prevents endpoint handlers from being canceled if the connection is closed.

use std::sync::Arc;

use poem::{Endpoint, Middleware};
use tokio_shield::Shield;

/// Prevent endpoint handlers from being canceled.
///
/// #### Example
/// ```no_run
/// use poem_ext::shield_mw::shield;
/// use poem_openapi::OpenApi;
/// use std::time::Duration;
///
/// struct Api;
///
/// #[OpenApi]
/// impl Api {
///     #[oai(path = "/test", method = "get", transform = "shield")]
///     async fn test(&self) {
///         tokio::time::sleep(Duration::from_secs(2)).await;
///         println!("test"); // will always run, even if connection is closed before.
///     }
/// }
/// ````
pub fn shield<E: Endpoint + 'static>(ep: E) -> ShieldEndpoint<E> {
    ShieldEndpoint(Arc::new(ep))
}

/// Prevent endpoint handlers from being canceled.
///
/// #### Example
/// ```rust
/// use poem::{EndpointExt, Route};
/// use poem_ext::shield_mw::ShieldMiddleware;
/// use poem_openapi::{OpenApi, OpenApiService};
/// use std::time::Duration;
///
/// struct Api;
///
/// #[OpenApi]
/// impl Api {
///     #[oai(path = "/test", method = "get")]
///     async fn test(&self) {
///         tokio::time::sleep(Duration::from_secs(2)).await;
///         println!("test"); // will always run, even if connection is closed before.
///     }
/// }
///
/// let api_service = OpenApiService::new(Api, "Test", "0.1.0");
/// let app = Route::new().nest("/", api_service).with(ShieldMiddleware);
/// ```
#[derive(Debug, Clone)]
pub struct ShieldMiddleware;

impl<E: Endpoint + 'static> Middleware<E> for ShieldMiddleware {
    type Output = ShieldEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        shield(ep)
    }
}

#[doc(hidden)]
#[derive(Debug)]
pub struct ShieldEndpoint<E>(Arc<E>);

#[poem::async_trait]
impl<E: Endpoint + 'static> Endpoint for ShieldEndpoint<E> {
    type Output = E::Output;

    async fn call(&self, req: poem::Request) -> poem::Result<Self::Output> {
        let ep = Arc::clone(&self.0);
        async move { ep.call(req).await }.shield().await
    }
}
