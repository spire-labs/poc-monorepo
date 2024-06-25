#!/bin/bash

# Update package list and install necessary packages
sudo apt-get update
sudo apt-get install -y python3-venv python3-pip git curl build-essential pkg-config libssl-dev

# Set permissions for the SSH private key file
# chmod 600 /home/username/.ssh/id_ed25519

# Install forge TODO (this breaks. need to run by hand)
# curl -L https://foundry.paradigm.xyz | bash
# source ~/.bashrc
# foundryup

# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

# Export Rust environment variables
export PATH="$HOME/.cargo/bin:$PATH"

# Set up the virtual environment
python3 -m venv venv

# Activate the virtual environment
source venv/bin/activate

# Upgrade pip
pip install --upgrade pip

# Install the dependencies
pip install -r requirements.txt

# Run the main script
python demo_setup_script.py

# Deactivate the virtual environment
deactivate
