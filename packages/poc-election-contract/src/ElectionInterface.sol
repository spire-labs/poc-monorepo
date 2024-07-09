// SPDX-License-Identifier: MIT

pragma solidity ^0.8.0;

/// Interface for an Election mechanism to determine the winner of a random slot leader election.
/// Used by the SPVM contract to permission the block proposing.
interface ElectionInterface {
    /// @notice Returns the address of the winner of the election for a certain L2 block number.
    function getWinner(uint block_number) external view returns (address);
}
