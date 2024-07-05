# gateway-api

DISCLAIMER

This software is provided as a proof of concept and is not intended for production use. The authors make no warranties or representations about the suitability of the software for any purpose. The software is provided "as is," without any express or implied warranties, including but not limited to the implied warranties of merchantability and fitness for a particular purpose.

The use of this software is at your own risk. In no event shall the authors or copyright holders be liable for any claim, damages, or other liability, whether in an action of contract, tort, or otherwise, arising from, out of, or in connection with the software or the use or other dealings in the software.


Spire Gateway API PoC

[Gateway PoC spec](https://www.notion.so/spirelabs/Spire-PoC-Gateway-API-431b2d5e979648318f73c435c821e88c)

[General PoC](https://www.notion.so/spirelabs/Spire-PoC-Infrastructure-9caebb8915f24a1fba9caf1365b05737)

TODO: 
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

Set up a local postgres database and configure `DATABASE_URL`. You can do this with the docker-compose configuration in this repo. Run the following command to start:

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
DATABASE_URL="postgresql://postgres:postgres@localhost:5433/gatewayapi"
RPC_URL="http://34.30.119.68:8545"
```

# Testing

TODO

```shell
cargo test
```
