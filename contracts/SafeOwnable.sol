/// @title SafeOwnable
/// @notice Based on OpenZepplin's Ownable library. 1: Ownership can never be renounced. 2: Ownership transfers require confirmation.
abstract contract SafeOwnable {
    address public owner;
    address public pendingOwner;

    /// @notice Emitted when an ownership transfer is proposed
    event OwnershipTransferProposed(
        address indexed oldOwner,
        address indexed newOwner
    );
    /// @notice Emitted when ownership has been transferred
    event OwnershipTransferred(
        address indexed oldOwner,
        address indexed newOwner
    );

    /// @notice Sets the initial owner to be the deployer of the contract
    constructor() {
        _claimOwnership();
    }

    /// @notice the onlyOnly modifier can be used by other functions, to restrict their use to the owner only
    modifier onlyOwner() {
        require(owner == msg.sender, "S01: caller is not the owner");
        _;
    }

    /// @notice Allow another account to claim ownership
    /// @param newOwner The new owner who will be allowed to claim ownership
    function transferOwnership(address newOwner) public virtual onlyOwner {
        pendingOwner = newOwner;
        emit OwnershipTransferProposed(owner, pendingOwner);
    }

    /// @notice Claim ownership over the contract
    function claimOwnership() public virtual {
        require(
            msg.sender == pendingOwner,
            "S02: caller is not the pending owner"
        );
        _claimOwnership();
    }

    /// @notice internal utility function for claiming ownership
    function _claimOwnership() internal virtual {
        address oldOwner = owner;
        owner = msg.sender;
        pendingOwner = address(0);
        emit OwnershipTransferred(oldOwner, owner);
    }
}
