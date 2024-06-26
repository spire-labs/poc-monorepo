# poc-monorepo

Spire PoC Monorepo!

[Poc Spec](https://www.notion.so/spirelabs/Spire-Based-Stack-PoC-45ecd6a1afa44f8c8f28f086b42b08c5)

[General PoC](https://www.notion.so/spirelabs/Spire-PoC-Infrastructure-9caebb8915f24a1fba9caf1365b05737)

TODO (megan):

- [x] finish creating a shared docker-compose.yml for all rust executables in monorepo (they live in the apps directory)
- [x] set up github actions for PRs and pushes to main
- [x] add .dockerignore files to apps to speed up build process
- [x] test out functionality with hosted Anvil instance
- [x] add Spire wallet/poc-infra/smart contracts to monorepo (useful commands for this include `rm -rf .git` inside the folder for the app added, then `git rm --cached path/to/app` from parent folder, followed by `git add path/to/app`)
- [x] make sure the apps are up to date with the latest in all individual repos, announce to the team that we now build in the monorepo only
- [ ] bring env vars out of individual apps into one consolidated file?
- [ ] write additional scripts (makefile?) to spin up all apps in monorepo
- [ ] add setup instructions in READMEs all apps for development purposes
- [ ] add TODOs in all READMEs for getting the monorepo ready for release

Note: Having a working docker installation is required.

# Demo Setup

If you want a one-line setup to run the demo (without making code changes), everything is set up in the `docker-compose.yml` file. Just run:

```shell
cp apps/enforcer/.env.docker apps/enforcer/.env
cp apps/gateway-api/.env.docker apps/gateway-api/.env
cp apps/proposer/.env.docker apps/proposer/.env
docker compose up
```

# Development Setup

If you want to change the code, it is recommended that you run the subsystem (enforcer, proposer, or gateway) you are working on locally. The subsystems you are not editing can be run in docker.

For example, if you are working on the enforcer, you can run the enforcer locally and the proposer and gateway in docker. It is convenient to run the docker containers separately, so you can see the logs of each subsystem separately.

```shell
cp apps/gateway-api/.env.docker apps/gateway-api/.env
docker compose up gateway-api
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

As seen in the notion documentation above, the enforcer takes preconfirmation requests from the gateway and submits validity conditions to the preconfirmation slashing contract.

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

# Addresses Used

Here is the list of [addresses](https://www.notion.so/spirelabs/Spire-PoC-Infrastructure-9caebb8915f24a1fba9caf1365b05737?pvs=4#d327fa44da264312ad8ac3bebae25c4a) used in the PoC, along with their private keys (may want to remove this when making this repo public, and add instructions for people to setup their own anvil node and generate their own wallets).

# Environment Configuration

TODO - For now see individual app READMEs and/or .env.example files.

# Testing

TODO
