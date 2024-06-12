# poc-monorepo
Spire PoC Monorepo!

[General PoC](https://www.notion.so/spirelabs/Spire-PoC-Infrastructure-9caebb8915f24a1fba9caf1365b05737)


TODO (megan): 
- [ ] finish creating a shared docker-compose.yml and Dockerfile for all rust executables in monorepo (they live in the apps directory)
- [ ] set up github actions (CI?) for PRs and pushes to main
- [ ] make sure the apps are up to date with the latest in all individual repos, announce to the team th
- [ ] test out functionality with hosted Anvil instance
- [ ] bring env vars out of individual apps into one consolidated file
- [ ] add Spire wallet and poc-infra apps to monorepo (useful commands for this include `rm -rf .git` inside the folder for the app added, then `git rm --cached path/to/app from parent folder, followed by `git add path/to/app)
- [ ] write additional scripts (makefile?) to spin up all apps in monorepo
- [ ] add setup instruction in READMEs all apps for development purposes
- [ ] add TODOs in all READMEs for getting the monorepo ready for release

# Dev Setup

Note: Having a working docker installation is required.

Start docker:
```shell
docker network create shared_network && docker-compose up
```

TODO


## Environment Configuration

TODO - For now see individual app READMEs and/or .env.example files.

# Testing

TODO
