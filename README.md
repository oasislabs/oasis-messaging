# Message board contract

This contract implements a messaging API that can be used to build dApps. It supports both broadcast and peer-to-peer messaging. The messages are restricted to a configurable number of characters. The sender and recipient are identified by their wallet addresses. The contract is expected to be deployed as a confidential contract and is written in Rust.

To interact with this contract locally follow the steps below.

## Initial Setup

* Run `npm install`

* Run `npm install -g truffle-oasis` to setup Oasis truffle

## Building, Deploying, and Testing the Smart Contract
To create a new message board, run:

* `truffle compile`

You can deploy the contract with truffle. to do so:

* Edit `truffle-config.js` to add the mnemonic for your private key. This will be the address that deploys the smart contract;

* `truffle migrate --reset --network oasis`

* `truffle test test/test-message-board.js`

## Notes and Troubleshooting
* The smart contract code is available as `MessageBoard` on our dashboard for dashboard based deployments

## Evolving features
* Automatic message expiry
* `Catch-up` mode of message retrieval

