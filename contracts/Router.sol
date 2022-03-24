// SPDX-License-Identifier: MIT

pragma solidity ^0.8.4;

import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import "@openzeppelin/contracts/token/ERC20/IERC20.sol";
import "@openzeppelin/contracts/security/ReentrancyGuard.sol";
import "./SafeOwnable.sol";
import "./IRouter.sol";

/// @title An ETH and ERC20 router contract
/// @notice For routing assets into and out of the Tidefi DEX
contract Router is SafeOwnable, ReentrancyGuard, IRouter {
    using SafeERC20 for IERC20;

    /// @notice acceptlist state
    mapping(address => bool) public override isAccepted;

    /// @notice add an asset to the acceptlist
    /// @param _address the asset to add
    function acceptToken(address _address) public override onlyOwner {
        isAccepted[_address] = true;
        emit Accepted(_address);
    }

    /// @notice remove an asset from the acceptlist
    /// @param _address the asset to remove
    function removeToken(address _address) public override onlyOwner {
        isAccepted[_address] = false;
        emit Removed(_address);
    }

    constructor() SafeOwnable() {}

    /// @notice deposit into the DEX. This function is nonReentrant to mitigate ERC777 re-entrancy on callback exploits
    /// @param account the AccountId on Tidechain
    /// @param asset the asset to deposit
    /// @param amount the amount to deposit
    /// @return bool (success)
    function deposit(
        bytes32 account,
        address asset,
        uint256 amount
    ) external payable override nonReentrant returns (bool) {
        if (asset == address(0)) {
            require(msg.value != 0, "T01: ETH required");
            require(msg.value == amount, "T02: Invalid ETH value");

            (bool success, ) = owner.call{value: amount}("");
            require(success, "T03: Send to owner failed");

            emit Deposit(account, asset, amount);
        } else {
            require(isAccepted[asset], "T04: Asset not accepted");
            require(msg.value == 0, "T02: Invalid ETH value");

            uint256 balanceBefore = IERC20(asset).balanceOf(address(this));
            IERC20(asset).safeTransferFrom(msg.sender, address(this), amount);
            uint256 balanceAfter = IERC20(asset).balanceOf(address(this));
            uint256 actualAmount = balanceAfter - balanceBefore;

            emit Deposit(account, asset, actualAmount);
        }

        return true;
    }

    /// @notice deposit into the DEX. This function is onlyOwner, as the owner is the Tidechain multisig Quorum
    /// @param account the AccountId on Tidechain
    /// @param asset the asset to deposit
    /// @param amount the amount to deposit
    /// @return bool (success)
    function withdraw(
        address account,
        address asset,
        uint256 amount
    ) external payable override onlyOwner nonReentrant returns (bool) {
        if (asset == address(0)) {
            require(amount == msg.value, "T02: Invalid ETH value");
            (bool success, ) = account.call{value: amount}("");
            require(success, "T05: withdraw failed");
            emit Withdraw(account, asset, amount);
        } else {
            require(msg.value == 0, "T02: Invalid ETH value");
            IERC20(asset).safeTransfer(account, amount);
            emit Withdraw(account, asset, amount);
        }
        return true;
    }
}
