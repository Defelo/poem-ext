/// Define a custom authorization dependency based on
/// [`poem_openapi::auth::Bearer`] that uses a custom function to perform
/// authorization.
///
/// To use this macro, you need both a tuple like struct that only contains the
/// type for a successful authorization (e.g. a struct with information about
/// the authenticated user) and a function that taks a request and a bearer
/// token to check authorization.
///
/// #### Example
/// ```
/// use poem::Request;
/// use poem_ext::{custom_auth, response};
/// use poem_openapi::{auth::Bearer, payload::PlainText, ApiExtractor, ApiResponse, OpenApi};
///
/// /// Contains information about the authenticated user.
/// struct User;
///
/// /// Dependency used by endpoints which require authorization.
/// struct UserAuth(User);
///
/// /// Response to return in case of unsuccessful authorization.
/// response!(AuthResult = {
///     /// The user is unauthenticated.
///     Unauthorized(401, error),
///     /// The authenticated user is not allowed to perform this action.
///     Forbidden(403, error),
/// });
///
/// /// Check authorization for a given request.
/// async fn user_auth_check(
///     _req: &Request,
///     token: Option<Bearer>,
/// ) -> Result<User, AuthResult::raw::Response> {
///     match token {
///         Some(Bearer { token }) if token == "secret_token" => Ok(User),
///         Some(_) => Err(AuthResult::raw::forbidden()),
///         None => Err(AuthResult::raw::unauthorized()),
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
        #[::poem::async_trait]
        impl<'a> ::poem_openapi::ApiExtractor<'a> for $auth {
            const TYPES: &'static [::poem_openapi::ApiExtractorType] =
                &[::poem_openapi::ApiExtractorType::SecurityScheme];

            type ParamType = ();
            type ParamRawType = ();

            async fn from_request(
                request: &'a ::poem::Request,
                _body: &mut ::poem::RequestBody,
                _param_opts: ::poem_openapi::ExtractParamOptions<Self::ParamType>,
            ) -> ::poem::Result<Self> {
                let output =
                    <::poem_openapi::auth::Bearer as ::poem_openapi::auth::BearerAuthorization>::from_request(request).ok();
                let checker = $checker;
                let output = checker(request, output).await?;
                ::std::result::Result::Ok(Self(output))
            }

            fn register(registry: &mut ::poem_openapi::registry::Registry) {
                registry.create_security_scheme(
                    ::std::stringify!($auth),
                    ::poem_openapi::registry::MetaSecurityScheme {
                        ty: "http",
                        description: ::std::option::Option::None,
                        name: ::std::option::Option::None,
                        key_in: ::std::option::Option::None,
                        scheme: ::std::option::Option::Some("bearer"),
                        bearer_format: ::std::option::Option::None,
                        flows: ::std::option::Option::None,
                        openid_connect_url: ::std::option::Option::None,
                    },
                );
            }

            fn security_schemes() -> ::std::vec::Vec<&'static str> {
                ::std::vec![::std::stringify!($auth)]
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use poem::Request;
    use poem_openapi::{auth::Bearer, ApiExtractor};

    use crate::response;

    #[test]
    fn test_scheme_name() {
        assert_eq!(UserAuth::security_schemes(), vec!["UserAuth"]);
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

    response!(UserAuthResult = {
        Unauthorized(401, error),
        Forbidden(403, error),
    });

    async fn user_auth_check(
        _req: &Request,
        token: Option<Bearer>,
    ) -> Result<User, UserAuthResult::raw::Response> {
        match token {
            Some(Bearer { token }) if token == "secret_token" => Ok(User),
            Some(_) => Err(UserAuthResult::raw::forbidden()),
            None => Err(UserAuthResult::raw::unauthorized()),
        }
    }

    custom_auth!(UserAuth, user_auth_check);
}
