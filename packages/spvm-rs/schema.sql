CREATE TABLE initialized_tickers (
	ticker TEXT PRIMARY KEY,
	is_initialized BOOLEAN NOT NULL);

CREATE TABLE state (
            ticker TEXT,
            owner_address TEXT,
            amount INTEGER CHECK (amount >= 0 AND amount <= 65535),
            PRIMARY KEY (ticker, owner_address),
            FOREIGN KEY (ticker) REFERENCES initialized_tickers(ticker)
        );

CREATE TABLE nonces (
            owner_address TEXT PRIMARY KEY,
            nonce INTEGER NOT NULL CHECK (nonce >= 0 AND nonce <= 4294967295)
        );
