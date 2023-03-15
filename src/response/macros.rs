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
///     async fn update_data(&self) -> TestResponse {
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
#[macro_export]
macro_rules! response {
    ($vis:vis $name:ident = {
        $(
            $(#[doc = $doc:expr])?
            $var:ident($status:expr $(,$error:ident)?) $(=> $data:ty)?,
        )*
    }) => {
        $crate::response::macros::paste! {
            #[allow(non_camel_case_types, clippy::enum_variant_names)]
            mod [< __ $name:snake >] {
                use super::*;

                $(
                    $crate::__response__response_type!($name, $var, $($error)?, $($data)?);
                )*

                #[derive(Debug, ::poem_openapi::ApiResponse)]
                pub(super) enum $name {
                    $(
                        $(#[doc = $doc])?
                        #[oai(status = $status)]
                        [< __ $var >](::poem_openapi::payload::Json<[< __ $name __ $var >]>),
                    )*
                }

                impl $name {
                    $(
                        $crate::__response__fn!($name, $var, $($error)?, $($data)?);
                    )*
                }

                pub(super) type Response<A = ()> = $crate::response::Response<$name, A>;
            }

            $vis use [< __ $name:snake >]::$name;
            $vis use [< __ $name:snake >]::Response as [< $name Response >];
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __response__response_type {
    ($name:ident, $var:ident, , ) => {
        $crate::response::macros::paste! {
            pub type [< __ $name __ $var >] = $crate::response::macros::Empty;
        }
    };
    ($name:ident, $var:ident, , $data:ty) => {
        $crate::response::macros::paste! {
            pub type [< __ $name __ $var >] = $data;
        }
    };
    ($name:ident, $var:ident, error,) => {
        $crate::response::macros::paste! {
            $crate::static_string!(pub [< __ $name __ $var __Error >], stringify!([< $var:snake >]));
            #[derive(Debug, poem_openapi::Object)]
            pub struct [< __ $name __ $var >] {
                pub error: [< __ $name __ $var __Error >],
            }
            impl [< __ $name __ $var >] {
                pub fn new() -> Self {
                    Self {
                        error: Default::default(),
                    }
                }
            }
        }
    };
    ($name:ident, $var:ident, error, $details:ty) => {
        $crate::response::macros::paste! {
            $crate::static_string!(pub [< __ $name __ $var __Error >], stringify!([< $var:snake >]));
            #[derive(Debug, poem_openapi::Object)]
            pub struct [< __ $name __ $var >] {
                pub error: [< __ $name __ $var __Error >],
                pub details: $details,
            }
            impl [< __ $name __ $var >] {
                pub fn new(details: $details) -> Self {
                    Self {
                        error: Default::default(),
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
        $crate::response::macros::paste! {
            pub fn [< $var:snake >]<A>() -> $crate::response::Response<Self, A> {
                ::std::result::Result::Ok(
                    Self::[< __ $var >](::poem_openapi::payload::Json($crate::response::macros::Empty)).into(),
                )
            }
        }
    };
    ($name:ident, $var:ident, , $data:ty) => {
        $crate::response::macros::paste! {
            pub fn [< $var:snake >]<A>(data: $data) -> $crate::response::Response<Self, A> {
                ::std::result::Result::Ok(
                    Self::[< __ $var >](::poem_openapi::payload::Json(data)).into(),
                )
            }
        }
    };
    ($name:ident, $var:ident, error, ) => {
        $crate::response::macros::paste! {
            pub fn [< $var:snake >]<A>() -> $crate::response::Response<Self, A> {
                ::std::result::Result::Ok(
                    Self::[< __ $var >](::poem_openapi::payload::Json([< __ $name __ $var >]::new())).into(),
                )
            }
        }
    };
    ($name:ident, $var:ident, error, $details:ty) => {
        $crate::response::macros::paste! {
            pub fn [< $var:snake >]<A>(details: $details) -> $crate::response::Response<Self, A> {
                ::std::result::Result::Ok(
                    Self::[< __ $var >](::poem_openapi::payload::Json([< __ $name __ $var >]::new(details))).into(),
                )
            }
        }
    };
}

#[doc(hidden)]
#[derive(Debug, poem_openapi::Object)]
pub struct Empty;
