# OpenFGA Setup# OpenFGA Docker Setup


## Postgres Setup


- create a database `openfga`
- create a user `openfga_user` with password `password`

```psql
postgres=# create database openfga
CREATE DATABASE
postgres=# create user openfga_user with encrypted password 'password';
CREATE ROLE
postgres=# grant all privileges on database openfga to openfga_user;
GRANT
```


## Docker Setup


### Run the OpenFGA Migrations

```shell
docker run --rm --network=host openfga/openfga migrate \
    --datastore-engine postgres \
    --datastore-uri "postgres://openfga_user:password@localhost:5432/openfga?sslmode=disable"
```


### Run the OpenFGA Service

```shell
docker run --name openfga --network=host -p 3000:3000 -p 8080:8080 -p 8081:8081 openfga/openfga run \
    --datastore-engine postgres \
    --datastore-uri 'postgres://openfga_user:password@localhost:5432/openfga?sslmode=disable'
```

### Open The Playground

Open the link `http://localhost:3000/playground` in the browser. 


## Reference

- https://openfga.dev/docs/getting-started/setup-openfga/docker#using-postgres

