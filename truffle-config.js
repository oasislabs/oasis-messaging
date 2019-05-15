const HDWalletProvider = require('truffle-hdwallet-provider');

// add your Oasis Devnet mnemonic here. Be sure to keep it secret!
const MNEMONIC = '';

// mnemonic for Contract Kit local blockchain
const DEVELOPMENT_MNEMONIC = 'range drive remove bleak mule satisfy mandate east lion minimum unfold ready';

/**
 * True iff we're using the network oasis
 */
const OASIS = process.argv.indexOf('oasis') > -1;

module.exports = {
  networks: {
    // Oasis Devnet
    oasis: {
      provider: function () {
        return new HDWalletProvider(MNEMONIC, 'https://web3.oasiscloud.io', 0, 10);
      },
      network_id: '42261',
      gasPrice: '0x3b9aca00',
      gasLimit: '0x8000000000000'
    },

    // Contract Kit local chain
    development: {
      provider: function () {
        return new HDWalletProvider(DEVELOPMENT_MNEMONIC, 'http://localhost:8545', 0, 10);
      },
      network_id: '*',
      gasPrice: '0x3b9aca00',
      gasLimit: '0x8000000000000'
    }
  },
  compilers: {
    external: {
      command: './node_modules/.bin/oasis-compile',
      targets: [{
        path: './.oasis-build/*.json'
      }]
    }
  },
  OASIS
};
