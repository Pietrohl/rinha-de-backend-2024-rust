use crate::error_handling::internal_error;
use crate::error_handling::not_found_error;
use crate::error_handling::saldo_error;
use crate::sql_db::query_balance;
use crate::sql_db::{insert_transaction, query_extrato};
use crate::structs::BalanceDTO;
use crate::structs::TransactionDTO;
use crate::StatementDTO;
use crate::StatusCode;
use alesia_client::AlesiaClient;
use axum::extract;
use axum::extract::Path;
use axum::response::IntoResponse;
use axum::Json;

pub async fn extrato(Path(id): Path<u16>) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut database = AlesiaClient::new_from_url("127.0.0.1:8080").await;

    let response = query_extrato(&mut database, id as i32)
        .await
        .map_err(internal_error)?;

    Ok((
        StatusCode::OK,
        Json(StatementDTO::from_alesia_response(response).map_err(not_found_error)?),
    ))
}

pub async fn transacao(
    Path(id): Path<u16>,
    extract::Json(payload): extract::Json<TransactionDTO>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut database = AlesiaClient::new_from_url("127.0.0.1:8080").await;

    let value: i32 = match (
        payload.transaction_type.as_str(),
        payload.description.as_str().len(),
    ) {
        ("c", 1..=10) => payload.value,
        ("d", 1..=10) => -payload.value,
        _ => {
            return Err((
                StatusCode::UNPROCESSABLE_ENTITY,
                "Invalid transaction type".to_string(),
            ))
        }
    };

    let response = insert_transaction(
        &mut database,
        id as i32,
        value,
        payload.transaction_type,
        payload.description,
    )
    .await
    .and(query_balance(&mut database, id as i32).await)
    .map_err(saldo_error)?;

    let balance = BalanceDTO::from_alesia_response(response).map_err(not_found_error)?;

    Ok((StatusCode::OK, Json(balance)))
}
