CREATE UNLOGGED TABLE clients
(
    id SERIAL PRIMARY KEY,
    name varchar(10) NOT NULL,
    max_limit integer NOT NULL,
    balance integer NOT NULL
);


CREATE UNLOGGED TABLE transactions
(
    id SERIAL PRIMARY KEY,
    client_id integer NOT NULL,
    value integer NOT NULL,
    type char NOT NULL,
    description varchar(10) NOT NULL,
    "timestamp" timestamp DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT fk_client_id FOREIGN KEY (client_id)
        REFERENCES public.clients (id) MATCH SIMPLE
)
;

INSERT INTO clients(name, max_limit, balance)
	VALUES ('one', 100000, 0),
    ('two', 80000, 0),
    ('three', 1000000, 0),
    ('four', 10000000, 0),
    ('five', 500000, 0);

