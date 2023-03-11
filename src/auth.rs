/// Define a custom authorization dependency based on [`poem_openapi::auth::Bearer`] that uses a
/// custom function to perform authorization.
///
/// To use this macro, you need both a tuple like struct that only contains the type for
/// a successful authorization (e.g. a struct with information about the authenticated user)
/// and a function that taks a request and a bearer token to check authorization.
///
/// #### Example
/// ```
/// use poem::Request;
/// use poem_ext::custom_auth;
/// use poem_openapi::{auth::Bearer, payload::PlainText, ApiExtractor, ApiResponse, OpenApi};
///
/// /// Contains information about the authenticated user.
/// struct User;
///
/// /// Dependency used by endpoints which require authorization.
/// struct UserAuth(User);
///
/// /// Response to return in case of unsuccessful authorization.
/// #[derive(ApiResponse)]
/// enum ErrorResponse {
///     #[oai(status = 401)]
///     Unauthorized,
///     #[oai(status = 403)]
///     Forbidden,
/// }
///
/// /// Check authorization for a given request.
/// async fn user_auth_check(_req: &Request, token: Option<Bearer>) -> Result<User, ErrorResponse> {
///     match token {
///         Some(Bearer { token }) if token == "secret_token" => Ok(User),
///         Some(_) => Err(ErrorResponse::Forbidden),
///         None => Err(ErrorResponse::Unauthorized),
///     }
/// }
///
/// // Finally use this macro to implement `ApiExtractor` on `UserAuth` so we can use it in our
/// // endpoint definitions.
/// custom_auth!(UserAuth, user_auth_check);
///
/// /// Example api with endpoint that requires authorization using `UserAuth`.
/// struct Api;
///
/// #[OpenApi]
/// impl Api {
///     #[oai(path = "/secret", method = "get")]
///     async fn secret(&self, _auth: UserAuth) -> PlainText<&'static str> {
///         // only executed if the `Authorization` header is set to `Bearer secret_token`
///         PlainText("success")
///     }
/// }
/// ```
#[macro_export]
macro_rules! custom_auth {
    ($auth:path, $checker:expr) => {
        #[poem::async_trait]
        impl<'a> poem_openapi::ApiExtractor<'a> for $auth {
            const TYPE: poem_openapi::ApiExtractorType =
                poem_openapi::ApiExtractorType::SecurityScheme;

            type ParamType = ();
            type ParamRawType = ();

            async fn from_request(
                request: &'a poem::Request,
                _body: &mut poem::RequestBody,
                _param_opts: poem_openapi::ExtractParamOptions<Self::ParamType>,
            ) -> poem::Result<Self> {
                let output =
                    <poem_openapi::auth::Bearer as poem_openapi::auth::BearerAuthorization>::from_request(request).ok();
                let checker = $checker;
                let output = checker(request, output).await?;
                Ok(Self(output))
            }

            fn register(registry: &mut poem_openapi::registry::Registry) {
                registry.create_security_scheme(
                    stringify!($auth),
                    poem_openapi::registry::MetaSecurityScheme {
                        ty: "http",
                        description: None,
                        name: None,
                        key_in: None,
                        scheme: Some("bearer"),
                        bearer_format: None,
                        flows: None,
                        openid_connect_url: None,
                    },
                );
            }

            fn security_scheme() -> Option<&'static str> {
                Some(stringify!($auth))
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use poem::Request;
    use poem_openapi::{auth::Bearer, ApiExtractor, ApiResponse};

    #[test]
    fn test_scheme_name() {
        assert_eq!(UserAuth::security_scheme(), Some("UserAuth"));
    }

    async fn check_request(authorization: Option<&str>) -> Result<UserAuth, u16> {
        let mut request = Request::builder();
        if let Some(token) = authorization {
            request = request.header("Authorization", format!("Bearer {token}"));
        }
        let request = request.finish();
        UserAuth::from_request(&request, &mut Default::default(), Default::default())
            .await
            .map_err(|err| err.into_response().status().into())
    }

    #[tokio::test]
    async fn test_missing_token() {
        assert_eq!(check_request(None).await.unwrap_err(), 401);
    }

    #[tokio::test]
    async fn test_invalid_token() {
        assert_eq!(check_request(Some("foobar")).await.unwrap_err(), 403);
    }

    #[tokio::test]
    async fn test_correct_token() {
        assert!(check_request(Some("secret_token")).await.is_ok());
    }

    #[derive(Debug)]
    struct User;

    #[derive(Debug)]
    struct UserAuth(User);

    #[derive(ApiResponse)]
    enum Response {
        #[oai(status = 401)]
        Unauthorized,
        #[oai(status = 403)]
        Forbidden,
    }

    async fn user_auth_check(_req: &Request, token: Option<Bearer>) -> Result<User, Response> {
        match token {
            Some(Bearer { token }) if token == "secret_token" => Ok(User),
            Some(_) => Err(Response::Forbidden),
            None => Err(Response::Unauthorized),
        }
    }

    custom_auth!(UserAuth, user_auth_check);
}
