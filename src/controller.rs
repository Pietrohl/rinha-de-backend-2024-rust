use crate::db::PostgresDatabase;
use crate::db::{DBConnection, PostgresConnection};
use crate::error_handling::internal_error;
use crate::error_handling::not_found_error;
use crate::error_handling::saldo_error;
use crate::structs::BalanceDTO;
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

    let value: i32 = match (payload.transaction_type.as_str(), payload.description.as_str().len()) {
        ("c", 1..=10) => payload.value,
        ("d", 1..=10) => -payload.value,
        _ => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid transaction type".to_string(),
            ))
        }
    };

    let balance = BalanceDTO::from(
        &conn
            .insert_transaction(
                id as i32,
                value as i32,
                &payload.transaction_type,
                &payload.description,
            )
            .await
            .map_err(saldo_error)?,
    );

    Ok((StatusCode::OK, Json(balance)))
}
