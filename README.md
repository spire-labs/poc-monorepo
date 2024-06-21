# poc-monorepo

Spire PoC Monorepo!

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

# Dev Setup

```shell
cp apps/enforcer/.env.sample apps/enforcer/.env
cp apps/gateway-api/.env.example apps/gateway-api/.env
cp apps/proposer/.env.example apps/proposer/.env
```

Note: Having a working docker installation is required.

Start docker:

```shell
docker network create shared_network && docker-compose up
```

The POC contains 3 services, each having their individual READMEs:

- enforcer (`./apps/enforcer/README.md`)
- proposer (`./apps/proposer/README.md`)
- gateway-api (`./apps/gateway-api/README.md`)

The enforcer depends on the gateway-api. If you try to `docker compose up enforcer` without the gateway-api running, you will get this error:

```shell
enforcer     | called `Result::unwrap()` on an `Err` value: reqwest::Error { kind: Request, url: Url { scheme: "http", cannot_be_a_base: false, username: "", password: None, host: Some(Domain("gateway_api")), port: Some(5433), path: "/enforcer_metadata", query: None, fragment: None }, source: hyper_util::client::legacy::Error(Connect, ConnectError("tcp connect error", Os { code: 111, kind: ConnectionRefused, message: "Connection refused" })) }
```

So make sure to start the gateway-api first.

```shell
docker compose up gateway_api
```

Then you can start the enforcer.

```shell
docker compose up enforcer
```

You might run into `Out of gas` error which runs every 12 seconds:

```shell
enforcer  | challenge_string: MetadatPayload { data: Data { challenge: "HS0IEPBteLwtQmhjYsjpSmyeXoG5RK" } }
enforcer  | 2024-06-21T15:22:59.164437Z ERROR enforcer: Failed to submit validity condition: MiddlewareError { e: MiddlewareError(JsonRpcClientError(JsonRpcError(JsonRpcError { code: -32003, message: "Out of gas: gas required exceeds allowance: 0", data: None }))) }
enforcer  | 2024-06-21T15:23:11.161009Z ERROR enforcer: Failed to submit validity condition: MiddlewareError { e: MiddlewareError(JsonRpcClientError(JsonRpcError(JsonRpcError { code: -32003, message: "Out of gas: gas required exceeds allowance: 0", data: None }))) }
enforcer  | 2024-06-21T15:23:23.165719Z ERROR enforcer: Failed to submit validity condition: MiddlewareError { e: MiddlewareError(JsonRpcClientError(JsonRpcError(JsonRpcError { code: -32003, message: "Out of gas: gas required exceeds allowance: 0", data: None }))) }
```

## Environment Configuration

TODO - For now see individual app READMEs and/or .env.example files.

# Testing

TODO
