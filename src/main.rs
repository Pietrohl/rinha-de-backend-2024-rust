mod controller;
mod db;
mod error_handling;
mod structs;

use crate::structs::StatementDTO;
use axum::{
    extract::Request,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use controller::{extrato, transacao};
use db::DATABASE_URL;
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server;
use tokio_postgres::NoTls;
use tower::Service;

#[tokio::main]
async fn main() {
    let manager = PostgresConnectionManager::new_from_stringlike(DATABASE_URL, NoTls).unwrap();
    let pool = Pool::builder().build(manager).await.unwrap();
    let database = db::PostgresDatabase::new(pool.clone());

    let listener = std::net::TcpListener::bind("0.0.0.0:9999")
        .expect("error listening to socket 0.0.0.0:9999");
    listener.set_nonblocking(true).unwrap();

    let listener = tokio::net::TcpListener::from_std(listener).expect("error parsing std listener");

    let app = Router::new()
        .route("/clientes/:id/transacoes", post(transacao))
        .route("/clientes/:id/extrato", get(extrato))
        .with_state(database);

    loop {
        let (stream, _) = listener.accept().await.unwrap();

        let tower_service = app.clone();

        tokio::spawn(async move {
            let socket = TokioIo::new(stream);

            let service = hyper::service::service_fn(move |request: Request<Incoming>| {
                tower_service.clone().call(request)
            });

            if let Err(err) = server::conn::auto::Builder::new(TokioExecutor::new())
                .serve_connection(socket, service)
                .await
            {
                eprintln!("failed to serve connection: {err:#}");
            }
        });
    }
}
