// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./PreconfirmationsTypes.sol";
import "election-contract/ElectionContract.sol";
import "@openzeppelin/contracts/utils/cryptography/SignatureChecker.sol";

contract Slashing {
    ElectionContract public election_contract;
    address public enforcer;

    mapping(uint256 => Transaction[]) public validity_conditions;

    constructor(address enforcerAddress) {
        election_contract = new ElectionContract(address(this));
        enforcer = enforcerAddress;
        // mint 100 tickets to provided enforecer address
        for (uint256 i = 0; i < 100; i++) {
            election_contract.mintTicket(enforcer);
        }
        // set default recipient of used tickets
        election_contract.setDefaultRecipient(enforcer);
    }

    // checks that a preconfirmations commitment was in submitted validity conditions
    // returns true if the enforcer should be slashed
    function slash(
        PreconfirmationCommitment calldata commitment
    ) external view returns (bool) {
        // check if the signature is valid
        require(
            validateSignature(
                keccak256(abi.encode(commitment.preconfirmation_request)),
                commitment.commitment,
                commitment.signer
            ),
            "Invalid signature"
        );
        // check that the preconfirmation is for this contract
        require(
            commitment.preconfirmation_request.preconf_contract ==
                address(this),
            "Invalid preconfirmation contract address"
        );

        // check if the preconfirmation is for a block number where the enforcer was the signer
        require(
            election_contract.getWinner(commitment.block_number) ==
                commitment.signer,
            "Enforcer was not the elected Enforcer for the committed block number"
        );

        // check if the preconfirmation was honored
        bool transaction_hash_in_block = false;
        for (
            uint256 i = 0;
            i < validity_conditions[commitment.block_number].length;
            i++
        ) {
            if (
                validity_conditions[commitment.block_number][i].tx_hash ==
                commitment.preconfirmation_request.tx_content.tx_hash
            ) {
                transaction_hash_in_block = true;
            }
        }

        // if the preconfirmation was honored
        require(!transaction_hash_in_block, "The preconfirmation was honored");

        // if the preconfirmation was not honored
        return true;
    }

    function submitValidityConditions(
        Transaction[] calldata transactions
    ) external {
        require(block.number % 2 == 1, "only accept validity conditions on odd L1 block numbers");
        require(
            election_contract.getWinner(block.number) == msg.sender,
            "Only the elected enforcer can submit validity conditions"
        );
        for (uint256 i = 0; i < transactions.length; i++) {
            Transaction memory transaction = transactions[i];
            // check that the transaction signature is valid
            require(
                validateSignature(
                    transaction.tx_hash,
                    transaction.signature,
                    transaction.tx_content.from
                ),
                "Invalid transaction signature"
            );
            // check that the transaction hash is correct
            bytes32 txHash = keccak256(abi.encode(transaction.tx_content));
            require(
                txHash == transaction.tx_hash,
                "Invalid transaction hash"
            );
            validity_conditions[block.number].push(transaction);
        }
    }

    function getValidityConditions(
        uint256 L1_block_number
    ) external view returns (Transaction[] memory) {
        return validity_conditions[L1_block_number];
    }

    // HELPER FUNCTIONS //

    function validateSignature(
        bytes32 message_hash,
        bytes memory signature,
        address expected_signer
    ) internal view returns (bool) {
        return
            SignatureChecker.isValidSignatureNow(
                expected_signer,
                message_hash,
                signature
            );
    }
}
