use axum::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use serde::de;
use tokio_postgres::{NoTls, Row};

pub(crate) type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

pub(crate) type BB8PooledConnection<'a> =
    bb8::PooledConnection<'a, bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>;

#[async_trait]
pub trait DBConnection<'a> {
    async fn query_extrato(&self, id: i32) -> Result<Vec<Row>, tokio_postgres::Error>;
    async fn query_client(&self, id: i32) -> Result<Row, tokio_postgres::Error>;
    async fn insert_transaction(
        &self,
        id: i32,
        value: i32,
        transaction_type: &str,
        description: &str,
    ) -> Result<u64, tokio_postgres::Error>;
    async fn update_client(&self, id: i32, balance: i32) -> Result<u64, tokio_postgres::Error>;
}

pub(crate) struct PostgresConnection<'a> {
    conn: BB8PooledConnection<'a>,
}



impl<'a> PostgresConnection<'a> {
    pub fn new(conn: BB8PooledConnection<'a>) -> Self {
        Self { conn }
    }
}

#[async_trait]
impl<'a> DBConnection<'a> for PostgresConnection<'a> {
    async fn query_extrato(&self, id: i32) -> Result<Vec<Row>, tokio_postgres::Error> {
        self.conn.query(EXTRATO_QUERY_STATEMENT, &[&id]).await
    }

    async fn query_client(&self, id: i32) -> Result<Row, tokio_postgres::Error> {
        self.conn
            .query_one(TRANSACAO_QUERY_STATEMENT_2, &[&id])
            .await
    }

    async fn insert_transaction(
        &self,
        id: i32,
        value: i32,
        transaction_type: &str,
        description: &str,
    ) -> Result<u64, tokio_postgres::Error> {
        self.conn
            .execute(
                TRANSACAO_QUERY_STATEMENT_1,
                &[&id, &value, &transaction_type, &description],
            )
            .await
    }

    async fn update_client(&self, id: i32, balance: i32) -> Result<u64, tokio_postgres::Error> {
        self.conn
            .execute(TRANSACAO_UPDATE_CLIENT, &[&id, &balance])
            .await
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PostgresDatabase {
   pub pool: ConnectionPool,
}

impl PostgresDatabase {
    pub(crate) fn new(pool: ConnectionPool) -> Self {
        Self { pool }
    }
}

pub(crate) const DATABASE_URL: &str = "host=localhost user=postgres dbname=rinha_db";

pub(crate) const EXTRATO_QUERY_STATEMENT: &str = "SELECT 
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
c.id = $1 
ORDER BY 
    t.id DESC
LIMIT 10;";

pub(crate) const TRANSACAO_QUERY_STATEMENT_1: &str = "
    INSERT INTO transactions (client_id, value, type, description) 
    VALUES 
        ($1, $2, $3, $4);
   ";

pub(crate) const TRANSACAO_QUERY_STATEMENT_2: &str = " SELECT c.id AS id,
   c.name AS name,
   c.limit AS limit,
   c.balance AS balance
   FROM clients c WHERE id = $1 ;";

pub(crate) const TRANSACAO_UPDATE_CLIENT: &str = "UPDATE clients SET balance = $2 WHERE id = $1;";
