use axum::async_trait;
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::{NoTls, Row};

pub(crate) type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

pub(crate) type BB8PooledConnection<'a> =
    bb8::PooledConnection<'a, bb8_postgres::PostgresConnectionManager<tokio_postgres::NoTls>>;

#[async_trait]
pub trait DBConnection<'a> {
    async fn query_extrato(&self, id: i32) -> Result<Vec<Row>, tokio_postgres::Error>;
    async fn insert_transaction(
        &self,
        id: i32,
        value: i32,
        transaction_type: &str,
        description: &str,
    ) -> Result<Row, tokio_postgres::Error>;
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

    async fn insert_transaction(
        &self,
        id: i32,
        value: i32,
        transaction_type: &str,
        description: &str,
    ) -> Result<Row, tokio_postgres::Error> {
       
        let statement = self.conn.prepare(TRANSACAO_QUERY_STATEMENT).await?;

        self.conn
            .query_one(
                &statement,
                &[&id, &value, &transaction_type, &description, &value.abs()],
            )
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

pub(crate) const DATABASE_URL: &str =
    "host=db port=5432 password=postgres user=postgres dbname=rinha_db";

pub(crate) const EXTRATO_QUERY_STATEMENT: &str = "SELECT 
c.id AS id,
c.name AS name,
c.max_limit AS limit,
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

pub(crate) const TRANSACAO_QUERY_STATEMENT: &str = "WITH updated_balance AS (
      UPDATE clients
      SET balance = balance + $2   
      WHERE id = $1  AND max_limit + balance + $2 >= 0  
       RETURNING balance, max_limit
  ),
  inserted_transaction AS (
    INSERT INTO transactions (client_id, value, type, description) 
      SELECT 
         $1,
         $5,
         $3,
         $4
    WHERE EXISTS (
        SELECT 1 
        FROM updated_balance
        WHERE balance IS NOT NULL
    )
)
SELECT * FROM updated_balance;
  ";
