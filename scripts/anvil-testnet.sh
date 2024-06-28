#!/bin/bash

# Note, ensure VM has valid ssh creds for github, otherwise this script will fail
# Note, assumes forge and anvil are already installed on VM

# NOTE: THIS FILE IS ARCHIVED AND IS NO LONGER IN USE. USE setup_and_run.sh instead


# clone repos if they do not exist
if [ ! -d "$HOME/spire-poc/repos" ]; then
  echo "Cloning repos"
  mkdir -p "$HOME/spire-poc/repos"
  cd "$HOME/spire-poc/repos"
  # Clone each repository
  git clone git@github.com:spire-labs/spvm-1.git
  git clone git@github.com:spire-labs/poc-preconfirmations-slashing.git
  git clone git@github.com:spire-labs/poc-election-contract.git
else
  echo "Pulling latest versions of repos"
  cd "$HOME/spire-poc/repos"
  # pull the latest for each repo

  cd spvm-1
  git pull origin main

  cd ../poc-preconfirmations-slashing
  git pull origin main

  cd ../poc-election-contract
  git pull origin main

fi



# build contracts in each repo
echo "Building and compiling contracts"
cd "$HOME/spire-poc/repos"

cd spvm-1
forge build

cd ../poc-preconfirmations-slashing
forge build

cd ../poc-election-contract
forge build

# start anvil
echo "Starting Anvil..."
nohup anvil > ~/anvil_logs.log 2>&1 &
echo "Anvil started in the background. PID: $!"

cd "$HOME/spire-poc/repos"

# deploy contracts
# TODO: Consider moving into deployment script instead
cd spvm-1
echo "Deploying SPVM..."
forge create src/spvm-1.sol --rpc-url http://localhost:8545