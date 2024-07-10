# gateway-api

Spire Gateway API PoC

[Gateway PoC spec](https://www.notion.so/spirelabs/Spire-PoC-Gateway-API-431b2d5e979648318f73c435c821e88c)

[General PoC](https://www.notion.so/spirelabs/Spire-PoC-Infrastructure-9caebb8915f24a1fba9caf1365b05737)

TODO (megan):

- [x] add into monorepo and resolve router build issues with spvm-rs
- [ ] clean up input/output types
- [ ] write tests (coverage tool?)
- [ ] error handling (make this into a struct, currently works but needs refactor)
- [ ] comment code

# Dev Setup

Note: Having a working docker installation is required.

Install SeaORM command line tool:

```shell
cargo install sea-orm-cli
```

## Database

### Start Postgres

Set up a local postgres database and configure `GATEWAY_API_DB`. You can do this with the docker-compose configuration in this repo. Run the following command to start:

```shell
docker-compose up -d postgres
```

### Migrate Database

To initialize/migrate your local database:

```shell
sea-orm-cli migrate -u postgresql://gatewayapi:gatewayapi@localhost:5433/gatewayapi
```

To generate entity files after a migration:

```shell
sea-orm-cli generate entity \
    -u postgresql://postgres:postgres@localhost:5433/gatewayapi \
    -o entity
```

## Environment Configuration

Create a .env file and add the following variables (should be fine to copy .env.example):

```shell
GATEWAY_API_DB="postgresql://postgres:postgres@localhost:5433/gatewayapi"
ANVIL_RPC_URL="http://34.30.119.68:8545"
```

# Testing

TODO

```shell
cargo test
```
