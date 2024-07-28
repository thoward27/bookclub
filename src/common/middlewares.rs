use crate::common::settings;
use async_trait::async_trait;
use axum::extract::FromRef;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use loco_rs::app::AppContext;
use loco_rs::errors::Error;
use serde::{Deserialize, Serialize};

pub mod auth {
    use super::*;
    use loco_rs::model::ModelResult;
    use sea_orm::DatabaseConnection;

    #[async_trait]
    pub trait Authenticable: Clone {
        // Proxy Auth Methods.
        async fn find_or_create_by_email(db: &DatabaseConnection, email: &str)
            -> ModelResult<Self>;
        async fn find_by_email(db: &DatabaseConnection, email: &str) -> ModelResult<Self>;

        // Disabled Auth Methods.
        async fn anonymous_user(db: &DatabaseConnection) -> ModelResult<Self>;
    }

    #[derive(Debug, Deserialize, Serialize)]
    pub struct Auth<T: Authenticable> {
        pub user: T,
    }

    #[async_trait]
    impl<S, T> FromRequestParts<S> for Auth<T>
    where
        AppContext: FromRef<S>,
        S: Send + Sync,
        T: Authenticable,
    {
        type Rejection = Error;

        async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Error> {
            let ctx: AppContext = AppContext::from_ref(state);
            let settings = settings::Settings::from_ctx(&ctx)?;
            match settings.auth {
                None => Err(Error::Unauthorized("auth is not configured".to_string())),
                Some(auth) => {
                    if let Some(anonymous) = auth.anonymous {
                        if anonymous {
                            return Ok(Self {
                                user: T::anonymous_user(&ctx.db).await?,
                            });
                        }
                    }
                    if let Some(proxy) = auth.proxy {
                        if proxy.enabled {
                            let email = parts
                                .headers
                                .get(proxy.header_name.as_str())
                                .ok_or_else(|| {
                                    Error::Unauthorized("Auth header not found".to_string())
                                })?
                                .to_str()
                                .map_err(|err| Error::Unauthorized(err.to_string()))?
                                .to_string();
                            if proxy.auto_sign_up {
                                return Ok(Self {
                                    user: T::find_or_create_by_email(&ctx.db, &email).await?,
                                });
                            }
                            let user = T::find_by_email(&ctx.db, &email).await?;
                            return Ok(Self { user });
                        }
                    }
                    return Err(Error::Unauthorized("unknown auth method".to_string()));
                }
            }
        }
    }
}
