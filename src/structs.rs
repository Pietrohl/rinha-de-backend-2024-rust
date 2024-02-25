use axum::{
    async_trait,
    extract::{path, rejection::PathRejection, FromRequestParts},
    http::{request::Parts, StatusCode},
};
use chrono::{DateTime, Utc};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::{io::Error, time::SystemTime};
use tokio_postgres::Row;

#[derive(Serialize, Deserialize)]
pub struct Client {
    id: i32,
    name: String,
    limit: i32,
    pub balance: i32,
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
    pub fn new_transaction(
        &mut self,
        value: i32,
        transaction_type: &str,
        description: &str,
    ) -> Result<(), Error> {
        match description.len() {
            1..=10 if transaction_type == "c" => {
                self.balance += value;

                Ok(())
            }
            1..=10 if transaction_type == "d" => {
                if (self.balance + self.limit - value.abs()) < 0 {
                    return Err(Error::new(std::io::ErrorKind::Other, "Insufficient funds"));
                }
                self.balance -= value.abs();
                Ok(())
            }
            _ => Err(Error::new(
                std::io::ErrorKind::Other,
                "Invalid transaction type",
            )),
        }
    }
}

#[derive(Serialize)]
pub struct Balance {
    total: i32,
    #[serde(rename = "data_extrato")]
    date: DateTime<Utc>,
    #[serde(rename = "limite")]
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
    #[serde(rename = "valor")]
    value: i32,
    #[serde(rename = "tipo")]
    transaction_type: Box<str>,
    #[serde(rename = "descricao")]
    description: String,
    #[serde(rename = "realizada_em")]
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct StatementDTO {
    #[serde(rename = "saldo")]
    balance: Balance,
    #[serde(rename = "ultimas_transacoes")]
    last_transactions: Vec<Transaction>,
}

impl StatementDTO {
    pub fn from(rows: Vec<Row>) -> Result<StatementDTO, Error> {
        match rows.first() {
            Some(client_row) => {
                let client = Client::from(client_row);
                let mut transactions = Vec::new();

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

                Ok(StatementDTO {
                    balance,
                    last_transactions: transactions,
                })
            }
            None => Err(Error::new(std::io::ErrorKind::Other, "Client not found")),
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
                            path::ErrorKind::WrongNumberOfParameters { .. } => PathError {
                                message: kind.to_string(),
                                location: None,
                            },

                            path::ErrorKind::ParseErrorAtKey { key, .. } => PathError {
                                message: kind.to_string(),
                                location: Some(key.clone()),
                            },

                            path::ErrorKind::ParseErrorAtIndex { index, .. } => PathError {
                                message: kind.to_string(),
                                location: Some(index.to_string()),
                            },

                            path::ErrorKind::ParseError { .. } => PathError {
                                message: kind.to_string(),
                                location: None,
                            },

                            path::ErrorKind::InvalidUtf8InPathParam { key } => PathError {
                                message: kind.to_string(),
                                location: Some(key.clone()),
                            },

                            path::ErrorKind::UnsupportedType { .. } => {
                                // this error is caused by the programmer using an unsupported type
                                // (such as nested maps) so respond with `500` instead
                                status = StatusCode::INTERNAL_SERVER_ERROR;
                                PathError {
                                    message: kind.to_string(),
                                    location: None,
                                }
                            }

                            path::ErrorKind::Message(msg) => PathError {
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
