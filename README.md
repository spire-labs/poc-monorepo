# poc-monorepo

DISCLAIMER

This software is provided as a proof of concept and is not intended for production use. The authors make no warranties or representations about the suitability of the software for any purpose. The software is provided "as is," without any express or implied warranties, including but not limited to the implied warranties of merchantability and fitness for a particular purpose.

The use of this software is at your own risk. In no event shall the authors or copyright holders be liable for any claim, damages, or other liability, whether in an action of contract, tort, or otherwise, arising from, out of, or in connection with the software or the use or other dealings in the software.

## Welcome to the Spire PoC Monorepo

This monorepo shows prototypes of some of the building blocks of Spire's based appchain framework. It consists of the following components:

- apps/wallet: A simple web wallet used for submitting transactions from appchains, and displaying appchain balances
- apps/gateway-api: This gateway receives transactions from users, converts them into preconfirmation requests, and routes them to elected enforcers
- apps/enforcer: This service is responsible for posting transaction validity conditions to L1 contracts
- apps/proposer: This service builds blocks that satisfy the Validity Conditions sent by the Enforcer.

There is also a set of smart contracts used to determine the election of proposers and enforcers. These can be found under `packages`:
- packages/spvm-1/src/spvm-1: This is a small state transition machine capable of being executed in the EVM. It is used only in the PoC.
- packages/spvm-1/lib/election-contract: Handles election of Proposers and Enforcers.
- packages/spvm-1/lib/preconfirmations-slashing: This contract is used to slash enforces that misbehave.

The goal of this PoC is to showcase some basic functionality of Spire, including cross-chain transfers with fast preconfirmations, and appchain composibility with cross-chain swaps.


## Demo Setup

Having a working docker installation is required to run things locally.
Please note that as this code is intended as a PoC, there may be some bugs in it.

If you want a one-line setup to run the demo (without making code changes), everything is set up in the `docker-compose.yml` file. Just run:

```shell
cp apps/enforcer/.env.docker apps/enforcer/.env
cp apps/gateway-api/.env.docker apps/gateway-api/.env
cp apps/proposer/.env.docker apps/proposer/.env
docker compose up
# in a separate tab
cd apps/wallet
cp .env.development .env
npm i
npm run start
```

## Development Setup

If you want to change the code, it is recommended that you run the subsystem (enforcer, proposer, or gateway) you are working on locally. The subsystems you are not editing can be run in docker.

For example, if you are working on the enforcer, you can run the enforcer locally and the proposer and gateway in docker. It is convenient to run the docker containers separately, so you can see the logs of each subsystem separately.

```shell
cp apps/gateway-api/.env.docker apps/gateway-api/.env
docker compose up gateway_api
# then in another tab
cp apps/proposer/.env.docker apps/proposer/.env
docker compose up proposer
# then in yet another tab
cp apps/enforcer/.env.development apps/enforcer/.env # .development instead of .docker
cd apps/enforcer
cargo run
```

The difference between `.env.docker` and `.env.development` is

```shell
diff apps/enforcer/.env.docker apps/enforcer/.env.development

4c4
< GATEWAY_IP=http://gateway_api:5433
---
> GATEWAY_IP=http://localhost:5433
```

In the docker network, `gateway_api` is resolved to the IP of the gateway container. When running locally, the gateway is port-forwarded.

```shell
docker ps

CONTAINER ID   IMAGE                      COMMAND                  CREATED          STATUS          PORTS                                                 NAMES
0000426ea6a4   poc-monorepo-gateway_api   "gateway-api"            42 minutes ago   Up 42 minutes   0.0.0.0:5433->5433/tcp, :::5433->5433/tcp, 8080/tcp   gateway_api
7048825ba0d2   postgres:16                "docker-entrypoint.s…"   8 days ago       Up 22 hours     0.0.0.0:5435->5432/tcp, :::5435->5432/tcp             gateway_db
```

## Enforcer

The enforcer takes preconfirmation requests from the gateway and submits validity conditions to the preconfirmation slashing contract.

<!-- image -->

![Enforcer](docs_assets/poc-architecture.png)

```shell
cargo run

# a lot of SQL commands like CREATE TABLE for the migration
# ...
2024-06-26T16:10:21.849562Z  INFO sea_orm_migration::migrator: Migration 'm20220101_000001_create_tables' has been applied
2024-06-26T16:10:21.850338Z  INFO sqlx::query: summary="INSERT INTO \"seaql_migrations\" (\"version\", …" db.statement="\n\nINSERT INTO\n  \"seaql_migrations\" (\"version\", \"applied_at\")\nVALUES\n  (?, ?)\n" rows_affected=1 rows_returned=0 elapsed=58.698µs elapsed_secs=5.8698e-5
2024-06-26T16:10:21.850517Z DEBUG enforcer: Enforcer listening on 0.0.0.0:5555
# every 12 seconds you should see
2024-06-26T16:10:33.837635Z  INFO enforcer: Successfully submitted validity condition.
2024-06-26T16:10:45.837453Z  INFO enforcer: Successfully submitted validity condition.
```

## spvm-1

```shell
git submodule update
cd apps/spvm-1
forge build
```

## Special instructions for running scripts

The scripts in /scripts can be used to spin up an Anvil node and populate it with contracts. There are two in particular:

`scripts/setup_and_run.sh`
This is used to install all dependencies for the projects, spin up a python virtual env, and then execute `demo_setup_script.py`, which will deploy contracts to anvil and populate them with some initial data. Note that this script is designed to run on Linux, as it is using apt for its package manager. If you are on a Mac, additional instructions have been added below.

`demo_setup_script.py`
This will spin up an anvil node, deploy an ERC20 contract, and deploy the Spire contracts.

## running scripts locally on a mac instructions

To get things up and running quickly on a local mac, some one-time steps are done manually to simplify the script. use the following instructions, starting in the root of the monorepo project.

- Install forge/foundry if not already done
  -- curl -L https://foundry.paradigm.xyz | bash
  -- source ~/.bashrc
  -- foundryup
- from the root of the monorepo project, run `forge install OpenZeppelin/openzeppelin-contracts`
- run `forge build` to compile the ERC20 contract used in Demo 3
- Ensure that you have your github ssh credentials set up locally. then `cd scripts` and then run `./setup_and_run_mac.sh`. This should pull down all of the smart contract repos, compile, and run anvil

## Common Errors

If you see `failed to solve: error from sender: open /poc-monorepo/apps/gateway-api/postgres: permission denied`, just change the permissions

```shell
sudo chmod -R 777 apps/gateway-api/postgres
```

# Testing

## Enforcer

```shell
cd apps/enforcer
cargo test
```

## Gateway API

```shell
cd apps/gateway-api
cargo test
```

## Proposer

```shell
cd apps/proposer
cargo test
```

## spvm-1

You must have `forge` [installed](https://book.getfoundry.sh/getting-started/installation)

```shell
cd apps/spvm-1
forge build
forge test
```

### Commands that may be helpful (disorganised)

```
insert into initialized_tickers (ticker, is_initialized) values ('RAIN', true);
insert into state (ticker, owner_address, amount) values ('RAIN', '0xa0ee7a142d267c1f36714e4a8f75612f20a79720', 100);
insert into initialized_tickers (ticker, is_initialized) values ('QUEEN', true);
insert into state (ticker, owner_address, amount) values ('QUEEN', '0xa0ee7a142d267c1f36714e4a8f75612f20a79720', 200);
```
