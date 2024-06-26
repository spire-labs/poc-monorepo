// SPDX-License-Identifier: MIT

pragma solidity ^0.8.20;

import "forge-std/Test.sol";

import "../src/ElectionContract.sol";
import "../src/GenericTicket.sol";

contract ElectionContractHarness is ElectionContract {
    constructor(address defaultRecipient) ElectionContract(defaultRecipient) {}
    function exposes_getWinnerTokenId(uint256 blockNumber) external view returns (uint256) {
        return getWinnerTokenId(blockNumber);
    }
}

contract ElectionContractTest is Test {
    function testCreation() public {
        ElectionContract election = new ElectionContract(address(this));
        assertEq(address(election.minter()), address(this));
        assertEq(address(election.default_recipient()), address(this));

        // check that the ticket was created successfully
        address ticket_address = election.ticket_address();
        GenericTicket ticket = GenericTicket(ticket_address);
        assertEq(ticket.totalSupply(), 0);
    }

    function testSetDefaultRecipient() public {
        ElectionContract election = new ElectionContract(address(this));
        assertEq(address(election.default_recipient()), address(this));
        address recipient = address(1);
        election.setDefaultRecipient(recipient);
        assertEq(address(election.default_recipient()), recipient);
    }

    function testMintTicket() public {
        ElectionContract election = new ElectionContract(address(this));
        address recipient = address(1);
        election.mintTicket(recipient);

        // check that the ticket was minted successfully
        address ticket_address = election.ticket_address();
        GenericTicket ticket = GenericTicket(ticket_address);
        assertEq(ticket.balanceOf(recipient), 1);
        assertEq(ticket.ownerOf(0), recipient);
    }

    function testMultipleMintTicket() public {
        ElectionContract election = new ElectionContract(address(this));
        address recipient1 = address(1);
        address recipient2 = address(2);
        election.mintTicket(recipient1);
        election.mintTicket(recipient2);

        // check that the tickets were minted successfully
        address ticket_address = election.ticket_address();
        GenericTicket ticket = GenericTicket(ticket_address);
        assertEq(ticket.balanceOf(recipient1), 1);
        assertEq(ticket.ownerOf(0), recipient1);
        assertEq(ticket.balanceOf(recipient2), 1);
        assertEq(ticket.ownerOf(1), recipient2);
    }

    function testGetOneWinnerTokenId() public {
        // getWinner will panic with ariethmetic underflow if the block number is too low
        vm.roll(1000);

        ElectionContractHarness election = new ElectionContractHarness(address(this));

        election.mintTicket(address(1));

        vm.roll(block.number + 10);

        // getwinner should not be called without refreshing the tickets
        uint winner = election.exposes_getWinnerTokenId(block.number - 5);

        assertEq(winner, 0, "winning token id should be 0");
    }

    function testGetOneWinner() public {
        // getWinner will panic with ariethmetic underflow if the block number is too low
        vm.roll(1000);

        ElectionContract election = new ElectionContract(address(this));

        election.mintTicket(address(1));

        vm.roll(block.number + 10);

        // getwinner should not be called without refreshing the tickets
        address winner = election.getWinner(block.number - 5);

        assertEq(winner, address(1), "winning address should be address(1)");
    }

    function testGetOneWinnerTokenIdWithRefresh() public {
        // getWinner will panic with ariethmetic underflow if the block number is too low
        vm.roll(1000);

        ElectionContractHarness election = new ElectionContractHarness(address(this));

        election.mintTicket(address(1));

        vm.roll(block.number + 5);

        uint preRefreshWinner = election.exposes_getWinnerTokenId(block.number - 3);
        assertEq(preRefreshWinner, 0, "pre-refresh getwinnertokenid should be the only ticket holder");
        election.refreshTickets();

        vm.roll(block.number + 5);

        uint preRefreshHistoricalWinner = election.exposes_getWinnerTokenId(block.number - 8);
        assertEq(preRefreshHistoricalWinner, preRefreshWinner, "pre-refresh historical getwinner should be the same as the pre-refresh getwinner");

        uint postRefreshWinner = election.exposes_getWinnerTokenId(block.number - 5);
        assertEq(postRefreshWinner, 0); // same token id as pre-refresh
        assertEq(election.getWinner(block.number - 5), address(this)); // default recipient
    }

    function testRefreshTicketsReverts() public {
        vm.roll(1000);

        ElectionContract election = new ElectionContract(address(this));
        election.mintTicket(address(1));

        vm.expectRevert("Cannot refresh tickets again before the next block");
        election.refreshTickets();
        
        vm.roll(block.number + 1);

        election.refreshTickets();

        vm.expectRevert("Cannot refresh tickets again before the next block");
        election.refreshTickets();  
    }

    mapping(uint256 => uint256) winner_frequency;

    /// forge-config: default.fuzz.runs = 100
    function testFuzz_WinnerTokenId(uint amount) public {
        vm.assume(amount > 1000);
        vm.assume(amount < 10000);
        vm.roll(amount);

        ElectionContractHarness election = new ElectionContractHarness(address(this));

        election.mintTicket(address(1));
        election.mintTicket(address(2));
        election.mintTicket(address(3));
        election.mintTicket(address(4));

        vm.roll(block.number + 64);

        for (uint i = 0; i < 64; i++) {
            uint256 winner = election.exposes_getWinnerTokenId(block.number - i);
            winner_frequency[winner]++;
        }
        assertEq(winner_frequency[4], 0);
    }

    function testFuzz_RandomnessOfWinnerTokenId(uint amount) public {
        vm.assume(amount > 1000);
        vm.assume(amount < 10000);
        vm.roll(amount);

        ElectionContractHarness election = new ElectionContractHarness(address(this));

        election.mintTicket(address(1));
        election.mintTicket(address(2));
        election.mintTicket(address(3));
        election.mintTicket(address(4));

        vm.roll(block.number + 64);
        for (uint i = 0; i < 64; i++) {
            uint256 winner = election.exposes_getWinnerTokenId(block.number - i);
            winner_frequency[winner]++;
        }

        // check that the winner_frequency is suitably random
        // lower bound and upper bound are calculatwd: http://www.cluster-text.com/confidence_interval.php
        // they assume a confidence interval of 99.99% with a large population and a sample size of 64, with relevant items as 12.
        // there may be better ways to determine if the winner_frequency is random, but this is a good approximation

        assertGt(winner_frequency[0], 5);
        assertLt(winner_frequency[0], 42);

        assertGt(winner_frequency[1], 5);
        assertLt(winner_frequency[1], 42);

        assertGt(winner_frequency[2], 5);
        assertLt(winner_frequency[2], 42);

        assertGt(winner_frequency[3], 5);
        assertLt(winner_frequency[3], 42);
    }

    function test_retrieveHistoricalWinner() public {
        vm.roll(1000);

        ElectionContractHarness election = new ElectionContractHarness(address(this));

        election.mintTicket(address(1));
        election.mintTicket(address(2));
        election.mintTicket(address(3));
        election.mintTicket(address(4));
    
        vm.roll(block.number + 1);
        address winner = election.getWinner(block.number);
        vm.roll(block.number + 1);
        assertEq(winner, election.getWinner(block.number-1));
        vm.roll(block.number + 1);
        assertEq(winner, election.getWinner(block.number-2));
    }
}
