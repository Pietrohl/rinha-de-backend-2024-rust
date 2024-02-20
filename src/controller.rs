use crate::db::ConnectionPool;
use crate::db::EXTRATO_QUERY_STATEMENT;
use crate::db::TRANSACAO_QUERY_STATEMENT_1;
use crate::db::TRANSACAO_QUERY_STATEMENT_2;
use crate::internal_error;
use crate::structs::BalanceDTO;
use crate::structs::Client;
use crate::structs::TransactionDTO;
use crate::StatementDTO;
use crate::StatusCode;
use axum::extract;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use tokio_postgres::GenericClient;

pub async fn extrato(
    State(pool): State<ConnectionPool>,
    Path(id): Path<u16>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    let rows = conn
        .query(EXTRATO_QUERY_STATEMENT, &[&(id as i32)])
        .await
        .map_err(internal_error)?;

    Ok(Json(StatementDTO::from(rows)))
}

pub async fn transacao(
    State(pool): State<ConnectionPool>,
    Path(id): Path<u16>,
    extract::Json(payload): extract::Json<TransactionDTO>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let conn = pool.get().await.map_err(internal_error)?;

    conn.client()
        .prepare(TRANSACAO_QUERY_STATEMENT_1)
        .await
        .map_err(internal_error)?;
    conn.client()
        .prepare(TRANSACAO_QUERY_STATEMENT_2)
        .await
        .map_err(internal_error)?;

    conn.execute(
        TRANSACAO_QUERY_STATEMENT_1,
        &[
            &(id as i32),
            &payload.value,
            &payload.transaction_type,
            &payload.description,
        ],
    )
    .await
    .map_err(internal_error)?;

    let row = conn
        .query_one(TRANSACAO_QUERY_STATEMENT_2, &[&(id as i32)])
        .await
        .map_err(internal_error)?;


    Ok((StatusCode::OK, Json(BalanceDTO::from(Client::from(&row)))))
}
