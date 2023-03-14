/// Construct an OpenApi type that always evaluates to a static string that is set at compile time.
///
/// #### Example
/// ```
/// use poem_openapi::{Object, types::ToJSON};
/// use poem_ext::static_string;
///
/// static_string!(NotFoundError, "not found");
///
/// #[derive(Debug, Object)]
/// struct NotFoundDetails {
///     error: NotFoundError,
///     foobar: i32,
/// }
///
/// let response = NotFoundDetails {
///     error: Default::default(),
///     foobar: 42,
/// };
/// assert_eq!(
///     response.to_json_string(),
///     r#"{"error":"not found","foobar":42}"#
/// );
/// ```
#[macro_export]
macro_rules! static_string {
    ($name:ident, $str:expr) => {
        #[derive(Debug)]
        pub struct $name;

        impl Default for $name {
            fn default() -> Self {
                Self
            }
        }

        impl ::poem_openapi::types::Type for $name {
            const IS_REQUIRED: bool = true;

            type RawValueType = &'static str;

            type RawElementValueType = &'static str;

            fn name() -> ::std::borrow::Cow<'static, str> {
                stringify!($name).into()
            }

            fn schema_ref() -> ::poem_openapi::registry::MetaSchemaRef {
                ::poem_openapi::registry::MetaSchemaRef::Inline(Box::new(
                    ::poem_openapi::registry::MetaSchema {
                        ty: "string",
                        read_only: true,
                        default: Some($str.into()),
                        ..::poem_openapi::registry::MetaSchema::ANY
                    },
                ))
            }

            fn as_raw_value(&self) -> Option<&Self::RawValueType> {
                Some(&$str)
            }

            fn raw_element_iter<'a>(
                &'a self,
            ) -> ::std::boxed::Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
                ::std::boxed::Box::new(self.as_raw_value().into_iter())
            }
        }

        impl ::poem_openapi::types::ParseFromJSON for $name {
            fn parse_from_json(
                _value: Option<::poem_openapi::__private::serde_json::Value>,
            ) -> ::poem_openapi::types::ParseResult<Self> {
                panic!("Cannot parse static string")
            }
        }

        impl ::poem_openapi::types::ToJSON for $name {
            fn to_json(&self) -> Option<poem_openapi::__private::serde_json::Value> {
                Some(::poem_openapi::__private::serde_json::Value::String(
                    $str.into(),
                ))
            }
        }
    };
}
