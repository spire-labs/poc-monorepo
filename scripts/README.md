# poc-infra
Scripts and infra related to POC

## scripts/anvil-testnet.py
[wip]
Python script used for setting up testnet for PoC demo.

Currently does the following:
- clones or pulls latest versions of contract repos
- builds contract repos
- deploys spvm contract
- sets election contract value in spvm

Todos:
- determine initial state(s) for other contracts which need to be deployed for PoC
- test


The script also creates a Flask server running on port 5000. This REST server can be used to query information about deployed contract addresses.

The deployed version currently lives at this IP
```
curl http://34.16.19.248:5000/contracts
```

## Notes:
(These are out of date as of 5/28/2024 - they will be updated)
- VM needs to have Flask installed `sudo apt install -y python3-flask`
- script will attempt to pull latest `main` for each repo using ssh. If using `.gitmodules`, please ensure that any spire repos have their `url` set to use `ssh` instead of `https`.
- If updating .gitmodules locally when developing, you'll need to sync them for each repo for  changes to take effect:
```
git submodule sync
git submodule update --init --recursive
```
- Script uses web3.py to listen for new blocks and execute methods on election contracts. Web3 does not install nicely via apt-get, so a virtualenv needs to be used instead. Script is currently (5/16/24) being updated to ensure this is done reliably

## Helpful commands for testing
This can be run from a command line to get the current block number
```
curl -X POST http://34.16.19.248:8545 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0", "method":"eth_blockNumber", "params":[], "id":1}'
```

This can be run to mine the current block
```
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"evm_mine","params":[],"id":1}' http://34.16.19.248:8545
```