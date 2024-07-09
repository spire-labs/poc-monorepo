// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

import "./ElectionInterface.sol";
import "./GenericTicket.sol";

contract ElectionContract is ElectionInterface {
    address public minter;
    address public default_recipient;
    address public ticket_address;

    // block number to winner mapping, used for historical blocks
    mapping(uint256 => address) private winners;

    uint256 public next_refresh_block;

    constructor(address _minter) {
        minter = _minter;

        // deploy the ERC721 contract for proposal tickets
        ticket_address = address(new GenericTicket(address(this)));

        next_refresh_block = block.number;

        // set the default recipient
        default_recipient = msg.sender;
    }

    function _max(uint _a, uint _b) internal pure returns (uint) {
        return (_a < _b) ? _b : _a;
    }

    /// @notice Sets the default recipient for tickets.
    function setDefaultRecipient(address _default_recipient) external {
        require(
            msg.sender == minter,
            "Only the minter can set the default recipient"
        );
        default_recipient = _default_recipient;
    }

    /// @notice Returns the address of the winner of the election for a certain L1 block number.
    function getWinner(
        uint block_number
    ) public view override returns (address) {
        GenericTicket ticket = GenericTicket(ticket_address);

        if (block_number >= next_refresh_block) {
            return ticket.ownerOf(getWinnerTokenId(block_number));
        } else {
            return winners[block_number];
        }
    }

    /// @dev Returns the tokenId of the winner of the election for a certain L1 block number. Used internally.
    function getWinnerTokenId(
        uint block_number
    ) internal view returns (uint256) {
        require(
            block_number > (block.number > 292 ? block.number - 292 : 0),
            "Cannot query blocks older than 292 blocks (election)"
        );

        GenericTicket ticket = GenericTicket(ticket_address);

        // get historical supply of tickets at the given block number
        block_number -= block_number % 2 == 0 ? 1 : 0;
        uint historical_supply = _max(
            ticket.historical_supply_at(block_number),
            1
        );

        // get the historical block hash at the seed block for the given block number
        uint historical_block_hash = uint(
            blockhash(block_number > 64 ? block_number - 64 : 0)
        );

        uint winner_token_id = historical_block_hash % historical_supply;

        return winner_token_id;
    }

    /// @notice Mints a ticket for the given address.
    function mintTicket(address to) external {
        require(msg.sender == minter, "Only the minter can mint new tickets");

        GenericTicket(ticket_address).safeMint(to);
    }

    /// @notice Refreshes tickets and redistributes previous tickets to the default recipient.
    /// This is needed so that a proposer that proposed a previous block cannot reuse a ticket.
    /// Does not apply for the current block, but only for historical blocks.
    function refreshTickets() external {
        require(
            block.number > next_refresh_block,
            "Cannot refresh tickets again before the next block"
        );

        // check all tickets since last_block_updated
        for (uint i = next_refresh_block; i < block.number; i++) {
            uint256 tokenId = getWinnerTokenId(i);
            winners[i] = GenericTicket(ticket_address).ownerOf(tokenId);
            GenericTicket(ticket_address).useTicket(default_recipient, tokenId);
        }

        next_refresh_block = block.number; // update the next refresh block
    }
}
