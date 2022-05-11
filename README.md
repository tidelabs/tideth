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

### deploy scripts

- create a config file, like `config/config.testnet.json`
- populate it with the necessary fields:
  - `eth_url` (infura or other)
  - `threshold`
  - `owners` array of H160 addresses
  - `chain_id`
  - `private_key` private key that owns rETH for deployment gas fees. You can send yourself ETH in metamask (ropsten) and extract your private key from there.
- deploy USDT/USDC: `NETWORK=testnet cargo run --bin 1_assets`
  - then add `usdt_address` and `usdc_address` to config
- deploy SAFE factory: `NETWORK=testnet cargo run --bin 2_factory`
  - then add `safe_factory_address` to config
- deploy SAFE: `NETWORK=testnet cargo run --bin 3_safe`
  - then add `safe_address` to config
- deploy Router: `NETWORK=testnet cargo run --bin 4_router`
  - then add `router_address` to config
- add assets to router acceptlist: `NETWORK=testnet cargo run --bin 5_add_assets`
- assign the router owner to be the SAFE: `NETWORK=testnet cargo run --bin 6_router_owner`
- assign the safe owners: `NETWORK=testnet cargo run --bin 7_safe_owners`
  - (make sure the owners array in config matches the ETH pubkeys from running qourum members)
- check everything is ready: `NETWORK=testnet cargo run --bin 8_ready_check`
