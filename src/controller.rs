use crate::db::PostgresDatabase;
use crate::db::{DBConnection, PostgresConnection};
use crate::error_handling::internal_error;
use crate::error_handling::not_found_error;
use crate::error_handling::saldo_error;
use crate::structs::BalanceDTO;
use crate::structs::Client;
use crate::structs::TransactionDTO;
use crate::StatementDTO;
use crate::StatusCode;
use axum::extract;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;

pub async fn extrato(
    State(database): State<PostgresDatabase>,
    Path(id): Path<u16>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = PostgresConnection::new(database.pool.get().await.map_err(internal_error)?);

    let rows = conn
        .query_extrato(id as i32)
        .await
        .map_err(internal_error)?;

    Ok((
        StatusCode::OK,
        Json(StatementDTO::from(rows).map_err(not_found_error)?),
    ))
}

pub async fn transacao(
    State(database): State<PostgresDatabase>,
    Path(id): Path<u16>,
    extract::Json(payload): extract::Json<TransactionDTO>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = PostgresConnection::new(database.pool.get().await.map_err(internal_error)?);

    let mut client = Client::from(
        &conn
            .query_client(id as i32)
            .await
            .map_err(not_found_error)?,
    );

    client
        .new_transaction(
            payload.value,
            &payload.transaction_type,
            &payload.description,
        )
        .map_err(saldo_error)?;

    conn.update_client(id as i32, client.balance)
        .await
        .map_err(saldo_error)?;

    conn.insert_transaction(
        id as i32,
        payload.value,
        &payload.transaction_type,
        &payload.description,
    )
    .await
    .map_err(saldo_error)?;

    Ok((StatusCode::OK, Json(BalanceDTO::from(client))))
}
