// SPDX-License-Identifier: MIT

pragma solidity ^0.8.20;

import "lib/forge-std/src/Test.sol";

import "lib/openzeppelin-contracts/contracts/token/ERC721/utils/ERC721Holder.sol";

import "src/GenericTicket.sol";

contract GenericTicketTest is Test, ERC721Holder {
    function testCreation() public {
        GenericTicket ticket = new GenericTicket(address(this));
        assertEq(ticket.owner(), address(this));
        assertEq(ticket.totalSupply(), 0);
    }

    function testMint() public {
        GenericTicket ticket = new GenericTicket(address(this));
        ticket.safeMint(address(this));
        assertEq(ticket.ownerOf(0), address(this));
        assertEq(ticket.totalSupply(), 1);
    }

    function testHistoricalSupply() public {
        GenericTicket ticket = new GenericTicket(address(this));
        vm.roll(block.number + 1);
        ticket.safeMint(address(this));
        assertEq(ticket.historical_supply_at(block.number - 1), 0);
        assertEq(ticket.historical_supply_at(block.number), 1);

        vm.roll(block.number + 1);
        ticket.safeMint(address(this));
        assertEq(ticket.historical_supply_at(block.number - 2), 0);
        assertEq(ticket.historical_supply_at(block.number - 1), 1);
        assertEq(ticket.historical_supply_at(block.number), 2);

        vm.roll(block.number + 2);
        ticket.safeMint(address(this));
        assertEq(ticket.historical_supply_at(block.number - 4), 0);
        assertEq(ticket.historical_supply_at(block.number - 3), 1);
        assertEq(ticket.historical_supply_at(block.number - 2), 2);
        assertEq(ticket.historical_supply_at(block.number - 1), 2);
        assertEq(ticket.historical_supply_at(block.number), 3);
    }

    function testHistoricalSupplyForFutureBlocks() public {
        GenericTicket ticket = new GenericTicket(address(this));
        vm.expectRevert("Cannot query future blocks");
        ticket.historical_supply_at(block.number + 1);
    }

    function testHistoricalSupplyAfterLongGap() public {
        GenericTicket ticket = new GenericTicket(address(this));
        vm.roll(block.number + 100);
        ticket.safeMint(address(this));
        assertEq(ticket.historical_supply_at(block.number - 100), 0);
        assertEq(ticket.historical_supply_at(block.number - 99), 0);
        assertEq(ticket.historical_supply_at(block.number - 50), 0);
        assertEq(ticket.historical_supply_at(block.number), 1);
    }

    function testUseTicket() public {
        GenericTicket ticket = new GenericTicket(address(this));
        ticket.safeMint(address(this));
        ticket.useTicket(address(1), 0);
        assertEq(ticket.ownerOf(0), address(1));
        assertEq(ticket.totalSupply(), 1);
    }

    function testUseTicketPermissions() public {
        GenericTicket ticket = new GenericTicket(address(this));
        ticket.safeMint(address(this));
        vm.startPrank(address(2));
        vm.expectRevert();
        ticket.useTicket(address(1), 0);
        vm.stopPrank();
        ticket.useTicket(address(1), 0);
        assertEq(ticket.ownerOf(0), address(1));
        assertEq(ticket.totalSupply(), 1);
    }
}
