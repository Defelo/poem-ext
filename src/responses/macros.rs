#[doc(hidden)]
pub use paste::paste;

/// Construct an [`ApiResponse`](derive@poem_openapi::ApiResponse) enum with some helper functions to
/// easily create both success and error responses.
///
/// #### Example
/// ```
/// use poem_ext::response;
/// use poem_openapi::{ApiResponse, Object, OpenApi};
///
/// # fn main() {
/// struct Api;
///
/// #[OpenApi]
/// impl Api {
///     #[oai(path = "/test", method = "put")]
///     async fn update_data(&self) -> Test::Response {
///         match todo!() {
///             // status = 200, content = {"foo": 42, "bar": "Hello World!"}
///             0 => Test::ok(Data { foo: 42, bar: "Hello World!".into() }),
///             // status = 201, content = {}
///             1 => Test::created(),
///             // status = 409, content = {"error": "conflict", "details": {"test": true}}
///             2 => Test::conflict(ConflictDetails { test: true }),
///             // status = 418, content = {"error": "teapot"}
///             3 => Test::teapot(),
///             // status = 402
///             4 => Ok(OtherResponse::PaymentRequired.into()),
///             _ => unimplemented!(),
///         }
///     }
///
///     #[oai(path = "/test", method = "put")]
///     async fn update_data_raw(&self) -> poem_ext::responses::Response<Test::raw::Response> {
///         Ok(match todo!() {
///             // status = 200, content = {"foo": 42, "bar": "Hello World!"}
///             0 => Test::raw::ok(Data { foo: 42, bar: "Hello World!".into() }),
///             // status = 201, content = {}
///             1 => Test::raw::created(),
///             // status = 409, content = {"error": "conflict", "details": {"test": true}}
///             2 => Test::raw::conflict(ConflictDetails { test: true }),
///             // status = 418, content = {"error": "teapot"}
///             3 => Test::raw::teapot(),
///             // status = 403
///             5 => OtherResponse::Forbidden.into(),
///             _ => unimplemented!(),
///         }.into())
///     }
/// }
///
/// response!(Test = {
///     /// Data found
///     Ok(200) => Data,
///     /// Data has been created
///     Created(201),
///     /// Data conflicts with stuff
///     Conflict(409, error) => ConflictDetails,
///     /// I'm a teapot
///     Teapot(418, error),
///     ..OtherResponse, // include OtherResponse and add From<OtherResponse> impls
/// });
/// # }
///
/// #[derive(Debug, Object)]
/// pub struct Data {
///     foo: i32,
///     bar: String,
/// }
///
/// #[derive(Debug, Object)]
/// pub struct ConflictDetails {
///     test: bool,
/// }
///
/// #[derive(Debug, ApiResponse)]
/// pub enum OtherResponse {
///     #[oai(status = 402)]
///     PaymentRequired,
///     #[oai(status = 403)]
///     Forbidden,
/// }
/// ```
///
/// The `response!` macro in this example expands to a module with the specified name (`Test` in
/// this case) that contains:
/// 1. A `Response` type that you can return directly from your endpoint. This is basically a
///    `poem::Result<ApiResponseEnum>` where `ApiResponseEnum` is an enum you could define using
///    the [`ApiResponse`](derive@poem_openapi::ApiResponse) derive macro that contains all the
///    variants you specified in the macro invocation (`Ok`, `Created`, `Conflict`, `Teapot` in
///    this example).
/// 2. For each variant a function that constructs a response which can be directly returned from
///    your endpoint. The function name is always the snake_case version of the variant's name.
///    If the variant contains data or error details (like `Ok` and `Conflict` in this example),
///    this function accepts exactly one parameter with the specified type.
///
/// The signature of the generated module for this example would look roughly like this:
/// ```
/// mod Test {
/// #   pub type Data = ();
/// #   pub type ConflictDetails = ();
/// #   pub enum ApiResponseEnum {}
///     type Response<A> = poem_ext::responses::Response<raw::Response, A>;
///
/// #   trait _1 {
///     fn ok<A>(data: Data) -> Response<A>;
///     fn created<A>() -> Response<A>;
///     fn conflict<A>(teapot: ConflictDetails) -> Response<A>;
///     fn teapot<A>() -> Response<A>;
/// #   }
///
///     pub mod raw {
/// #       use super::*;
/// #       pub
///         type Response = ApiResponseEnum;
///
/// #       trait _2 {
///         fn ok(data: Data) -> Response;
///         fn created() -> Response;
///         fn conflict(teapot: ConflictDetails) -> Response;
///         fn teapot() -> Response;
/// #       }
///     }
/// }
/// ```
#[macro_export]
macro_rules! response {
    ($vis:vis $name:ident = {
        $(
            $(#[doc = $doc:literal])?
            $var:ident($status:expr $(,$error:ident)?) $(=> $data:ty)?,
        )*
        $(
            ..$($include:ident)::+,
        )*
    }) => {
        $crate::responses::macros::paste! {
            #[allow(dead_code, unused, non_snake_case, non_camel_case_types, clippy::enum_variant_names)]
            $vis mod $name {
                use super::*;

                mod __inner {
                    use super::*;

                    $(
                        $crate::__response__response_type!($name, $var, $($error)?, $($data)?);
                    )*

                    #[derive(::std::fmt::Debug)]
                    pub enum $name {
                        $(
                            $(#[doc = $doc])?
                            $var(::poem_openapi::payload::Json<[< __ $name __ $var >]>),
                        )*
                        $(
                            [< __Include__ $($include)__+ >]($($include)::+),
                        )*
                    }

                    impl ::poem_openapi::__private::poem::IntoResponse for $name {
                        fn into_response(self) -> ::poem_openapi::__private::poem::Response {
                            match self {
                                $(
                                    Self::$var(media) => {
                                        let mut resp = ::poem_openapi::__private::poem::IntoResponse::into_response(media);
                                        resp.set_status(poem_openapi::__private::poem::http::StatusCode::from_u16($status).unwrap());
                                        resp
                                    }
                                )*
                                $(
                                    Self::[< __Include__ $($include)__+ >](inner) => ::poem_openapi::__private::poem::IntoResponse::into_response(inner),
                                )*
                            }
                        }
                    }

                    impl ::poem_openapi::ApiResponse for $name {
                        const BAD_REQUEST_HANDLER: bool = false;
                        fn meta() -> ::poem_openapi::registry::MetaResponses {
                            ::poem_openapi::registry::MetaResponses {
                                responses: vec![
                                    $(
                                        ::poem_openapi::registry::MetaResponse {
                                            description: {
                                                let mut description = "";
                                                $(description = $doc;)?
                                                description
                                            },
                                            status: ::std::option::Option::Some($status),
                                            content: <::poem_openapi::payload::Json<[< __ $name __ $var >]> as ::poem_openapi::ResponseContent>::media_types(),
                                            headers: vec![],
                                        },
                                    )*
                                ]
                                .into_iter()
                                $(
                                    .chain(<$($include)::+ as ::poem_openapi::ApiResponse>::meta().responses)
                                )*
                                .collect()
                            }
                        }
                        fn register(registry: &mut ::poem_openapi::registry::Registry) {
                            $(
                                <::poem_openapi::payload::Json<[< __ $name __ $var >]> as ::poem_openapi::ResponseContent>::register(registry);
                            )*
                            $(
                                <$($include)::+ as ::poem_openapi::ApiResponse>::register(registry);
                            )*
                        }
                    }

                    impl ::std::convert::From<$name> for ::poem_openapi::__private::poem::Error {
                        fn from(resp: $name) -> ::poem_openapi::__private::poem::Error {
                            use ::poem_openapi::__private::poem::IntoResponse;
                            let error_msg: ::std::option::Option<&str> = match resp {
                                $(
                                    $name::$var(_) => ::std::option::Option::Some({
                                        let mut description = "";
                                        $(description = $doc;)?
                                        description
                                    }),
                                )*
                                $(
                                    $name::[< __Include__ $($include)__+ >](inner) => return ::poem_openapi::__private::poem::Error::from(inner),
                                )*
                            };
                            let mut err = ::poem_openapi::__private::poem::Error::from_response(
                                resp.into_response(),
                            );
                            if let ::std::option::Option::Some(error_msg) = error_msg {
                                err.set_error_message(error_msg);
                            }
                            err
                        }
                    }

                    $(
                        impl ::std::convert::From<$($include)::+> for $name {
                            fn from(value: $($include)::+) -> Self {
                                Self::[< __Include__ $($include)__+ >](value)
                            }
                        }
                        impl<A> ::std::convert::From<$($include)::+> for $crate::responses::InnerResponse<$name, A> {
                            fn from(value: $($include)::+) -> Self {
                                $name::[< __Include__ $($include)__+ >](value).into()
                            }
                        }
                    )*
                }

                pub mod raw {
                    use super::*;

                    pub type Response = super::__inner::$name;
                    $(
                        $crate::__response__raw_fn!($name, $var, $($error)?, $($data)?);
                    )*
                }

                pub type Response<A = ()> = $crate::responses::Response<self::raw::Response, A>;

                $(
                    $crate::__response__fn!($name, $var, $($error)?, $($data)?);
                )*
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __response__response_type {
    ($name:ident, $var:ident, , ) => {
        $crate::responses::macros::paste! {
            pub type [< __ $name __ $var >] = $crate::responses::macros::Empty;
        }
    };
    ($name:ident, $var:ident, , $data:ty) => {
        $crate::responses::macros::paste! {
            pub type [< __ $name __ $var >] = $data;
        }
    };
    ($name:ident, $var:ident, error,) => {
        $crate::responses::macros::paste! {
            $crate::static_string!(pub [< __ $name __ $var __Error >], ::std::stringify!([< $var:snake >]));
            #[derive(::std::fmt::Debug, ::std::default::Default, ::poem_openapi::Object)]
            pub struct [< __ $name __ $var >] {
                pub error: [< __ $name __ $var __Error >],
            }
            impl [< __ $name __ $var >] {
                pub fn new() -> Self {
                    Self::default()
                }
            }
        }
    };
    ($name:ident, $var:ident, error, $details:ty) => {
        $crate::responses::macros::paste! {
            $crate::static_string!(pub [< __ $name __ $var __Error >], ::std::stringify!([< $var:snake >]));
            #[derive(::std::fmt::Debug, ::poem_openapi::Object)]
            pub struct [< __ $name __ $var >] {
                pub error: [< __ $name __ $var __Error >],
                pub details: $details,
            }
            impl [< __ $name __ $var >] {
                pub fn new(details: $details) -> Self {
                    Self {
                        error: ::std::default::Default::default(),
                        details,
                    }
                }
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __response__raw_fn {
    ($name:ident, $var:ident, , ) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]() -> Response {
                Response::$var(::poem_openapi::payload::Json($crate::responses::macros::Empty))
            }
        }
    };
    ($name:ident, $var:ident, , $data:ty) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >](data: $data) -> Response {
                Response::$var(::poem_openapi::payload::Json(data))
            }
        }
    };
    ($name:ident, $var:ident, error, ) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]() -> Response {
                Response::$var(::poem_openapi::payload::Json(super::__inner::[< __ $name __ $var >]::new()))
            }
        }
    };
    ($name:ident, $var:ident, error, $details:ty) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >](details: $details) -> Response {
                Response::$var(::poem_openapi::payload::Json(super::__inner::[< __ $name __ $var >]::new(details)))
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __response__fn {
    ($name:ident, $var:ident, , ) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]<A>() -> Response<A> {
                ::std::result::Result::Ok(self::raw::[< $var:snake >]().into())
            }
        }
    };
    ($name:ident, $var:ident, , $data:ty) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]<A>(data: $data) -> Response<A> {
                ::std::result::Result::Ok(self::raw::[< $var:snake >](data).into())
            }
        }
    };
    ($name:ident, $var:ident, error, ) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]<A>() -> Response<A> {
                ::std::result::Result::Ok(self::raw::[< $var:snake >]().into())
            }
        }
    };
    ($name:ident, $var:ident, error, $details:ty) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]<A>(details: $details) -> Response<A> {
                ::std::result::Result::Ok(self::raw::[< $var:snake >](details).into())
            }
        }
    };
}

#[doc(hidden)]
#[derive(Debug, poem_openapi::Object)]
pub struct Empty;
