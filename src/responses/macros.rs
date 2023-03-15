#[doc(hidden)]
pub use paste::paste;

/// Construct an [`ApiResponse`](derive@poem_openapi::ApiResponse) enum with some helper functions to
/// easily create both success and error responses.
///
/// #### Example
/// ```
/// use poem_ext::response;
/// use poem_openapi::{Object, OpenApi};
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
///             _ => unimplemented!(),
///         }
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
/// # pub type Data = ();
/// # pub type ConflictDetails = ();
/// # pub enum ApiResponseEnum {}
///     pub type Response = poem::Result<ApiResponseEnum>;
///
/// # extern {
///     pub fn ok(data: Data) -> Response;
///     pub fn created() -> Response;
///     pub fn conflict(teapot: ConflictDetails) -> Response;
///     pub fn teapot() -> Response;
/// # }
/// }
/// ```
#[macro_export]
macro_rules! response {
    ($vis:vis $name:ident = {
        $(
            $(#[doc = $doc:expr])?
            $var:ident($status:expr $(,$error:ident)?) $(=> $data:ty)?,
        )*
    }) => {
        $crate::responses::macros::paste! {
            #[allow(non_snake_case, non_camel_case_types, clippy::enum_variant_names)]
            $vis mod $name {
                use super::*;

                mod __inner {
                    use super::*;

                    $(
                        $crate::__response__response_type!($name, $var, $($error)?, $($data)?);
                    )*

                    #[derive(::std::fmt::Debug, ::poem_openapi::ApiResponse)]
                    pub enum $name {
                        $(
                            $(#[doc = $doc])?
                            #[oai(status = $status)]
                            [< __ $var >](::poem_openapi::payload::Json<[< __ $name __ $var >]>),
                        )*
                    }
                }

                pub type Response<A = ()> = $crate::responses::Response<self::__inner::$name, A>;
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
            #[derive(::std::fmt::Debug, ::poem_openapi::Object)]
            pub struct [< __ $name __ $var >] {
                pub error: [< __ $name __ $var __Error >],
            }
            impl [< __ $name __ $var >] {
                pub fn new() -> Self {
                    Self {
                        error: ::std::default::Default::default(),
                    }
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
macro_rules! __response__fn {
    ($name:ident, $var:ident, , ) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]<A>() -> $crate::responses::Response<self::__inner::$name, A> {
                ::std::result::Result::Ok(
                    self::__inner::$name::[< __ $var >](::poem_openapi::payload::Json($crate::responses::macros::Empty)).into(),
                )
            }
        }
    };
    ($name:ident, $var:ident, , $data:ty) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]<A>(data: $data) -> $crate::responses::Response<self::__inner::$name, A> {
                ::std::result::Result::Ok(
                    self::__inner::$name::[< __ $var >](::poem_openapi::payload::Json(data)).into(),
                )
            }
        }
    };
    ($name:ident, $var:ident, error, ) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]<A>() -> $crate::responses::Response<self::__inner::$name, A> {
                ::std::result::Result::Ok(
                    self::__inner::$name::[< __ $var >](::poem_openapi::payload::Json(self::__inner::[< __ $name __ $var >]::new())).into(),
                )
            }
        }
    };
    ($name:ident, $var:ident, error, $details:ty) => {
        $crate::responses::macros::paste! {
            pub fn [< $var:snake >]<A>(details: $details) -> $crate::responses::Response<self::__inner::$name, A> {
                ::std::result::Result::Ok(
                    self::__inner::$name::[< __ $var >](::poem_openapi::payload::Json(self::__inner::[< __ $name __ $var >]::new(details))).into(),
                )
            }
        }
    };
}

#[doc(hidden)]
#[derive(Debug, poem_openapi::Object)]
pub struct Empty;
