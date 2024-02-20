use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use tokio_postgres::NoTls;

pub(crate) type ConnectionPool = Pool<PostgresConnectionManager<NoTls>>;

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
c.id = $1;";

pub(crate) const TRANSACAO_QUERY_STATEMENT_1: &str = "
    INSERT INTO transactions (client_id, value, type, description) 
    VALUES 
        ($1, $2, $3, $4);
   ";





pub(crate) const TRANSACAO_QUERY_STATEMENT_2: &str = " SELECT c.id AS id,
   c.name AS name,
   c.limit AS limit,
   c.balance AS balance
   FROM clients c WHERE id = $1;";

