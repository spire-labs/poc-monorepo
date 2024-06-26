// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

struct PreconfirmationRequest {
    // The signed transaction to be preconfirmed
    Transaction tx_content;
    // A signed transaction that delivers a tip to the preconfer (optional, leave all fields empty if not needed)
    Transaction tip_tx;
    // The address of the preconfirmations slashing contract where this preconfirmation is relevant
    address preconf_contract;
}

struct PreconfirmationCommitment {
    // A PreconfirmationRequest object.
    PreconfirmationRequest preconfirmation_request;
    // An ECDSA signature of a keccak256 hash of the abi-encoded preconfirmation_request
    bytes commitment;
    // The address of the enforcer that signed the commitment
    address signer;
    // The block number for which the preconfirmation is relevant
    uint256 block_number;
}

struct Transaction {
    // The hash of the transaction
    bytes32 tx_hash;
    // The abi encoded body of the transaction
    PreconfTransactionContent tx_content;
    // The signature of the transaction
    bytes signature;
}

struct PreconfTransactionContent {
    address from;
    uint8 txType; // only first 2 bits used
    bytes txParam; // abi encoded parameters
    uint32 nonce;
}