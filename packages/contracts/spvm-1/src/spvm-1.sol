// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "openzeppelin-contracts/contracts/utils/cryptography/SignatureChecker.sol";

import "poc-election-contract/ElectionInterface.sol";
import "poc-preconfirmations-slashing/Slashing.sol";

/// @title Spire PoC Virtual Machine - version 1 - interpreter
/// @author mteam
contract SPVM {
    mapping(string => bool) public initialized_tickers;
    mapping(string => mapping(address => uint16)) public state;
    mapping(address => uint32) public nonces;
    // all historical blocks (blockNumber => Block)
    mapping(uint32 => Block) public blocks;
    uint32 public blockNumber = 0;

    ElectionInterface public electionContract;

    Slashing public slashingContract;

    struct TransactionContent {
        address from;
        uint8 txType; // only first 2 bits used
        bytes txParam; // abi encoded parameters
        uint32 nonce;
    }

    struct MintTransactionParams {
        string tokenTicker;
        address owner;
        uint16 supply;
    }

    struct TransferTransactionParams {
        string tokenTicker;
        address to;
        uint16 amount;
    }

    struct Block {
        SPVMTransaction[] transactions;
        bytes32 blockHash;
        bytes32 parentHash;
        uint32 blockNumber;
        address proposer;
        bytes proposer_signature;
    }

    struct SPVMTransaction {
        TransactionContent txContent;
        bytes32 transactionHash;
        bytes signature;
    }

    constructor() {
        // create genesis block
        Block storage genesisBlock = blocks[0];
        genesisBlock.blockNumber = 0;
        genesisBlock.blockHash = bytes32(0);
        genesisBlock.parentHash = bytes32(0);
    }

    function setElectionContract(ElectionInterface _electionContract) external {
        require(
            address(electionContract) == address(0),
            "Election contract already set"
        );
        electionContract = _electionContract;
    }

    function setSlashingContract(Slashing _slashingContract) external {
        require(
            address(slashingContract) == address(0),
            "Slashing contract already set"
        );
        slashingContract = _slashingContract;
    }

    // Function to set a balance in the nested map
    function setBalance(
        string memory tokenTicker,
        address holder_address,
        uint16 balance
    ) internal {
        initialized_tickers[tokenTicker] = true;
        state[tokenTicker][holder_address] = balance;
    }

    // Function to get a balance from the nested map
    function getBalance(
        string memory tokenTicker,
        address holder_address
    ) public view returns (uint16) {
        return state[tokenTicker][holder_address];
    }

    function executeRawTransaction(bytes memory rawTx) internal {
        TransactionContent memory txContent = abi.decode(
            rawTx,
            (TransactionContent)
        );

        require(checkValidity(txContent), "Invalid transaction");

        if (txContent.txType == 0) {
            MintTransactionParams memory mintParams = abi.decode(
                txContent.txParam,
                (MintTransactionParams)
            );
            setBalance(
                mintParams.tokenTicker,
                mintParams.owner,
                mintParams.supply
            );
        } else if (txContent.txType == 1) {
            TransferTransactionParams memory transferParams = abi.decode(
                txContent.txParam,
                (TransferTransactionParams)
            );
            setBalance(
                transferParams.tokenTicker,
                txContent.from,
                getBalance(transferParams.tokenTicker, txContent.from) -
                    transferParams.amount
            );
            setBalance(
                transferParams.tokenTicker,
                transferParams.to,
                getBalance(transferParams.tokenTicker, transferParams.to) +
                    transferParams.amount
            );
        }
        nonces[txContent.from] += 1;
    }

    function checkValidity(
        TransactionContent memory txContent
    ) internal view returns (bool) {
        if (txContent.txType == 0) {
            MintTransactionParams memory mintParams = abi.decode(
                txContent.txParam,
                (MintTransactionParams)
            );
            require(
                !initialized_tickers[mintParams.tokenTicker],
                "Token already initialized"
            );
        } else if (txContent.txType == 1) {
            TransferTransactionParams memory transferParams = abi.decode(
                txContent.txParam,
                (TransferTransactionParams)
            );
            require(
                initialized_tickers[transferParams.tokenTicker],
                "Token not initialized"
            );
            require(
                state[transferParams.tokenTicker][txContent.from] >=
                    transferParams.amount,
                "Insufficient balance"
            );
        } else {
            revert("Invalid transaction type");
        }
        require(nonces[txContent.from] == txContent.nonce, "Invalid nonce");
        return true;
    }

    // recover signer of transaction from signature
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

    function executeTx(SPVMTransaction memory transaction) internal {
        bytes32 txHash = keccak256(abi.encode(transaction.txContent));
        require(
            txHash == transaction.transactionHash,
            "Invalid transaction hash"
        );
        require(
            validateSignature(
                transaction.transactionHash,
                transaction.signature,
                transaction.txContent.from
            ),
            "Invalid signature"
        );
        executeRawTransaction(abi.encode(transaction.txContent));
    }

    function executeBlockTransactions(SPVMTransaction[] memory txs) internal {
        for (uint i = 0; i < txs.length; i++) {
            // note: reverting transactions revert the entire block
            executeTx(txs[i]);
        }
    }

    function proposeBlock(Block calldata proposed_block) external {
        require(block.number % 2 == 0, "Too early to propose");
        // validate proposer signature
        require(
            validateSignature(
                proposed_block.blockHash,
                proposed_block.proposer_signature,
                proposed_block.proposer
            ),
            "Invalid proposer signature"
        );

        // check that proposer is winner of election if electionContract is set
        if (address(electionContract) != address(0)) {
            require(
                proposed_block.proposer ==
                    electionContract.getWinner(blockNumber),
                "proposer was not winner"
            );
        }

        blockNumber += 1;

        // get most recent block
        Block storage lastBlock = blocks[blockNumber - 1];

        require(
            proposed_block.blockHash ==
                keccak256(
                    abi.encodePacked(
                        proposed_block.parentHash,
                        abi.encode(proposed_block.transactions)
                    )
                ),
            "Invalid block hash"
        );
        require(
            proposed_block.blockNumber == lastBlock.blockNumber + 1,
            "Invalid block number"
        );
        require(
            proposed_block.parentHash == lastBlock.blockHash,
            "Invalid parent hash"
        );

        // check that the proposed block includes all transactions from the previous validity conditions
        if (address(slashingContract) != address(0)) {
            Transaction[] memory validity_conditions = slashingContract
                .getValidityConditions(blockNumber - 1);
            uint txs_included = 0;
            for (uint i = 0; i < validity_conditions.length; i++) {
                for (uint j = 0; j < proposed_block.transactions.length; j++) {
                    if (
                        validity_conditions[i].tx_hash ==
                        proposed_block.transactions[j].transactionHash
                    ) {
                        txs_included += 1;
                    }
                }
            }
            require(
                txs_included == validity_conditions.length,
                "Block does not satisfy validity conditions"
            );
        }

        blocks[blockNumber] = proposed_block;

        executeBlockTransactions(proposed_block.transactions);
    }

    /////// Helper functions ///////

    function getTransactionsInBlock(
        uint32 block_number
    ) public view returns (SPVMTransaction[] memory) {
        Block storage b = blocks[block_number];
        return b.transactions;
    }
}
