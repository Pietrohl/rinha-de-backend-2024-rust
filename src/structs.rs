use alesia_client::types::structs::TableRow;
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


#[allow(dead_code)]

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
    pub fn from(row: &Row) -> BalanceDTO {
        BalanceDTO {
            total: row.get("return_balance"),
            limit: row.get("return_limit"),
        }
    }

    pub fn from_alesia_response(response: Vec<TableRow>) -> Result<BalanceDTO, Error> {
        match response.first() {
            Some(row) => Ok(BalanceDTO {
                total: row.get(0).into(),
                limit: row.get(1).into(),
            }),
            None => Err(Error::new(std::io::ErrorKind::Other, "Client not found")),
        }
    }
}

#[derive(Deserialize)]
pub struct TransactionDTO {
    #[serde(rename = "valor")]
    pub value: i32,
    #[serde(rename = "tipo")]
    pub transaction_type: String,
    #[serde(rename = "descricao")]
    pub description: String,
}

#[derive(Serialize)]
struct Transaction {
    #[serde(rename = "valor")]
    value: i32,
    #[serde(rename = "tipo")]
    transaction_type: String,
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

    pub fn from_alesia_response(response: Vec<TableRow>) -> Result<StatementDTO, Error> {
        match response.first() {
            Some(client_row) => {
                let client = Client {
                    id: client_row.get(0).into(),
                    name: client_row.get(1).into(),
                    limit: client_row.get(2).into(),
                    balance: client_row.get(3).into(),
                };
                let mut transactions = Vec::new();

                let balance = Balance {
                    total: client.balance,
                    limit: client.limit,
                    date: SystemTime::now().into(),
                };

                for row in response {
                    let unix_timestamp_string: String = row.get(8).into();
                    let unix_timestamp = DateTime::parse_from_str(
                        &unix_timestamp_string,
                        "%Y %b %d %H:%M:%S%.3f %z",
                    );

                    match unix_timestamp {
                        Ok(timestamp) => {
                            let transaction = Transaction {
                                value: row.get(5).into(),
                                transaction_type: row.get(6).into(),
                                description: row.get(7).into(),
                                timestamp: timestamp.into(),
                            };
                            transactions.push(transaction);
                        }
                        Err(_) => {}
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
