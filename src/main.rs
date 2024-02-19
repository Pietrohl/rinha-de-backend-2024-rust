use std::time::SystemTime;
use axum::async_trait;
use axum::extract::path::ErrorKind;
use axum::extract::rejection::PathRejection;
use axum::extract::FromRequestParts;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::Json;
use axum::{
    http::{request::Parts, StatusCode},
    routing::get,
    routing::post,
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use chrono::{DateTime, Utc};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio_postgres::NoTls;
use tokio_postgres::Row;
#[derive(Serialize, Deserialize)]
struct Client {
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
struct Balance {
    total: i32,
    date: DateTime<Utc>,
    limit: i32,
}
#[derive(Serialize)]
struct Transaction {
    value: i32,
    transaction_type: Box<str>,
    description: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
struct Statement {
    balance: Balance,
    last_transactions: Vec<Transaction>,
}

impl Statement {
    pub fn from(rows: Vec<Row>) -> Statement {
        let mut transactions = Vec::new();

        let client = Client::from(&rows.first().unwrap());

        let balance = Balance {
            total: client.balance,
            limit: client.limit,
            date: SystemTime::now().into(),
        };

        for row in rows {
            let unix_timestamp: SystemTime = row.get("transaction_timestamp");

            let transaction = Transaction {
                value: row.get("transaction_value"),
                transaction_type: row.get("transaction_type"),
                description: row.get("transaction_description"),
                timestamp: unix_timestamp.into(),
            };

            transactions.push(transaction);
        }

        Statement {
            balance,
            last_transactions: transactions,
        }
    }
}

#[tokio::main]
async fn main() {
    let manager = PostgresConnectionManager::new_from_stringlike(
        "host=localhost user=postgres dbname=rinha_db",
        NoTls,
    )
    .unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/clientes/:id/transacoes", post(transacao))
        .route("/clientes/:id/extrato", get(extrato))
        .with_state(pool);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

async fn transacao() {}

async fn extrato(
    State(pool): State<ConnectionPool>,
    Path(id): Path<u16>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let rows = conn
        .query(
            "SELECT 
            c.id AS id,
            c.name AS name,
            c.limit AS limit,
            c.balance AS balance,
            t.id AS transaction_id,
            t.value AS transaction_value,
            t.type AS transaction_type,
            t.description AS transaction_description,
            t.timestamp AS transaction_timestamp
        FROM 
            clients c
        LEFT JOIN 
            transactions t ON c.id = t.client_id
        WHERE
            c.id = $1;",
            &[&(id as i32)],
        )
        .await
        .map_err(internal_error)?;

    Ok(Json(Statement::from(rows)))
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
    E: std::error::Error,
{
    (StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
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
