// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "forge-std/Test.sol";

import "../src/spvm-1.sol";
import "poc-election-contract/ElectionInterface.sol";

contract SPVMTest is Test, SPVM {
    uint256 internal pk;
    address internal signer;

    function setUp() public {
        pk = 0x1234567890abcdef;
        signer = vm.addr(pk);
    }

    function createTransaction(
        address _signer,
        uint8 _type,
        bytes memory _params,
        uint32 _nonce
    ) internal view returns (SPVMTransaction memory) {
        bytes memory rawTx = abi.encode(
            TransactionContent(_signer, _type, _params, _nonce)
        );
        bytes32 txHash = keccak256(rawTx);
        bytes memory signature = signHash(pk, txHash);
        return
            SPVMTransaction(
                TransactionContent(_signer, _type, _params, _nonce),
                txHash,
                signature
            );
    }

    function signHash(uint256 _pk, bytes32 _hash) internal pure returns (bytes memory) {
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(_pk, _hash);
        return abi.encodePacked(r, s, v);
    }

    function encodeRawTransactionContents(
        SPVMTransaction memory _tx
    ) internal pure returns (bytes memory) {
        return abi.encode(_tx.txContent);
    }

    function createMintTransaction(
        string memory _tokenTicker,
        address _from,
        address _owner,
        uint16 _supply,
        uint32 _nonce
    ) internal view returns (SPVMTransaction memory) {
        bytes memory txParam = abi.encode(
            MintTransactionParams(_tokenTicker, _owner, _supply)
        );
        return createTransaction(_from, 0, txParam, _nonce);
    }

    function createTransferTransaction(
        string memory _tokenTicker,
        address _from,
        address _to,
        uint16 _amount,
        uint32 _nonce
    ) internal view returns (SPVMTransaction memory) {
        bytes memory txParam = abi.encode(
            TransferTransactionParams(_tokenTicker, _to, _amount)
        );
        return createTransaction(_from, 1, txParam, _nonce);
    }

    function testGetBalance() external view {
        assertEq(getBalance("TST", address(this)), 0);
    }

    function testSetBalance() external {
        setBalance("TST", address(this), 100);
        assertEq(getBalance("TST", address(this)), 100);

        setBalance("TST", address(1), 200);
        assertEq(getBalance("TST", address(1)), 200);

        setBalance("TST", address(this), 0);
        assertEq(getBalance("TST", address(this)), 0);
    }

    function testExecuteRawMintTransaction() external {
        SPVMTransaction memory tx1 = createMintTransaction(
            "TST",
            address(this),
            address(this),
            100,
            0
        );

        // Execute the transaction
        executeRawTransaction(encodeRawTransactionContents(tx1));
        assertEq(getBalance("TST", address(this)), 100);
        assertEq(getBalance("TST", address(1)), 0);

        SPVMTransaction memory tx2 = createMintTransaction(
            "TST2",
            address(this),
            address(1),
            200,
            1
        );

        // Execute the second transaction
        executeRawTransaction(encodeRawTransactionContents(tx2));
        assertEq(getBalance("TST2", address(1)), 200);
        assertEq(getBalance("TST2", address(this)), 0);
        assertEq(getBalance("TST", address(this)), 100);
    }

    function testFuzz_ExecuteRawMintTransaction(uint16 amount) external {
        SPVMTransaction memory tx1 = createMintTransaction(
            "TST",
            address(this),
            address(this),
            amount,
            0
        );

        // Execute the transaction
        executeRawTransaction(encodeRawTransactionContents(tx1));
        assertEq(getBalance("TST", address(this)), amount);
    }

    function testExecuteRawTransferTransaction() external {
        SPVMTransaction memory tx1 = createMintTransaction(
            "TST",
            address(this),
            address(this),
            100,
            0
        );
        executeRawTransaction(encodeRawTransactionContents(tx1));
        assertEq(getBalance("TST", address(this)), 100);
        assertEq(getBalance("TST", address(1)), 0);

        SPVMTransaction memory tx2 = createMintTransaction(
            "TST2",
            address(1),
            address(1),
            200,
            0
        );
        executeRawTransaction(encodeRawTransactionContents(tx2));
        assertEq(getBalance("TST2", address(1)), 200);
        assertEq(getBalance("TST2", address(this)), 0);
        assertEq(getBalance("TST", address(this)), 100);

        SPVMTransaction memory tx3 = createTransferTransaction(
            "TST",
            address(this),
            address(1),
            50,
            1
        );
        executeRawTransaction(encodeRawTransactionContents(tx3));
        assertEq(getBalance("TST", address(this)), 50);
        assertEq(getBalance("TST", address(1)), 50);

        // self transfer
        SPVMTransaction memory tx4 = createTransferTransaction(
            "TST",
            address(this),
            address(this),
            50,
            2
        );
        executeRawTransaction(encodeRawTransactionContents(tx4));
        assertEq(getBalance("TST", address(this)), 50);
    }

    function testFuzz_ExecuteRawTransferTransaction(
        uint16 mint_amount,
        uint16 send_amount
    ) external {
        vm.assume(mint_amount >= send_amount);
        SPVMTransaction memory tx1 = createMintTransaction(
            "TST",
            address(this),
            address(this),
            mint_amount,
            0
        );
        executeRawTransaction(encodeRawTransactionContents(tx1));
        assertEq(getBalance("TST", address(this)), mint_amount);

        SPVMTransaction memory tx2 = createTransferTransaction(
            "TST",
            address(this),
            address(1),
            send_amount,
            1
        );
        executeRawTransaction(encodeRawTransactionContents(tx2));
        assertEq(getBalance("TST", address(this)), mint_amount - send_amount);
        assertEq(getBalance("TST", address(1)), send_amount);
    }

    // check that function reverts when it should
    function testValidityChecking() external {
        // token already initialized
        SPVMTransaction memory tx1 = createMintTransaction(
            "TST",
            address(this),
            address(this),
            100,
            0
        );
        executeRawTransaction(encodeRawTransactionContents(tx1));
        SPVMTransaction memory tx2 = createMintTransaction(
            "TST",
            address(this),
            address(this),
            100,
            1
        );
        vm.expectRevert(bytes("Token already initialized"));
        executeRawTransaction(encodeRawTransactionContents(tx2));

        // token not initialized
        SPVMTransaction memory tx3 = createTransferTransaction(
            "NOTTST",
            address(this),
            address(1),
            50,
            1
        );
        vm.expectRevert(bytes("Token not initialized"));
        executeRawTransaction(encodeRawTransactionContents(tx3));

        // Insufficient balance
        SPVMTransaction memory tx4 = createTransferTransaction(
            "TST",
            address(this),
            address(1),
            100,
            1
        );
        vm.expectRevert(bytes("Insufficient balance"));
        executeRawTransaction(encodeRawTransactionContents(tx4));

        SPVMTransaction memory tx5 = createTransferTransaction(
            "TST",
            address(1),
            address(this),
            100,
            1
        );
        vm.expectRevert(bytes("Insufficient balance"));
        executeRawTransaction(encodeRawTransactionContents(tx5));

        // Invalid transaction
        bytes memory txParam6 = abi.encode(
            MintTransactionParams("TST", address(this), 100)
        );
        bytes memory rawTx6 = abi.encode(
            TransactionContent(address(this), 2, txParam6, 1)
        );
        vm.expectRevert(bytes("Invalid transaction type"));
        executeRawTransaction(rawTx6);

        // invalid nonce
        SPVMTransaction memory tx7 = createMintTransaction(
            "TST2",
            address(this),
            address(this),
            100,
            0
        );
        vm.expectRevert(bytes("Invalid nonce"));
        executeTx(tx7);
    }

    function testValidateSignature() external view {
        bytes32 tx_hash = keccak256(abi.encodePacked("test"));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(pk, tx_hash);
        bytes memory signature = abi.encodePacked(r, s, v); // note the order here is different from line above.

        assert(validateSignature(tx_hash, signature, signer));
    }

    function testInvalidSignature() view external {
        bytes32 tx_hash = keccak256(abi.encodePacked("test"));

        bytes memory signature = bytes("test");
        assert(!validateSignature(tx_hash, signature, signer));
    }

    // test executeTx
    function testExecuteTx() external {
        SPVMTransaction memory Tx = createMintTransaction(
            "TST",
            signer,
            signer,
            100,
            0
        );
        executeTx(Tx);
        assertEq(getBalance("TST", signer), 100);
    }

    ////// BLOCK ///////
    function testInitialBlockState() external view {
        assert(blocks[0].blockNumber == 0);
        assert(blocks[0].parentHash == bytes32(0));
        assert(blocks[0].blockHash == bytes32(0));
        assert(blocks[0].transactions.length == 0);
    }

    function testExecuteBlockTransactions() external {
        SPVMTransaction memory Tx = createMintTransaction(
            "TST",
            signer,
            signer,
            100,
            0
        );

        SPVMTransaction memory Tx2 = createMintTransaction(
            "TST2",
            signer,
            signer,
            200,
            1
        );

        // transfer
        SPVMTransaction memory Tx3 = createTransferTransaction(
            "TST2",
            signer,
            address(1),
            50,
            2
        );

        SPVMTransaction[] memory txs = new SPVMTransaction[](3);
        txs[0] = Tx;
        txs[1] = Tx2;
        txs[2] = Tx3;

        executeBlockTransactions(txs);
        assertEq(getBalance("TST", signer), 100);
        assertEq(getBalance("TST2", signer), 150);
        assertEq(getBalance("TST2", address(1)), 50);
    }

    function testProposeEmptyBlock() external {
        bytes memory txs = abi.encode(new SPVMTransaction[](0));

        bytes32 blockHash = keccak256(
            abi.encodePacked(blocks[0].blockHash, txs)
        );

        Block memory b = Block({
            blockNumber: 1,
            parentHash: blocks[0].blockHash,
            blockHash: blockHash,
            transactions: new SPVMTransaction[](0),
            proposer: signer,
            proposer_signature: signHash(pk, blockHash)
        });
        vm.expectRevert(bytes("Too early to propose"));
        this.proposeBlock(b);
        vm.roll(block.number + 1);
        this.proposeBlock(b);

        assertEq(blocks[1].blockNumber, 1);
        assertEq(blocks[1].parentHash, blocks[0].blockHash);
        assertEq(blocks[1].blockHash, blockHash);
        assertEq(blocks[1].transactions.length, 0);
    }

    function testProposeBlock() external {
        // propose a block with one transaction
        SPVMTransaction memory Tx = createMintTransaction(
            "TST",
            signer,
            signer,
            100,
            0
        );

        SPVMTransaction[] memory txs = new SPVMTransaction[](1);
        txs[0] = Tx;

        bytes memory encoded_txs = abi.encode(txs);

        bytes32 blockHash = keccak256(
            abi.encodePacked(blocks[0].blockHash, encoded_txs)
        );

        Block memory b = Block({
            blockNumber: 1,
            parentHash: blocks[0].blockHash,
            blockHash: blockHash,
            transactions: txs,
            proposer: signer,
            proposer_signature: signHash(pk, blockHash)
        });
        vm.expectRevert(bytes("Too early to propose"));
        this.proposeBlock(b);
        vm.roll(block.number + 1);
        this.proposeBlock(b);
        
        assertEq(blocks[1].blockNumber, 1);
        assertEq(blocks[1].parentHash, blocks[0].blockHash);
        assertEq(blocks[1].blockHash, blockHash);
        assertEq(blocks[1].transactions.length, 1);
        assertEq(getBalance("TST", signer), 100);

        // second block
        SPVMTransaction memory Tx2 = createTransferTransaction(
            "TST",
            signer,
            address(1),
            50,
            1
        );
        
        SPVMTransaction[] memory txs2 = new SPVMTransaction[](1);
        txs2[0] = Tx2;

        bytes memory encoded_txs2 = abi.encode(txs2);

        bytes32 blockHash2 = keccak256(
            abi.encodePacked(blocks[1].blockHash, encoded_txs2)
        );

        Block memory b2 = Block({
            blockNumber: 2,
            parentHash: blocks[1].blockHash,
            blockHash: blockHash2,
            transactions: txs2,
            proposer: signer,
            proposer_signature: signHash(pk, blockHash2)
        });
        this.proposeBlock(b2);

        assertEq(blocks[2].blockNumber, 2);
        assertEq(blocks[2].parentHash, blocks[1].blockHash);
        assertEq(blocks[2].blockHash, blockHash2);
        assertEq(blocks[2].transactions.length, 1);
        assertEq(getBalance("TST", signer), 50);
        assertEq(getBalance("TST", address(1)), 50);
    }

    function testProposeBlockWithMultipleTxs() external {
        // mint
        SPVMTransaction memory Tx = createMintTransaction(
            "TST",
            signer,
            signer,
            100,
            0
        );

        // transfer
        SPVMTransaction memory Tx2 = createTransferTransaction(
            "TST",
            signer,
            address(1),
            50,
            1
        );

        SPVMTransaction[] memory txs = new SPVMTransaction[](2);
        txs[0] = Tx;
        txs[1] = Tx2;

        bytes memory encoded_txs = abi.encode(txs);

        bytes32 blockHash = keccak256(
            abi.encodePacked(blocks[0].blockHash, encoded_txs)
        );

        Block memory b = Block({
            blockNumber: 1,
            parentHash: blocks[0].blockHash,
            blockHash: blockHash,
            transactions: txs,
            proposer: signer,
            proposer_signature: signHash(pk, blockHash)
        });
        vm.expectRevert(bytes("Too early to propose"));
        this.proposeBlock(b);
        vm.roll(block.number + 1);
        this.proposeBlock(b);

        assertEq(blocks[1].blockNumber, 1);
        assertEq(blocks[1].parentHash, blocks[0].blockHash);
        assertEq(blocks[1].blockHash, blockHash);
        assertEq(blocks[1].transactions.length, 2);
        assertEq(getBalance("TST", signer), 50);
        assertEq(getBalance("TST", address(1)), 50);
    }

    // test block proposal permissioning
    function testProposeBlockWithElectionContract() external {
        TestElectionContract electionContract = new TestElectionContract();
        this.setElectionContract(electionContract);

        // mint
        SPVMTransaction memory Tx = createMintTransaction(
            "TST",
            signer,
            signer,
            100,
            0
        );

        // transfer
        SPVMTransaction memory Tx2 = createTransferTransaction(
            "TST",
            signer,
            address(1),
            50,
            1
        );

        SPVMTransaction[] memory txs = new SPVMTransaction[](2);
        txs[0] = Tx;
        txs[1] = Tx2;

        bytes memory encoded_txs = abi.encode(txs);

        bytes32 blockHash = keccak256(
            abi.encodePacked(blocks[0].blockHash, encoded_txs)
        );

        Block memory b = Block({
            blockNumber: 1,
            parentHash: blocks[0].blockHash,
            blockHash: blockHash,
            transactions: txs,
            proposer: signer,
            proposer_signature: signHash(pk, blockHash)
        });

        // set winner
        electionContract.setWinner(signer);
        vm.expectRevert(bytes("Too early to propose"));
        this.proposeBlock(b);
        vm.roll(block.number + 1);
        this.proposeBlock(b);

        assertEq(blocks[1].blockNumber, 1);
        assertEq(blocks[1].parentHash, blocks[0].blockHash);
        assertEq(blocks[1].blockHash, blockHash);
        assertEq(blocks[1].transactions.length, 2);
        assertEq(getBalance("TST", signer), 50);
        assertEq(getBalance("TST", address(1)), 50);
    }

    function testGetTransactionsInBlock() external {
        SPVMTransaction memory Tx = createMintTransaction(
            "TST",
            signer,
            signer,
            100,
            0
        );

        SPVMTransaction memory Tx2 = createMintTransaction(
            "TST2",
            signer,
            signer,
            200,
            1
        );

        // transfer
        SPVMTransaction memory Tx3 = createTransferTransaction(
            "TST2",
            signer,
            address(1),
            50,
            2
        );

        SPVMTransaction[] memory txs = new SPVMTransaction[](3);
        txs[0] = Tx;
        txs[1] = Tx2;
        txs[2] = Tx3;

        bytes memory encoded_txs = abi.encode(txs);

        bytes32 blockHash = keccak256(
            abi.encodePacked(blocks[0].blockHash, encoded_txs)
        );

        Block memory b = Block({
            blockNumber: 1,
            parentHash: blocks[0].blockHash,
            blockHash: blockHash,
            transactions: txs,
            proposer: signer,
            proposer_signature: signHash(pk, blockHash)
        });
        vm.expectRevert(bytes("Too early to propose"));
        this.proposeBlock(b);
        vm.roll(block.number + 1);
        this.proposeBlock(b);

        assertEq(getTransactionsInBlock(1).length, 3);
        assertEq(getTransactionsInBlock(1)[0].txContent.txType, 0);
        assertEq(getTransactionsInBlock(1)[1].txContent.txType, 0);
        assertEq(getTransactionsInBlock(1)[2].txContent.txType, 1);
    }

}

// election contract implementation for testing
contract TestElectionContract is ElectionInterface {
    address public winner;

    function setWinner(address _winner) public {
        winner = _winner;
    }

    function getWinner(uint /* block_number */) public view returns (address) {
        return winner;
    }
}
