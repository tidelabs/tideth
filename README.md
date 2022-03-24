# tideth

This repo contains the smart contracts for wrapping ETH and ERC20 tokens onto Tidechain. It also contains Rust utilities for interacting with these contracts in a type-safe way.

### key files

- `contracts/Router.sol`
- `contracts/IRouter.sol`
- `contracts/SafeOwnable.sol`

### testing

`npx hardhat compile`

`npx hardhat node`

`cargo test`