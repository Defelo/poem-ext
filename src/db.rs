//! Contains a middleware that automatically creates and manages a
//! [`sea_orm::DatabaseTransaction`](sea_orm::DatabaseTransaction) for each
//! incoming request. The transaction is automatically
//! [`commit()`](sea_orm::DatabaseTransaction::commit)ed if the endpoint returns
//! a successful response or
//! [`rollback()`](sea_orm::DatabaseTransaction::rollback)ed in case of an
//! error.
//!
//! #### Example
//! ```no_run
//! use poem::{web::Data, EndpointExt, Route};
//! use poem_ext::db::{DbTransactionMiddleware, DbTxn};
//! use poem_openapi::{payload::PlainText, OpenApi, OpenApiService};
//! use sea_orm::DatabaseTransaction;
//!
//! struct Api;
//!
//! #[OpenApi]
//! impl Api {
//!     #[oai(path = "/test", method = "get")]
//!     async fn test(&self, txn: Data<&DbTxn>) -> PlainText<&'static str> {
//!         let txn: &DatabaseTransaction = &txn;
//!         todo!()
//!     }
//! }
//!
//! # let db_connection = todo!();
//! let api_service = OpenApiService::new(Api, "test", "0.1.0");
//! let app = Route::new()
//!     .nest("/", api_service)
//!     .with(DbTransactionMiddleware::new(db_connection));
//! ```

use std::{fmt::Debug, sync::Arc};

use poem::{async_trait, Endpoint, IntoResponse, Middleware, Response};
use sea_orm::{DatabaseConnection, DatabaseTransaction, TransactionTrait};

use crate::responses::internal_server_error;

/// Param type to use in endpoints that need a database transaction.
pub type DbTxn = Arc<DatabaseTransaction>;

/// A function that checks if a response is successful.
pub type CheckFn = Arc<dyn Fn(&Response) -> bool + Send + Sync>;

/// A middleware for automatically creating and managing
/// [`sea_orm::DatabaseTransaction`](sea_orm::DatabaseTransaction)s for incoming
/// requests.
pub struct DbTransactionMiddleware {
    db: DatabaseConnection,
    check_fn: Option<CheckFn>,
}

impl Debug for DbTransactionMiddleware {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbTransactionMiddleware")
            .field("db", &self.db)
            // .field("check_fn", &self.check_fn) // Can't implement this because it can't really be debugged.
            .finish()
    }
}

impl DbTransactionMiddleware {
    /// Create a new DbTransactionMiddleware.
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db, check_fn: None }
    }

    /// Adds a custom filter function to the middleware.
    pub fn with_filter<F>(self, check_fn: F) -> Self
    where
        F: Fn(&Response) -> bool + Send + Sync + 'static,
    {
        Self {
            db: self.db,
            check_fn: Some(Arc::new(check_fn)),
        }
    }
}

impl<E: Endpoint> Middleware<E> for DbTransactionMiddleware {
    type Output = DbTransactionMwEndpoint<E>;

    fn transform(&self, ep: E) -> Self::Output {
        DbTransactionMwEndpoint {
            inner: ep,
            db: self.db.clone(),
            check_fn: self.check_fn.clone(),
        }
    }
}

#[doc(hidden)]
pub struct DbTransactionMwEndpoint<E> {
    inner: E,
    db: DatabaseConnection,
    check_fn: Option<CheckFn>,
}

impl<E> Debug for DbTransactionMwEndpoint<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbTransactionMwEndpoint")
            .field("inner", &"...")
            .field("db", &self.db)
            .field(
                "check_fn",
                &if self.check_fn.is_some() {
                    "Some(...)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}

#[async_trait]
impl<E: Endpoint> Endpoint for DbTransactionMwEndpoint<E> {
    type Output = Response;

    async fn call(&self, mut req: poem::Request) -> Result<Self::Output, poem::Error> {
        let txn = Arc::new(self.db.begin().await.map_err(internal_server_error)?);
        req.extensions_mut().insert(txn.clone());
        let result = self.inner.call(req).await;
        let txn = Arc::try_unwrap(txn).map_err(|_| {
            internal_server_error("db transaction has not been dropped in endpoint")
        })?;
        match result {
            Ok(resp) => {
                let resp = resp.into_response();
                if let Some(check_fn) = &self.check_fn {
                    if check_fn(&resp) {
                        txn.commit().await.map_err(internal_server_error)?;
                    } else {
                        txn.rollback().await.map_err(internal_server_error)?;
                    }
                } else if resp.status().is_server_error() || resp.status().is_client_error() {
                    txn.rollback().await.map_err(internal_server_error)?;
                } else {
                    txn.commit().await.map_err(internal_server_error)?;
                }
                Ok(resp)
            }
            Err(err) => {
                txn.rollback().await.map_err(internal_server_error)?;
                Err(err)
            }
        }
    }
}
