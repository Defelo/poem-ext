//! Contains the [`PatchValue`] enum that can be used in `PATCH` endpoints to distinguish between
//! values that should be updated and those that should remain unchanged.
//!
//! #### Example
//! ```
//! use poem_ext::{patch_value::PatchValue, responses::internal_server_error};
//! use poem_openapi::{param::Path, payload::Json, Object, OpenApi};
//! use sea_orm::{ActiveModelTrait, DatabaseConnection, EntityTrait, Unchanged};
//!
//! struct Api {
//!     db: DatabaseConnection,
//! }
//!
//! #[OpenApi]
//! impl Api {
//!     #[oai(path = "/user/:user_id", method = "patch")]
//!     async fn update_user(
//!         &self,
//!         user_id: Path<i32>,
//!         data: Json<UpdateUserRequest>,
//!     ) -> UpdateUser::Response {
//!         let Some(user) = users::Entity::find_by_id(user_id.0)
//!             .one(&self.db)
//!             .await?
//!             else { return UpdateUser::not_found(); };
//!
//!         users::ActiveModel {
//!             id: Unchanged(user.id),
//!             name: data.0.name.update(user.name),
//!             password: data.0.password.update(user.password),
//!         }
//!         .update(&self.db)
//!         .await?;
//!
//!         UpdateUser::ok()
//!     }
//! }
//!
//! #[derive(Debug, Object)]
//! pub struct UpdateUserRequest {
//!     #[oai(validator(max_length = 255))]
//!     pub name: PatchValue<String>,
//!     #[oai(validator(max_length = 255))]
//!     pub password: PatchValue<String>,
//! }
//! #
//! # poem_ext::response!(UpdateUser = {
//! #     Ok(200),
//! #     NotFound(404),
//! # });
//! # mod users {
//! #     use sea_orm::entity::prelude::*;
//! #
//! #     #[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
//! #     #[sea_orm(table_name = "users")]
//! #     pub struct Model {
//! #         #[sea_orm(primary_key, auto_increment = false)]
//! #         pub id: i32,
//! #         #[sea_orm(column_type = "Text")]
//! #         pub name: String,
//! #         #[sea_orm(column_type = "Text")]
//! #         pub password: String,
//! #     }
//! #
//! #     #[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
//! #     pub enum Relation {}
//! #
//! #     impl ActiveModelBehavior for ActiveModel {}
//! # }
//! ```

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

    /// Return the new value if this is [`Set(T)`](Self::Unchanged) or the old value if [`Unchanged`](Self::Unchanged).
    pub fn get_new<'a>(&'a self, old: &'a T) -> &'a T {
        match self {
            Self::Set(x) => x,
            Self::Unchanged => old,
        }
    }

    /// Convert a [`PatchValue<T>`] to a [`PatchValue<U>`].
    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> PatchValue<U> {
        match self {
            PatchValue::Set(x) => PatchValue::Set(f(x)),
            PatchValue::Unchanged => PatchValue::Unchanged,
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

impl<T> Default for PatchValue<T> {
    fn default() -> Self {
        Self::Unchanged
    }
}
