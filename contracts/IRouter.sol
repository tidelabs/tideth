// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

/// @dev Interface of the Router contract
interface IRouter {
    /// @dev Emitted when a deposit succeeds
    event Deposit(
        bytes32 indexed account,
        address indexed asset,
        uint256 amount
    );
    /// @dev Emitted when a withdrawal succeeds
    event Withdraw(
        address indexed account,
        address indexed asset,
        uint256 amount
    );
    /// @dev Emitted when an asset is added to the acceptlist
    event Accepted(address indexed asset);
    /// @dev Emitted when an asset is removed to the acceptlist
    event Removed(address indexed asset);

    /// @dev check if an asset is accepted according to the acceptlist
    /// @param asset the asset to check
    function isAccepted(address asset) external returns (bool);

    /// @dev add an asset to the acceptlist
    /// @param asset the asset to add
    function acceptToken(address asset) external;

    /// @dev remove an asset from the acceptlist
    /// @param asset the asset to remove
    function removeToken(address asset) external;

    /// @dev deposit into the DEX. This function is nonReentrant to mitigate ERC777 re-entrancy on callback exploits
    /// @param account the AccountId on Tidechain
    /// @param asset the asset to deposit
    /// @param amount the amount to deposit
    /// @return bool (success)
    function deposit(
        bytes32 account,
        address asset,
        uint256 amount
    ) external payable returns (bool);

    /// @dev deposit into the DEX. This function is onlyOwner, as the owner is the Tidechain multisig Quorum
    /// @param account the AccountId on Tidechain
    /// @param asset the asset to deposit
    /// @param amount the amount to deposit
    /// @return bool (success)
    function withdraw(
        address account,
        address asset,
        uint256 amount
    ) external payable returns (bool);
}
