from flask import Flask, jsonify
import os
import subprocess
import re
import sys
import venv
import json
from pathlib import Path

app = Flask(__name__)

# Variables to hold contract addresses. Retrievable via Flask API
chain_a_spvm_address = None
chain_b_spvm_address = None
chain_a_election_address = None
chain_b_election_address = None
chain_a_slashing_address = None
chain_b_slashing_address = None

spvm_contract_abi = None
election_contract_abi = None
slashing_contract_abi = None
# TODO: For testing purposes until gateway is setup
spvm_test_contract = None

DEFAULT_ANVIL_UNLOCKED_ADDRESS = "0xf39Fd6e51aad88F6F4ce6aB8827279cffFb92266"

DEFAULT_AVNIL_USER_WALLET_ADDRESS = "0xa0Ee7A142d267C1f36714E4a8F75612F20a79720"
DEFAULT_ANVIL_USER_WALLET_PK = "0x2a871d0798f97d79848a013d4936a73bf4cc922c825d33c1cf7073dff6d409c6"

def run_command(command):
    try:
        result = subprocess.run(command, shell=True, check=True, text=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        print(result.stdout)
        return result.stdout
    except subprocess.CalledProcessError as e:
        print(f"Error executing {' '.join(command)}:")
        print(f"Return code: {e.returncode}")
        print(f"Error output: {e.stderr}")
        print(f"Standard output: {e.stdout}")
        # For now, continue executing script
        # raise

def clone_or_pull_repo(repo_path, repo_url, branch='main'):
    if not repo_path.exists():
        print(f"Cloning {repo_url}")
        run_command(f"git clone {repo_url} {repo_path}")
    else:
        print(f"Pulling latest version of {repo_path.name}")
        run_command(f"git -C {repo_path} pull origin {branch}")

def extract_deployed_address(output):
    """Extracts the 'Deployed to:' address from the output"""
    match = re.search(r"Deployed to: (\S+)", output)
    if match:
        return match.group(1)
    else:
        print("No deployed address found in output.")
        return None

def load_abi(base_dir, abi_path):
    with open(f"{base_dir}{abi_path}", 'r') as abi_file:
        return json.load(abi_file)

def update_cargo_toml(file_path):
    with open(file_path, 'r') as file:
        content = file.read()
    
    # Define the regex patterns and replacements for the enforcer
    patterns = {
        r'spvm-rs = { path = "spvm-rs" }': r'spvm-rs = { path = "../spvm-rs" }',
        r'entity = { version = "0.1.0", path = "spvm-rs/entity" }': r'entity = { version = "0.1.0", path = "../spvm-rs/entity" }'
    }
    
    for pattern, replacement in patterns.items():
        content = re.sub(pattern, replacement, content)
    
    with open(file_path, 'w') as file:
        file.write(content)

def set_election_contract(deployed_contract_address, election_contract_address, port=8545):
    """Sets the Election contract address on SPVM contract"""
    command = f"cast send {deployed_contract_address} 'setElectionContract(address)' {election_contract_address} --rpc-url http://localhost:{port} --unlocked --from {DEFAULT_ANVIL_UNLOCKED_ADDRESS}"
    print("Executing command:", command)
    return run_command(command)

# This method does the following (TODO)
# Takes addresses of two proposers, ProposerA and ProposerB (TODO: how to get these addrs?)
# Takes the address of a deployed Proposer election contract
# Distributes 50 tickets to each of these proposers through the election contract
# calls setDefaultRecip and points to one of the propsers
def initialize_proposers(election_contract, proposer_a_address, proposer_b_address, web3, tickets_to_mint = 50):
    for i in range(0, tickets_to_mint):
        try:
            tx_hash_a = election_contract.functions.mintTicket(proposer_a_address).transact({'from': web3.eth.accounts[0]})
            receipt_a = web3.eth.wait_for_transaction_receipt(tx_hash_a)
            tx_hash_b = election_contract.functions.mintTicket(proposer_b_address).transact({'from': web3.eth.accounts[0]})
            receipt_b = web3.eth.wait_for_transaction_receipt(tx_hash_b)
        except Exception as e:
            print(f"Error distributing election tickets to proposers {e}")
    
    # set default recip to proposer a (arbitrarily)
    try:
        tx_hash_set_default = election_contract.functions.setDefaultRecipient(proposer_a_address).transact({'from': web3.eth.accounts[0]})
    except Exception as e:
        print(f"Error setting default recipient of proposal tickets {e}")
    
    print("Successfully issued proposal tickets")

# Mint a few tokens on appchains A and B
# transfer some of the tokens to the demo wallet address
# TODO: WIP
def initialize_tokens(spvm_contract, spvm_address, web3):

    try:
        # check initial wallet balance
        check_balance_before = spvm_contract.functions.getBalance("RAIN", DEFAULT_AVNIL_USER_WALLET_ADDRESS).call()
    except:
        print(f"Error checking initial tx setup")

    try:
        tx_hash = spvm_contract.functions.testExecuteRawMintTransaction().transact({'from': web3.eth.accounts[0]})
        recp = web3.eth.wait_for_transaction_receipt(tx_hash)
    except:
        print(f"Error executing test mint txs {e}")
    
    print(f"successfully initialized tokens {tx_hash} and {recp}")
    # check initialized balances, for testing purposes
    try:
        check_balance_after = spvm_contract.functions.getBalance("RAIN", DEFAULT_AVNIL_USER_WALLET_ADDRESS).call()
        
    except:
        print(f"Error checking tx setup")
    print(f"successfully checked balance {check_balance_before} and after {check_balance_after}")

# TODO: This will be replaced by calls to the Enforcer
# def initialize_tokens_by_chain(spvm_contract, spvm_address, token_name, initial_amount=100, initial_recipient=DEFAULT_ANVIL_UNLOCKED_ADDRESS, web3):
#     try:
#         # check initial wallet balance
#         check_balance_before = spvm_contract.functions.getBalance(token_name, initial_recipient).call()
#     except:
#         print(f"Error checking token setup for mint of {initial_amount} of {token_name} to {initial_recipient}")

#     try:
#         tx_hash = spvm_contract.functions.testExecuteRawMintTransaction().transact({'from': web3.eth.accounts[0]})
#         recp = web3.eth.wait_for_transaction_receipt(tx_hash)
#     except:
#         print(f"Error executing test mint txs {e}")
    
#     print(f"successfully initialized tokens {tx_hash} and {recp}")
#     # check initialized balances, for testing purposes
#     try:
#         check_balance_after = spvm_contract.functions.getBalance(token_name, initial_recipient).call()
        
#     except:
#         print(f"Error checking tx setup")
#     print(f"successfully checked balance {check_balance_before} and after {check_balance_after}")


# Function to call on each new anvil block
def on_new_block(block, election_contract, web3):
    print(f"New block: {block['number']}")

    # call necessary functions on election contracts
    try:
        tx_hash = election_contract.functions.refreshTickets().transact({'from': web3.eth.accounts[0]})
        receipt = web3.eth.wait_for_transaction_receipt(tx_hash)
        print(f"Function call receipt: {receipt}")
    except Exception as e:
        print(f"Error calling function: {e}")
    
# Listener for new blocks from Anvil node
def handle_new_block(block, election_contract, web3):
    on_new_block(block, election_contract, web3)

def start_anvil(port):
    print(f"Starting Anvil on port {port}...")
    anvil_command = f"nohup anvil --host 0.0.0.0 --port {port} > ~/anvil_logs_{port}.log 2>&1 &"
    anvil_process = subprocess.Popen(anvil_command, shell=True)
    print(f"Anvil started in the background on port {port}. PID: {anvil_process.pid}")
    return anvil_process

def build_rust_project(project_dir):
    project_dir = os.path.expanduser(project_dir)
    os.chdir(project_dir)
    result = subprocess.run(["cargo", "build"], check=True)
    print("Build completed with return code:", result.returncode)

def run_rust_project(project_dir):
    project_dir = os.path.expanduser(project_dir)
    os.chdir(project_dir)
    with open("nohup.out", "w") as output_file:
        process = subprocess.Popen(
            ["nohup", "cargo", "run"],
            stdout=output_file,
            stderr=subprocess.STDOUT,
            preexec_fn=subprocess.os.setpgrp,  # Ensure the process runs independently from the parent
            env=os.environ
        )
    print(f"Rust API running in background with PID: {process.pid}")


# public api routes
# TODO: Update to include chain information as well
@app.route('/contracts', methods=['GET'])
def get_contract_info():
    return jsonify({
        'chain_a': {
            'spvm': {
                'address': chain_a_spvm_address,
                'abi': spvm_contract_abi["abi"]
            },
            'election': {
                'address': chain_a_election_address,
                'abi': election_contract_abi["abi"]
            },
            'slashing': {
                'address': chain_a_slashing_address,
                'abi': slashing_contract_abi["abi"]
            }
        },
        'chain_b': {
            'spvm': {
                'address': chain_b_spvm_address,
                'abi': spvm_contract_abi["abi"]
            },
            'election': {
                'address': chain_b_election_address,
                'abi': election_contract_abi["abi"]
            },
            'slashing': {
                'address': chain_b_slashing_address,
                'abi': slashing_contract_abi["abi"]
            }
        }
    })

# TODO: helper method until gateway is ready
@app.route('/test-wallet-balance', methods=['GET'])
def get_wallet_balance():
    global spvm_test_contract
    rain_balance = spvm_test_contract.functions.getBalance("RAIN", DEFAULT_AVNIL_USER_WALLET_ADDRESS).call()
    queen_balance = spvm_test_contract.functions.getBalance("QUEEN", DEFAULT_AVNIL_USER_WALLET_ADDRESS).call()
    infinity_balance = spvm_test_contract.functions.getBalance("INFINITY", DEFAULT_AVNIL_USER_WALLET_ADDRESS).call()
    # TODO: Does this call the enforcer instead to create initial balances?
    return jsonify({
        'balance': balance,
        'rain': rain_balance,
        'queen': queen_balance,
        'infinity': infinity_balance
    })

def main():
    global chain_a_spvm_address, chain_b_spvm_address, chain_a_election_address, chain_b_election_address, chain_a_slashing_address, chain_b_slashing_address, spvm_test_contract, spvm_contract_abi, election_contract_abi, slashing_contract_abi
    home = Path.home()
    base_dir = home / "spire-poc/repos"
    
    # from flask import Flask, jsonify
    from web3 import Web3
    from flask_cors import CORS

    
    # Ensure base directory exists
    base_dir.mkdir(parents=True, exist_ok=True)
    
    # Repositories information
    repos = [
        ("spvm-1", "git@github.com:spire-labs/spvm-1.git"),
        ("poc-preconfirmations-slashing", "git@github.com:spire-labs/poc-preconfirmations-slashing.git"),
        ("poc-election-contract", "git@github.com:spire-labs/poc-election-contract.git"),
    ]

    # Clone or pull repositories
    # TODO: is it beter to rm -rf and rebuild repos dirs each time?
    for repo_name, repo_url in repos:
        clone_or_pull_repo(base_dir / repo_name, repo_url)

    # Build contracts in each repo
    print("Building and compiling contracts")
    for repo_name, _ in repos:
        print(f"Building in {repo_name}")
        run_command(f"cd {base_dir / repo_name} && forge clean && forge update && forge build")

    
    abi_paths = {
        "spvm": "/spvm-1/out/spvm-1.sol/SPVM.json",
        "election": "/poc-election-contract/out/ElectionContract.sol/ElectionContract.json",
        "slashing": "/poc-preconfirmations-slashing/out/Slashing.sol/Slashing.json"
    }

    spvm_contract_abi = load_abi(base_dir, abi_paths["spvm"])
    print("loaded spvm abi")

    election_contract_abi = load_abi(base_dir, abi_paths["election"])
    print("loaded election abi")

    slashing_contract_abi = load_abi(base_dir, abi_paths["slashing"])
    print("loaded slashing abi")

    CHAIN_A_PORT = 8545

    # Start Anvil chains
    anvil_chain_1 = start_anvil(CHAIN_A_PORT)
    # anvil_chain_2 = start_anvil(CHAIN_A_PORT)

    # connect to anvil instance and execute methods on every new block
    chain_a_anvil_url = f"http://0.0.0.0:{CHAIN_A_PORT}"
    chain_a_web3 = Web3(Web3.HTTPProvider(chain_a_anvil_url))

    if not chain_a_web3.is_connected():
        print("Failed to connect to the Chain A Anvil instance")
    

    # Deploy contracts
    print("Deploying SPVM...")
    spvm_repo_path = base_dir / "spvm-1"
    chain_a_spvm_deploy_output = run_command(f"cd {spvm_repo_path} && forge create src/spvm-1.sol:SPVM --rpc-url http://localhost:{CHAIN_A_PORT} --unlocked --from {DEFAULT_ANVIL_UNLOCKED_ADDRESS}")

    chain_a_spvm_address = extract_deployed_address(chain_a_spvm_deploy_output)

    if chain_a_spvm_address:
        print(f"[CHAIN A]:Deployed spvm contract address: {chain_a_spvm_address}")
    else:
        raise "CHAIN A]: error deploying SPVM contract"

    chain_b_spvm_deploy_output = run_command(f"cd {spvm_repo_path} && forge create src/spvm-1.sol:SPVM --rpc-url http://localhost:{CHAIN_A_PORT} --unlocked --from {DEFAULT_ANVIL_UNLOCKED_ADDRESS}")

    chain_b_spvm_address = extract_deployed_address(chain_b_spvm_deploy_output)

    if chain_b_spvm_address:
        print(f"[CHAIN B]:Deployed spvm contract address: {chain_b_spvm_address}")
    else:
        raise "[CHAIN B]: error deploying SPVM contract"

    # deploy election contract
    # TODO: Set this to something realistic
    test_minter_address = chain_a_web3.eth.accounts[0]#"0x70997970C51812dc3A010C7d01b50e0d17dc79C8"

    election_repo_path = base_dir / "poc-election-contract"

    chain_a_election_deploy_output = run_command(f"cd {election_repo_path} && forge create src/ElectionContract.sol:ElectionContract --constructor-args {test_minter_address} --rpc-url http://localhost:{CHAIN_A_PORT} --unlocked --from {DEFAULT_ANVIL_UNLOCKED_ADDRESS}")
    
    chain_a_election_address = extract_deployed_address(chain_a_election_deploy_output)

    if chain_a_election_address:
        print(f"Deployed election contract address: {chain_a_election_address}")
    else:
        raise "error deploying election contract"
    
    # TODO: ?
    set_election_output = set_election_contract(chain_a_spvm_address, chain_a_election_address, port=CHAIN_A_PORT)
    print(set_election_output)


    chain_b_election_deploy_output = run_command(f"cd {election_repo_path} && forge create src/ElectionContract.sol:ElectionContract --constructor-args {test_minter_address} --rpc-url http://localhost:{CHAIN_A_PORT} --unlocked --from {DEFAULT_ANVIL_UNLOCKED_ADDRESS}")
    
    chain_b_election_address = extract_deployed_address(chain_b_election_deploy_output)

    if chain_b_election_address:
        print(f"Deployed election contract address: {chain_b_election_address}")
    else:
        raise "error deploying election contract"
    
    # TODO: ?
    set_election_output = set_election_contract(chain_b_spvm_address, chain_b_election_address, port=CHAIN_A_PORT)
    print(set_election_output)


    # deploy slashing contract
    slashing_repo_path = base_dir / "poc-preconfirmations-slashing"

    test_enforcer_address = chain_a_web3.eth.accounts[0]

    chain_a_slashing_deploy_output = run_command(f"cd {slashing_repo_path} && forge create src/Slashing.sol:Slashing --constructor-args {test_enforcer_address} --rpc-url http://localhost:{CHAIN_A_PORT} --unlocked --from {DEFAULT_ANVIL_UNLOCKED_ADDRESS} --gas-limit 20000000")

    chain_a_slashing_address = extract_deployed_address(chain_a_slashing_deploy_output)

    if chain_a_slashing_address:
        print(f"Deployed slashing contract address: {chain_a_slashing_address}")
    else:
        raise "error deploying slashing contract"

    
    chain_b_slashing_deploy_output = run_command(f"cd {slashing_repo_path} && forge create src/Slashing.sol:Slashing --constructor-args {test_enforcer_address} --rpc-url http://localhost:{CHAIN_A_PORT} --unlocked --from {DEFAULT_ANVIL_UNLOCKED_ADDRESS} --gas-limit 20000000")

    chain_b_slashing_address = extract_deployed_address(chain_b_slashing_deploy_output)

    if chain_b_slashing_address:
        print(f"Deployed slashing contract address: {chain_b_slashing_address}")
    else:
        raise "error deploying slashing contract"

    # create a web3 instance of the election contract
    chain_a_election_contract = chain_a_web3.eth.contract(address=chain_a_election_address, abi=election_contract_abi["abi"])
    chain_b_election_contract = chain_a_web3.eth.contract(address=chain_b_election_address, abi=election_contract_abi["abi"])

    # create a web3 instance of the spvm contract
    chain_a_spvm_contract = chain_a_web3.eth.contract(address=chain_a_spvm_address, abi=spvm_contract_abi["abi"])
    chain_b_spvm_contract = chain_a_web3.eth.contract(address=chain_b_spvm_address, abi=spvm_contract_abi["abi"])

    # TODO: for testing purposes until gateway is setup
    spvm_test_contract = chain_a_web3.eth.contract(address=chain_a_spvm_address, abi=spvm_contract_abi["abi"])

    # create a chain_a_web3 instance of the slashing contract
    chain_a_slashing_contract = chain_a_web3.eth.contract(address=chain_a_slashing_address, abi=slashing_contract_abi["abi"])
    chain_b_slashing_contract = chain_a_web3.eth.contract(address=chain_b_slashing_address, abi=slashing_contract_abi["abi"])


    # initialize proposers for chain A
    initialize_proposers(chain_a_election_contract, proposer_a_address="0x70997970C51812dc3A010C7d01b50e0d17dc79C8", proposer_b_address="0x3C44CdDdB6a900fa2b585dd299e03d12FA4293BC", web3=chain_a_web3)
    # initialize proposers for chain B
    initialize_proposers(chain_b_election_contract, proposer_a_address="0x90F79bf6EB2c4f870365E785982E1f101E93b906", proposer_b_address="0x15d34AAf54267DB7D7c367839AAf71A00a2C6A65", web3=chain_a_web3)

    # test initialize some mints
    # initialize_tokens(chain_a_spvm_contract, chain_a_spvm_address, chain_a_web3)
    # TODO: Initialize Chain B token mints as well
    # initialize_tokens(chain_b_spvm_contract, chain_b_spvm_address, chain_b_web3)

    # Initialize token balances on each chain
    # - RAIN on appchain A
    # initialize_tokens_by_chain(chain_a_spvm_contract, chain_a_spvm_address, token_name="RAIN", initial_amount=100, initial_recipient=DEFAULT_ANVIL_UNLOCKED_ADDRESS, chain_a_web3)
    # - QUEEN on appchain A
    # - INFINITY on appchain A
    # - SUNSET on appchain B

    
    # Subscribe to new block headers
    chain_a_block_filter = chain_a_web3.eth.filter('latest')
    # start API
    CORS(app)
    app.run(host='0.0.0.0', port=5000)
    print("Listening for new blocks...")
    while True:
        for block_hash in chain_a_block_filter.get_new_entries():
            block = chain_a_web3.eth.get_block(block_hash)
            # TODO: SPVM blocks happen on even L1 block numbers, validity conditions are posted on odd L1 block numbers.
            handle_new_block(block, chain_a_election_contract, chain_a_web3)


if __name__ == "__main__":
    main()
    # app.run(host='0.0.0.0', port=5000)