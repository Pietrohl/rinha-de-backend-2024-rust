mod controller;
mod db;
mod error_handling;
mod structs;

use crate::structs::StatementDTO;
use axum::{http::StatusCode, routing::get, routing::post, Router};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use controller::{extrato, transacao};
use db::DATABASE_URL;
use tokio_postgres::NoTls;

#[tokio::main]
async fn main() {
    let manager = PostgresConnectionManager::new_from_stringlike(DATABASE_URL, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();

    let app = Router::new()
        .route("/clientes/:id/transacoes", post(transacao))
        .route("/clientes/:id/extrato", get(extrato))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:9999").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
