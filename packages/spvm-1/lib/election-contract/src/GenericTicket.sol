// SPDX-License-Identifier: MIT
// Compatible with OpenZeppelin Contracts ^5.0.0
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721Enumerable.sol";
import "@openzeppelin/contracts/access/Ownable.sol";

contract GenericTicket is ERC721, ERC721Enumerable, Ownable {
    uint256 private _nextTokenId;

    mapping(uint => uint) public historical_supply;
    uint last_block_updated;

    constructor(
        address initialOwner
    ) ERC721("Ticket", "T") Ownable(initialOwner) {
        _nextTokenId = 0;
    }

    function safeMint(address to) public onlyOwner {
        uint256 tokenId = _nextTokenId++;
        _safeMint(to, tokenId);
    }

    function useTicket(address to, uint256 tokenId) public onlyOwner {
        // for testing purposes
        // _transfer(ownerOf(tokenId), to, tokenId);
        _update(to, tokenId, address(0));
    }

    function updateSupply() internal {
        if (block.number - last_block_updated > 1) {
            // update all blocks since last_block_updated
            for (uint i = last_block_updated + 1; i < block.number; i++) {
                historical_supply[i] = historical_supply[last_block_updated];
            }
        }

        // update current block
        historical_supply[block.number] = totalSupply();
        last_block_updated = block.number;
    }

    function historical_supply_at(
        uint block_number
    ) public view returns (uint) {
        if (block_number <= last_block_updated) {
            return historical_supply[block_number];
        } else {
            return totalSupply();
        }
    }

    function _update(
        address to,
        uint256 tokenId,
        address auth
    ) internal override(ERC721, ERC721Enumerable) returns (address) {
        address out = super._update(to, tokenId, auth);

        updateSupply();

        return out;
    }

    function _increaseBalance(
        address account,
        uint128 value
    ) internal override(ERC721, ERC721Enumerable) {
        super._increaseBalance(account, value);

        updateSupply();
    }

    function transferFrom(
        address from,
        address to,
        uint256 tokenId
    ) public override(ERC721, IERC721) {}

    function supportsInterface(
        bytes4 interfaceId
    ) public view override(ERC721, ERC721Enumerable) returns (bool) {
        return super.supportsInterface(interfaceId);
    }
}
