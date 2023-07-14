CREATE TABLE users (
                       id SERIAL PRIMARY KEY,
                       login VARCHAR UNIQUE NOT NULL,
                       name VARCHAR NOT NULL,
                       is_admin BOOLEAN NOT NULL,
                       is_worker BOOLEAN NOT NULL,
                       pay DOUBLE PRECISION NOT NULL,
                       percent DOUBLE PRECISION NOT NULL,
                       pwd_hash VARCHAR NOT NULL,
                       pwd_salt VARCHAR NOT NULL,
                       token VARCHAR NOT NULL
);

CREATE TABLE schedule (
                          day INTEGER NOT NULL,
                          month INTEGER NOT NULL,
                          year INTEGER NOT NULL,
                          user_id SERIAL NOT NULL,

                          PRIMARY KEY(day, month, year, user_id),
                          FOREIGN KEY(user_id) REFERENCES users(id)
);

CREATE TABLE revenue (
                         day INTEGER NOT NULL,
                         month INTEGER NOT NULL,
                         year INTEGER NOT NULL,
                         with_percent DOUBLE PRECISION NOT NULL,
                         without_percent DOUBLE PRECISION NOT NULL,
                         PRIMARY KEY(day, month, year)
);

CREATE TABLE payouts (
                         day INTEGER NOT NULL,
                         month INTEGER NOT NULL,
                         year INTEGER NOT NULL,
                         user_id SERIAL NOT NULL,
                         amount DOUBLE PRECISION NOT NULL,
                         PRIMARY KEY(day, month, year, user_id),
                         FOREIGN KEY(user_id) REFERENCES users(id)
);