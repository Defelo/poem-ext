//! Contains the [`PatchValue`] enum that can be used in `PATCH` endpoints to distinguish between
//! values that should be updated and those that should remain unchanged.

use std::borrow::Cow;

use poem_openapi::{
    registry::MetaSchemaRef,
    types::{ParseFromJSON, ParseResult, ToJSON, Type},
};
#[cfg(feature = "sea-orm")]
use sea_orm::ActiveValue;

/// Can be used as a parameter in `PATCH` endpoints to distinguish between values that should
/// be updated and those that should remain unchanged.
#[derive(Debug, Clone, Copy)]
pub enum PatchValue<T> {
    /// Update the value to the contained `T`.
    Set(T),
    /// Don't change the value.
    Unchanged,
}

impl<T> PatchValue<T> {
    /// Convert this type to a [`sea_orm::ActiveValue`] that can be used to construct an `ActiveModel`.
    #[cfg(feature = "sea-orm")]
    pub fn update(self, old: T) -> ActiveValue<T>
    where
        T: Into<sea_orm::Value>,
    {
        match self {
            Self::Set(x) => ActiveValue::Set(x),
            Self::Unchanged => ActiveValue::Unchanged(old),
        }
    }
}

impl<T> ParseFromJSON for PatchValue<T>
where
    T: ParseFromJSON,
{
    fn parse_from_json(
        value: Option<poem_openapi::__private::serde_json::Value>,
    ) -> ParseResult<Self> {
        match Option::<T>::parse_from_json(value) {
            Ok(Some(x)) => Ok(Self::Set(x)),
            Ok(None) => Ok(Self::Unchanged),
            Err(x) => Err(x.propagate()),
        }
    }
}

impl<T> ToJSON for PatchValue<T>
where
    T: ToJSON,
{
    fn to_json(&self) -> Option<poem_openapi::__private::serde_json::Value> {
        match self {
            Self::Set(x) => Some(x),
            Self::Unchanged => None,
        }
        .to_json()
    }
}

impl<T> Type for PatchValue<T>
where
    T: Type,
{
    const IS_REQUIRED: bool = false; // default to unchanged

    type RawValueType = T::RawValueType;

    type RawElementValueType = T::RawElementValueType;

    fn name() -> Cow<'static, str> {
        format!("optional<{}>", T::name()).into()
    }

    fn schema_ref() -> MetaSchemaRef {
        T::schema_ref()
    }

    fn as_raw_value(&self) -> Option<&Self::RawValueType> {
        match self {
            Self::Set(value) => value.as_raw_value(),
            Self::Unchanged => None,
        }
    }

    fn raw_element_iter<'a>(
        &'a self,
    ) -> Box<dyn Iterator<Item = &'a Self::RawElementValueType> + 'a> {
        match self {
            Self::Set(value) => value.raw_element_iter(),
            Self::Unchanged => Box::new(std::iter::empty()),
        }
    }
}
