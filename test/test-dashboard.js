const MessageBoard = artifacts.require('MessageBoard');
const Web3c = require('web3c');
const web3c = new Web3c(MessageBoard.web3.currentProvider);

contract('MessageBoard', (accounts) => {
  // Run the tests against both a confidential and non confidential contract.
  let contracts = [
    {
      // The contract address below comes from our dashboard deployment of the MessageBoard
      // contract. This test is to exercise a contract deployed using our dashboard.
      // Replace the address below with the address of your MessageBoard deployment when
      // you want to test your deployment
      board: new web3c.oasis.Contract(MessageBoard.abi, '0xC90C96bbb46257250a450ad5CD2673A0cD92A2A0'),
      label: 'oasis'
    }
  ];

  contracts.forEach((contract) => {
    // The deployed contract instance to use in all the subsequent tests.
    let instance;

    it(`${contract.label}: deploys a message board contract`, async () => {
      instance = contract.board;
      console.log('Contract address: %s', instance.options.address);
      assert.equal(instance.options.address.length, 42);
    });

    it(`${contract.label}: multiple p2p message send`, async () => {
      var str = 'Did you read my monograph on Tapanuli Fever?';
      var receipt = await instance.methods.send(accounts[1], str).send({ from: accounts[0] });
      assert.equal(receipt.status, true);

      str = 'I was eating this delicious carbonara...';
      receipt = await instance.methods.send(accounts[0], str).send({ from: accounts[1] });
      assert.equal(receipt.status, true);

      str = 'Your potheaded pedestrian pursuits be damned!';
      receipt = await instance.methods.send(accounts[1], str).send({ from: accounts[0] });
      assert.equal(receipt.status, true);

      str = 'I was going to say.. your monograph was easier to digest :)';
      receipt = await instance.methods.send(accounts[0], str).send({ from: accounts[1] });
      assert.equal(receipt.status, true);
    });

    it(`${contract.label}: p2p get message by index`, async () => {
      var message = await instance.methods.get_message_by_index(accounts[0], accounts[1], 0).call();
      assert.strictEqual(message, '"I was going to say.. your monograph was easier to digest :)"');

      message = await instance.methods.get_message_by_index(accounts[0], accounts[1], 1).call();
      assert.strictEqual(message, '"Your potheaded pedestrian pursuits be damned!"');

      message = await instance.methods.get_message_by_index(accounts[1], accounts[0], 2).call();
      assert.strictEqual(message, '"I was eating this delicious carbonara..."');

      message = await instance.methods.get_message_by_index(accounts[0], accounts[1], 3).call();
      assert.strictEqual(message, '"Did you read my monograph on Tapanuli Fever?"');
    });
  });
});
