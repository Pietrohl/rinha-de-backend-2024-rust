use std::io::Error;

use alesia_client::{types::structs::TableRow, AlesiaClient};

pub async fn query_extrato(
    alesia_client: &mut AlesiaClient,
    id: i32,
) -> Result<Vec<TableRow>, Error> {
    let response = alesia_client.query(EXTRATO_QUERY_STATEMENT, &[&id]).await;

    match response {
        Ok(response) => Ok(response),
        Err(err) => Err(Error::new(std::io::ErrorKind::Other, err.to_string())),
    }
}

pub async fn insert_transaction(
    alesia_client: &mut AlesiaClient,
    id: i32,
    value: i32,
    transaction_type: String,
    description: String,
) -> Result<(), Error> {
    alesia_client
        .insert(
            TRANSACAO_QUERY_STATEMENT,
            &[&id, &value, &transaction_type, &description, &value.abs()],
        )
        .await
        .map_err(|err| Error::new(std::io::ErrorKind::Other, err.to_string()))
}

pub async fn query_balance(alesia_client: &mut AlesiaClient, id: i32) -> Result<Vec<TableRow>, Error> {
    let response = alesia_client.query(BALANCE_QUERY_STATEMENT, &[&id]).await;

    match response {
        Ok(response) => Ok(response),
        Err(err) => Err(Error::new(std::io::ErrorKind::Other, err.to_string())),
    }
}



const EXTRATO_QUERY_STATEMENT: &str = "SELECT 
c.id AS id,
c.name AS name,
c.max_limit,
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
c.id = ?1 
ORDER BY 
    t.id DESC
LIMIT 10;";

const TRANSACAO_QUERY_STATEMENT: &str = "WITH TempTable AS (
    SELECT * FROM clients
    WHERE id = ?1 AND balance + max_limit + ?2 >=0 	
)
INSERT INTO transactions (client_id, value, type, description)
SELECT
    ?1 AS client_id,
    ?5 AS value,
    ?3 AS type,
    ?4 AS description
WHERE EXISTS (
    SELECT 1
    FROM TempTable
    WHERE id IS NOT NULL
);
  ";


const BALANCE_QUERY_STATEMENT: &str = "SELECT balance, max_limit FROM clients WHERE id = ?1;";