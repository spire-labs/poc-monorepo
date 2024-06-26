// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "lib/forge-std/src/Test.sol";
import "./ElectionContract.sol";
import "src/Slashing.sol";

contract SlashingTest is Test {
    Slashing slashing;

    address enforcer;
    uint256 enforcerPk;

    function setUp() public {
        (enforcer, enforcerPk) = makeAddrAndKey("enforcer");

        slashing = new Slashing(enforcer);
    }

    function testConstruction() public {
        assertNotEq(address(slashing).code.length, 0);
    }

    function testValidityConditions() public {
        (address signer, uint256 signerPk) = makeAddrAndKey("signer");

        // construct PreconfCommitment
        PreconfTransactionContent memory preconfTx = PreconfTransactionContent(
            signer, // from
            uint8(0), // type
            abi.encode(0), // param
            uint32(0) // nonce
        );
        // Sign tx
        bytes32 txHash = keccak256(abi.encode(preconfTx));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(signerPk, txHash);
        bytes memory txSig = abi.encodePacked(r, s, v);
        Transaction memory signerTx = Transaction(
            txHash, // tx hash
            preconfTx, // tx content
            txSig // signature
        );

        // Submit validity conditions
        Transaction[] memory txs = new Transaction[](1);
        txs[0] = signerTx;
        vm.prank(enforcer);
        slashing.submitValidityConditions(txs);

        // Get validity conditions
        Transaction[] memory validityTxs = slashing.getValidityConditions(
            block.number
        );
        assertEq(
            keccak256(abi.encode(validityTxs)),
            keccak256(abi.encode(txs))
        );
    }

    function testSlash() public {
        // construct PreconfCommitment
        PreconfTransactionContent memory preconfTx = PreconfTransactionContent(
            address(0), // from
            uint8(0), // type
            abi.encode(0), // param
            uint32(0) // nonce
        );
        Transaction memory tx = Transaction(
            keccak256(abi.encode(preconfTx)), // tx hash
            preconfTx, // tx content
            abi.encode(0) // signature
        );
        PreconfirmationRequest memory preconfRequest = PreconfirmationRequest(
            tx, // tx content
            tx, // tip tx
            address(slashing) // preconf contract
        );
        // Sign request
        bytes32 requestHash = keccak256(abi.encode(preconfRequest));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(enforcerPk, requestHash);
        bytes memory requestSig = abi.encodePacked(r, s, v);

        PreconfirmationCommitment memory preconfCommit = PreconfirmationCommitment(
            preconfRequest,
            requestSig, // commitment
            enforcer, // commit signer
            block.number // block number
        );

        assertTrue(slashing.slash(preconfCommit));
    }

    function testSlashFail() public {
        (address signer, uint256 signerPk) = makeAddrAndKey("signer");
        vm.deal(signer, 1 ether);

        // construct PreconfCommitment
        PreconfTransactionContent memory preconfTx = PreconfTransactionContent(
            signer, // from
            uint8(0), // type
            abi.encode(0), // param
            uint32(0) // nonce
        );
        // Sign tx
        bytes32 txHash = keccak256(abi.encode(preconfTx));
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(signerPk, txHash);
        bytes memory txSig = abi.encodePacked(r, s, v);
        Transaction memory signerTx = Transaction(
            txHash, // tx hash
            preconfTx, // tx content
            txSig // signature
        );
        PreconfirmationRequest memory preconfRequest = PreconfirmationRequest(
            signerTx, // tx content
            signerTx, // tip tx
            address(slashing) // preconf contract
        );
        // Sign request
        bytes32 requestHash = keccak256(abi.encode(preconfRequest));
        (v, r, s) = vm.sign(enforcerPk, requestHash);
        bytes memory requestSig = abi.encodePacked(r, s, v);

        PreconfirmationCommitment memory preconfCommit = PreconfirmationCommitment(
            preconfRequest,
            requestSig, // commitment
            enforcer, // commit signer
            block.number // block number
        );

        // Submit validity conditions
        Transaction[] memory txs = new Transaction[](1);
        txs[0] = signerTx;
        vm.prank(enforcer);
        slashing.submitValidityConditions(txs);

        // Submit tx
        vm.prank(signer);
        address(0).call("");

        vm.expectRevert("The preconfirmation was honored");
        slashing.slash(preconfCommit);
    }
}
