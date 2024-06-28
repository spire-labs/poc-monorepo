#!/bin/bash


# Set permissions for the SSH private key file. Only use this if you don't already have an ssh key registered with github
# chmod 600 /home/username/.ssh/id_ed25519

# Install forge TODO (this breaks. need to run by hand)
curl -L https://foundry.paradigm.xyz | bash
source ~/.bashrc
foundryup


# Set up the virtual environment
python3 -m venv venv

# Activate the virtual environment
source venv/bin/activate

# Upgrade pip
pip install --upgrade pip

# Install the dependencies
pip install -r requirements.txt

echo "Installing OpenZeppelin contracts"
forge install OpenZeppelin/openzeppelin-contracts

# For demo 3, we need to compile a local erc20
echo "Compiling ERC20 contract"
forge build

# Run the main script
python demo_setup_script.py

# Deactivate the virtual environment
deactivate
