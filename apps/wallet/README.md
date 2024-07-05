# Spire PoC Wallet
This is a simple React frontend which creates transactions and sends them to the server pointed to by the `REACT_APP_API_URL` env var.

## Setup
To run locally, clone this repo and run:

```
npm i && npm run start
```


# Notes

This repo uses ethersjs to generate addresses and keys. It insecurely stores keys client side, and should be used only for demonstration purposes. DO NOT USE THIS IN PRODUCTION
