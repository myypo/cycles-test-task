use std::{borrow::Cow, collections::HashMap};

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use sqlx::error::DatabaseError;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error, aide::OperationIo)]
pub enum Error {
    #[error("authentication required")]
    Unauthorized,

    #[error("the current user can't perform this action: {0}")]
    Forbidden(&'static str),

    #[error("the relevant resource was not found: {0}")]
    ResourceNotFound(&'static str),

    #[error("invalid request")]
    BadRequest(InputErrorList),

    #[error("some of the provided data that has to be unique turned out not to be")]
    Conflict(InputErrorList),

    #[error("{}", internal(.0))]
    Internal(#[from] anyhow::Error),

    #[error("{}", sqlx(.0))]
    Sqlx(#[from] sqlx::Error),
}

fn internal(e: &anyhow::Error) -> String {
    if cfg!(feature = "dev") {
        format!("internal error: {}", e)
    } else {
        "an internal error occurred".to_owned()
    }
}

fn sqlx(e: &sqlx::Error) -> String {
    if cfg!(feature = "dev") {
        format!("sqlx error: {}", e)
    } else {
        "an internal error occurred".to_owned()
    }
}

#[derive(Debug, serde::Serialize, schemars::JsonSchema)]
#[serde(transparent)]
pub struct InputErrorList(HashMap<Cow<'static, str>, Vec<Cow<'static, str>>>);

impl InputErrorList {
    pub fn new<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        let mut error_map = HashMap::new();

        for (key, val) in errors {
            error_map
                .entry(key.into())
                .or_insert_with(Vec::new)
                .push(val.into());
        }

        Self(error_map)
    }
}

impl Error {
    pub fn conflict<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        Error::Conflict(InputErrorList::new(errors))
    }

    pub fn bad_request<K, V>(errors: impl IntoIterator<Item = (K, V)>) -> Self
    where
        K: Into<Cow<'static, str>>,
        V: Into<Cow<'static, str>>,
    {
        Error::BadRequest(InputErrorList::new(errors))
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::ResourceNotFound(_) => StatusCode::NOT_FOUND,
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct LogicError {
    error: String,
}

pub type LogicErrBody = Json<LogicError>;

#[derive(serde::Serialize, schemars::JsonSchema)]
pub struct InputError {
    input_error_list: InputErrorList,
}

pub type InputErrBody = Json<InputError>;

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let code = self.status_code();

        match self {
            Self::BadRequest(e) | Self::Conflict(e) => (
                code,
                Json(InputError {
                    input_error_list: e,
                }),
            )
                .into_response(),
            _ => {
                (
                    code,
                    Json(LogicError {
                        error: self.to_string(),
                    }),
                )
            }
            .into_response(),
        }
    }
}

pub trait OnConstraint<T> {
    fn on_constraint(
        self,
        name: &str,
        f: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error>;
}

impl<T, E> OnConstraint<T> for Result<T, E>
where
    E: Into<Error>,
{
    fn on_constraint(
        self,
        name: &str,
        map_err: impl FnOnce(Box<dyn DatabaseError>) -> Error,
    ) -> Result<T, Error> {
        self.map_err(|e| match e.into() {
            Error::Sqlx(sqlx::Error::Database(dbe)) if dbe.constraint() == Some(name) => {
                map_err(dbe)
            }
            e => e,
        })
    }
}
