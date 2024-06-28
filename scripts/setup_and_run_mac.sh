#!/bin/bash


# Set permissions for the SSH private key file. Only use this if you don't already have an ssh key registered with github
# chmod 600 /home/username/.ssh/id_ed25519

# Install forge TODO (this breaks. need to run by hand)



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
