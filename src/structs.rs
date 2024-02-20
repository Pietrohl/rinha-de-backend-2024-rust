use serde::de::DeserializeOwned;
use std::time::SystemTime;

use axum::{
    async_trait,
    extract::{path::ErrorKind, rejection::PathRejection, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres::Row;

#[derive(Serialize, Deserialize)]
pub struct Client {
    id: i32,
    name: String,
    limit: i32,
    balance: i32,
}

impl Client {
    pub fn from(row: &Row) -> Client {
        Client {
            id: row.get("id"),
            name: row.get("name"),
            limit: row.get("limit"),
            balance: row.get("balance"),
        }
    }
}

#[derive(Serialize)]
pub struct Balance {
    total: i32,
    date: DateTime<Utc>,
    limit: i32,
}

#[derive(Serialize)]
pub struct BalanceDTO {
    #[serde(rename = "saldo")]
    total: i32,
    #[serde(rename = "limite")]
    limit: i32,
}

impl BalanceDTO {
    pub fn from(client: Client) -> BalanceDTO {
        BalanceDTO {
            total: client.balance,
            limit: client.limit,
        }
    }
}

#[derive(Deserialize)]
pub struct TransactionDTO {
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "tipo")]
    pub transaction_type: Box<str>,
    #[serde(rename = "descricao")]
    pub description: String,
}

#[derive(Serialize)]
struct Transaction {
    value: i32,
    transaction_type: Box<str>,
    description: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct StatementDTO {
    balance: Balance,
    last_transactions: Vec<Transaction>,
}

impl StatementDTO {
    pub fn from(rows: Vec<Row>) -> StatementDTO {
        let mut transactions = Vec::new();

        let client = Client::from(&rows.first().unwrap());

        let balance = Balance {
            total: client.balance,
            limit: client.limit,
            date: SystemTime::now().into(),
        };

        for row in rows {
            let unix_timestamp: Option<SystemTime> = row.get("transaction_timestamp");

            match unix_timestamp {
                Some(timestamp) => {
                    let transaction = Transaction {
                        value: row.get("transaction_value"),
                        transaction_type: row.get("transaction_type"),
                        description: row.get("transaction_description"),
                        timestamp: timestamp.into(),
                    };
                    transactions.push(transaction);
                }
                None => {}
            }
        }

        StatementDTO {
            balance,
            last_transactions: transactions,
        }
    }
}

struct Path<T>(T);

#[async_trait]
impl<S, T> FromRequestParts<S> for Path<T>
where
    // these trait bounds are copied from `impl FromRequest for axum::extract::path::Path`
    T: DeserializeOwned + Send,
    S: Send + Sync,
{
    type Rejection = (StatusCode, axum::Json<PathError>);

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        match axum::extract::Path::<T>::from_request_parts(parts, state).await {
            Ok(value) => Ok(Self(value.0)),
            Err(rejection) => {
                let (status, body) = match rejection {
                    PathRejection::FailedToDeserializePathParams(inner) => {
                        let mut status = StatusCode::BAD_REQUEST;

                        let kind = inner.into_kind();
                        let body = match &kind {
                            ErrorKind::WrongNumberOfParameters { .. } => PathError {
                                message: kind.to_string(),
                                location: None,
                            },

                            ErrorKind::ParseErrorAtKey { key, .. } => PathError {
                                message: kind.to_string(),
                                location: Some(key.clone()),
                            },

                            ErrorKind::ParseErrorAtIndex { index, .. } => PathError {
                                message: kind.to_string(),
                                location: Some(index.to_string()),
                            },

                            ErrorKind::ParseError { .. } => PathError {
                                message: kind.to_string(),
                                location: None,
                            },

                            ErrorKind::InvalidUtf8InPathParam { key } => PathError {
                                message: kind.to_string(),
                                location: Some(key.clone()),
                            },

                            ErrorKind::UnsupportedType { .. } => {
                                // this error is caused by the programmer using an unsupported type
                                // (such as nested maps) so respond with `500` instead
                                status = StatusCode::INTERNAL_SERVER_ERROR;
                                PathError {
                                    message: kind.to_string(),
                                    location: None,
                                }
                            }

                            ErrorKind::Message(msg) => PathError {
                                message: msg.clone(),
                                location: None,
                            },

                            _ => PathError {
                                message: format!("Unhandled deserialization error: {kind}"),
                                location: None,
                            },
                        };

                        (status, body)
                    }
                    PathRejection::MissingPathParams(error) => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        PathError {
                            message: error.to_string(),
                            location: None,
                        },
                    ),
                    _ => (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        PathError {
                            message: format!("Unhandled path rejection: {rejection}"),
                            location: None,
                        },
                    ),
                };

                Err((status, axum::Json(body)))
            }
        }
    }
}

#[derive(Serialize)]
struct PathError {
    message: String,
    location: Option<String>,
}
