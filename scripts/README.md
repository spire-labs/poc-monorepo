# poc-infra
Scripts related to running the PoC. The responsibilities of these scripts is to:
1. spin up an Anvil node
2. deploy smart contracts to that Anvil node

Please note that all private keys and addresses appearing in this codebase are the default Anvil testing addresses - no real ETH is at stake here.

To run on a mac, execute `./setup_and_run.sh`. This will prepare the necesary environment, and then execute the `demo_setup_script.py` file, which will spin up the anvil node

## Helpful commands for testing
This can be run from a command line to get the current block number
```
curl -X POST http://localhost:8545 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0", "method":"eth_blockNumber", "params":[], "id":1}'
```

This can be run to mine the current block
```
curl -X POST -H "Content-Type: application/json" --data '{"jsonrpc":"2.0","method":"evm_mine","params":[],"id":1}' http://localhost:8545
```