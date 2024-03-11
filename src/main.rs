mod controller;
mod db;
mod error_handling;
mod structs;
mod sql_db;

use crate::structs::StatementDTO;
use axum::{
    extract::Request,
    http::StatusCode,
    routing::{get, post},
    Router,
};
use controller::{extrato, transacao};
use hyper::body::Incoming;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server;
use tower::Service;

#[tokio::main]
async fn main() {

    let listener = std::net::TcpListener::bind("0.0.0.0:3000")
        .expect("error listening to socket 0.0.0.0:3000");
    listener.set_nonblocking(true).unwrap();

    let listener = tokio::net::TcpListener::from_std(listener).expect("error parsing std listener");

    let app = Router::new()
        .route("/clientes/:id/transacoes", post(transacao))
        .route("/clientes/:id/extrato", get(extrato));

    eprintln!("Server up!");
    
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
