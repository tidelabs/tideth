pragma solidity ^0.8.0;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

contract Circle is ERC20 {
    constructor() ERC20("Circle", "USDC") {
        _mint(msg.sender, 1000000 * (10**18));
    }

    function decimals() public view override returns (uint8) {
        return 6;
    }

    receive() external payable {
        _mint(msg.sender, msg.value);
    }
}
