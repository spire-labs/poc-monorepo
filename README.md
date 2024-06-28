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

Note: Having a working docker installation is required.

Start docker:
```shell
docker-compose up
```

TODO

## Special instructions for running scripts
The scripts in /scripts can be used to spin up an Anvil node and populate it with contracts. There are two in particular:

`scripts/setup_and_run.sh`
This is used to install all dependencies for the projects, spin up a python virtual env, and then execute `demo_setup_script.py`, which will deploy contracts to anvil and populate them with some initial data. Note that this script is designed to run on Linux, as it is using apt for its package manager. If you are on a Mac, additional instructions will be added below.

Note that this script used to be responsible for spinning up Rust repos as well as contracts. Now that we have moved to using a monorepo, the script no longer handles any of the rust projects or their dependencies.

`demo_setup_script.py`
This will spin up an anvil node, deploy an ERC20 contract, and deploy the Spire contracts.


## running scripts locally on a mac instructions
To get things up and running quickly on a local mac, some one-time steps are done manually to simplify the script.  use the following instructions, starting in the root of the monorepo project.

- Install forge/foundry if not already done
-- curl -L https://foundry.paradigm.xyz | bash
-- source ~/.bashrc
-- foundryup
- from the root of the monorepo project, run `forge install OpenZeppelin/openzeppelin-contracts`
- run `forge build` to compile the ERC20 contract used in Demo 3
- Ensure that you have your github ssh credentials set up locally. then `cd scripts` and then run `./setup_and_run_mac.sh`. This should pull down all of the smart contract repos, compile, and run anvil

## Environment Configuration

TODO - For now see individual app READMEs and/or .env.example files.

# Testing

TODO
