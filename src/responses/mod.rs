//! Improve endpoint documentation by adding response schemas to the OpenAPI spec that might be
//! returned by an [authorization dependency](crate::custom_auth!), a bad request handler or other
//! middlewares.

use std::marker::PhantomData;

use poem::IntoResponse;
use poem_openapi::{
    payload::Json,
    registry::{MetaResponse, MetaResponses, Registry},
    ApiResponse, Object,
};
use tracing::error;

use crate::static_string;

use self::merge_schemas::merge_meta_responses;

#[doc(hidden)]
pub mod macros;
mod merge_schemas;

/// Enhanced response type for registering additional response schemas for OpenAPI documentation and handling bad request errors.
///
/// Wrapping the actual return type of an endpoint in this type currently adds the following
/// response schemas to the endpoint's OpenAPI documentation:
/// 1. Anything defined by the [`MetaResponsesExt`] trait implementation of the supplied Authorization type
/// 2. The response schema for an `Unprocessable Content` error
///
/// #### Example
/// ```
/// use poem_ext::{add_response_schemas, custom_auth, responses::Response};
/// use poem_openapi::{payload::PlainText, ApiResponse, OpenApi};
///
/// struct Api;
///
/// #[OpenApi]
/// impl Api {
///     #[oai(path = "/test", method = "get")]
///     async fn test(&self) -> Response<PlainText<&'static str>> {
///         Ok(PlainText("Hello World!").into())
///     }
///
///     #[oai(path = "/test_auth", method = "get")]
///     async fn test_auth(&self, _auth: Auth) -> Response<PlainText<&'static str>, Auth> {
///         Ok(PlainText("Hello World!").into())
///     }
/// }
///
/// #[derive(ApiResponse)]
/// enum AuthError {
///     /// Unauthorized
///     #[oai(status = 401)]
///     Unauthorized,
///     /// Forbidden
///     #[oai(status = 403)]
///     Forbidden,
/// }
///
/// struct Auth(());
/// add_response_schemas!(Auth, AuthError);
/// # async fn auth_checker(_req: &poem::Request, _token: Option<poem_openapi::auth::Bearer>) -> Result<(), AuthError> { Ok(()) }
/// custom_auth!(Auth, auth_checker);
/// ```
pub type Response<T, A = ()> = poem::Result<InnerResponse<T, A>>;

#[doc(hidden)]
#[derive(Debug)]
pub struct InnerResponse<T, A>(InnerResponseData<T, A>);

#[derive(Debug)]
enum InnerResponseData<T, A> {
    Ok { value: T, _auth: PhantomData<A> },
    BadRequest { error: poem::Error },
}

impl<T, A> From<T> for InnerResponse<T, A> {
    fn from(value: T) -> Self {
        Self(InnerResponseData::Ok {
            value,
            _auth: PhantomData,
        })
    }
}

/// Construct an internal server error response and log the error.
///
/// #### Example
/// ```
/// use poem_ext::{response, responses::{internal_server_error, Response}};
/// use poem_openapi::{OpenApi, payload::PlainText};
///
/// struct Api;
///
/// #[OpenApi]
/// impl Api {
///     #[oai(path = "/test", method = "get")]
///     async fn test(&self) -> Test::Response {
///         fallible_function().map_err(internal_server_error)?;
///         Test::ok("Hello World!")
///     }
/// }
///
/// response!(Test = {
///     Ok(200) => &'static str,
/// });
/// # fn fallible_function() -> Result<(), &'static str> { todo!() }
/// ```
pub fn internal_server_error<E>(error: E) -> ErrorResponse
where
    E: std::fmt::Display,
{
    error!("{error}");
    ErrorResponse::InternalServerError(Json(InternalServerError {
        error: InternalServerErrorText,
    }))
}

static_string!(UnprocessableContentText, "unprocessable_content");
static_string!(InternalServerErrorText, "internal_server_error");

#[doc(hidden)]
#[derive(Debug, Object)]
pub struct BadRequestError {
    error: UnprocessableContentText,
    reason: String,
}

#[doc(hidden)]
#[derive(Debug, Object)]
pub struct InternalServerError {
    error: InternalServerErrorText,
}

#[doc(hidden)]
#[derive(Debug, ApiResponse)]
pub enum ErrorResponse {
    /// Unprocessable Content
    #[oai(status = 422)]
    UnprocessableContent(Json<BadRequestError>),
    /// Internal Server Error
    #[oai(status = 500)]
    InternalServerError(Json<InternalServerError>),
}

impl<T, A> ApiResponse for InnerResponse<T, A>
where
    T: ApiResponse,
    A: MetaResponsesExt,
{
    const BAD_REQUEST_HANDLER: bool = true;

    fn meta() -> MetaResponses {
        MetaResponses {
            responses: merge_meta_responses(
                T::meta()
                    .responses
                    .into_iter()
                    .chain(A::responses())
                    .chain(ErrorResponse::meta().responses),
            ),
        }
    }

    fn register(registry: &mut Registry) {
        T::register(registry);
        A::register(registry);
        ErrorResponse::register(registry);
    }

    fn from_parse_request_error(error: poem::Error) -> Self {
        Self(InnerResponseData::BadRequest { error })
    }
}

impl<T, A> IntoResponse for InnerResponse<T, A>
where
    A: Send,
    T: IntoResponse,
{
    fn into_response(self) -> poem::Response {
        match self.0 {
            InnerResponseData::Ok { value, _auth } => value.into_response(),
            InnerResponseData::BadRequest { error } => {
                if error.status() == 400 {
                    ErrorResponse::UnprocessableContent(Json(BadRequestError {
                        error: UnprocessableContentText,
                        reason: error.to_string(),
                    }))
                    .into_response()
                } else {
                    error.into_response()
                }
            }
        }
    }
}

/// Trait for adding additional response schemas using the [`Response`] type.
///
/// The easiest way to implement this trait for a type is to use the [`add_response_schemas!`](crate::add_response_schemas!) macro.
pub trait MetaResponsesExt {
    /// Iterator type for [`Self::responses()`] return value
    type Iter: IntoIterator<Item = MetaResponse>;
    /// Return an iterable of endpoint schemas.
    fn responses() -> Self::Iter;
    /// Register any child response schemas.
    fn register(registry: &mut Registry);
}

/// Implement [`MetaResponsesExt`] for a type to add additional response schemas to an endpoint by using the [`Response`] type.
///
/// #### Example
/// ```
/// use poem_ext::add_response_schemas;
/// use poem_openapi::ApiResponse;
///
/// #[derive(ApiResponse)]
/// enum AuthError {
///     /// Unauthorized
///     #[oai(status = 401)]
///     Unauthorized,
///     /// Forbidden
///     #[oai(status = 403)]
///     Forbidden,
/// }
///
/// #[derive(ApiResponse)]
/// enum OtherError {
///     /// Foo
///     #[oai(status = 404)]
///     Foo,
///     /// Bar
///     #[oai(status = 418)]
///     Bar,
/// }
///
/// struct Auth;
///
/// add_response_schemas!(Auth, AuthError, OtherError);
/// ```
///
/// Endpoints that return a [`Response<T, Auth>`] will now additionally list all `AuthError` and `OtherError` variants in their OpenAPI documentation.
#[macro_export]
macro_rules! add_response_schemas {
    ($type:ty) => {$crate::add_response_schemas!($type,);};
    ($type:ty, $($responses:ty),*) => {
        impl $crate::responses::MetaResponsesExt for $type {
            type Iter = ::std::vec::Vec<::poem_openapi::registry::MetaResponse>;
            fn responses() -> Self::Iter {
                ::std::iter::empty()
                    $(.chain(<$responses as ::poem_openapi::ApiResponse>::meta().responses))*
                .collect()
            }
            #[allow(unused_variables)]
            fn register(registry: &mut ::poem_openapi::registry::Registry) {
                $(
                    <$responses as ::poem_openapi::ApiResponse>::register(registry);
                )*
            }
        }
    };
}

// Implement `MetaResponsesExt` on unit, so we can use it as a default for the `A` type parameter in `Response`.
add_response_schemas!(());

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[test]
    fn test_response_schemas() {
        let mut responses = Response::<EndpointResponse, Auth>::meta()
            .responses
            .into_iter()
            .sorted_by_key(|e| e.status);
        let mut check = |s, d| {
            let resp = responses.next().unwrap();
            assert_eq!(resp.status, Some(s));
            assert_eq!(resp.description, d);
        };
        check(200, "Ok");
        check(401, "Unauthorized");
        check(403, "Forbidden");
        check(404, "There are multiple possible responses with this status code:\n- FooNotFound\n- BarNotFound");
        check(422, "Unprocessable Content");
        check(500, "Internal Server Error");
        assert!(responses.next().is_none());
    }

    struct Auth;

    #[allow(dead_code)]
    #[derive(ApiResponse)]
    enum EndpointResponse {
        /// Ok
        #[oai(status = 200)]
        Ok,
        /// FooNotFound
        #[oai(status = 404)]
        FooNotFound,
    }

    #[allow(dead_code)]
    #[derive(ApiResponse)]
    enum AuthError {
        /// Unauthorized
        #[oai(status = 401)]
        Unauthorized,
        /// Forbidden
        #[oai(status = 403)]
        Forbidden,
        /// BarNotFound
        #[oai(status = 404)]
        BarNotFound,
    }

    add_response_schemas!(Auth, AuthError);
}
