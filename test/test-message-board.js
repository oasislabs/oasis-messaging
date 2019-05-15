require('google-closure-library');
goog.require('goog.crypt');

const MessageBoard = artifacts.require('MessageBoard');
const Web3c = require('web3c');
const web3c = new Web3c(MessageBoard.web3.currentProvider);

const BN = require('bn.js');

const truffleConfig = require('../truffle-config');

contract('MessageBoard', (accounts) => {
  // Run the tests against both a confidential and non confidential contract.
  let contracts = [
    {
      board: new web3c.eth.Contract(MessageBoard.abi, undefined, {
        from: accounts[0]
      }),
      label: 'eth'
    },
    {
      board: new web3c.oasis.Contract(MessageBoard.abi, undefined, {
        from: accounts[0]
      }),
      label: 'oasis'
    }
  ];

  console.log('Accounts: %s', accounts);

  contracts.forEach((contract) => {
    // The deployed contract instance to use in all the subsequent tests.
    let instance;

    it(`${contract.label}: deploys a message board contract`, async () => {
      instance = await contract.board.deploy({ data: MessageBoard.bytecode, arguments: [280] }).send({
        from: accounts[0]
      });
      assert.equal(instance.options.address.length, 42);

      // if we are not on devnet then fund a couple of accounts that we will use for tests
      if (!truffleConfig.OASIS) {
        var balance = await web3c.eth.getBalance(accounts[0]);
        console.log('accounts[0] has %d', balance);

        let transferAmount = balance / 42;
        await web3c.eth.sendTransaction({
          from: accounts[0],
          to: accounts[1],
          value: transferAmount
        });
        await web3c.eth.sendTransaction({
          from: accounts[0],
          to: accounts[2],
          value: transferAmount
        });
        await web3c.eth.sendTransaction({
          from: accounts[0],
          to: accounts[3],
          value: transferAmount
        });
        balance = await web3c.eth.getBalance(accounts[1]);
        console.log('accounts[1] has %d', balance);
        balance = await web3c.eth.getBalance(accounts[2]);
        console.log('accounts[2] has %d', balance);
        balance = await web3c.eth.getBalance(accounts[3]);
        console.log('accounts[3] has %d', balance);
      }
    });

    it(`${contract.label}: char limit should be 280`, async () => {
      const charLimit = await instance.methods.get_char_limit().call();

      assert.equal(charLimit, 280);
    });

    it(`${contract.label}: simple message broadcast`, async () => {
      const str = 'Hello World of Oasis!';
      let receipt = await instance.methods.post(str).send();
      assert.equal(receipt.status, true);
    });

    it(`${contract.label}: simple message get`, async () => {
      const result = await instance.methods.get_broadcast_messages(1).call();
      const resultingJSON = JSON.parse(new BN(result.substring(2), 16).toBuffer().toString());
      const message = resultingJSON['0'].message;
      assert.strictEqual(message, 'Hello World of Oasis!');
    });

    it(`${contract.label}: multiple message broadcast`, async () => {
      var str = 'Did you read my monograph on Tapanuli Fever?';
      let receipt = await instance.methods.post(str).send();
      assert.equal(receipt.status, true);

      str = 'I was eating this delicious carbonara...';
      receipt = await instance.methods.post(str).send();
      assert.equal(receipt.status, true);

      str = 'Your potheaded pedestrian pursuits be damned!';
      receipt = await instance.methods.post(str).send();
      assert.equal(receipt.status, true);

      str = 'I was going to say.. your monograph was easier to digest :)';
      receipt = await instance.methods.post(str).send();
      assert.equal(receipt.status, true);
    });

    it(`${contract.label}: get message by index`, async () => {
      var message = await instance.methods.get_broadcast_message_by_index(0).call();
      assert.strictEqual(message, '"I was going to say.. your monograph was easier to digest :)"');

      message = await instance.methods.get_broadcast_message_by_index(1).call();
      assert.strictEqual(message, '"Your potheaded pedestrian pursuits be damned!"');

      message = await instance.methods.get_broadcast_message_by_index(2).call();
      assert.strictEqual(message, '"I was eating this delicious carbonara..."');

      message = await instance.methods.get_broadcast_message_by_index(3).call();
      assert.strictEqual(message, '"Did you read my monograph on Tapanuli Fever?"');

      message = await instance.methods.get_broadcast_message_by_index(108).call();
      assert.strictEqual(message, '');
    });

    it(`${contract.label}: request more messages than there are`, async () => {
      const result = await instance.methods.get_broadcast_messages(1999).call();
      const resultingJSON = JSON.parse(new BN(result.substring(2), 16).toBuffer().toString());
      const message = resultingJSON['0'].message;
      // make sure we can get the one message that's there
      assert.strictEqual(message, 'Hello World of Oasis!');
      // make sure there are no more messages than the one
      assert.equal(Object.keys(resultingJSON).length, 5);
    });

    it(`${contract.label}: simple p2p send`, async () => {
      var str = "Don't forget to pick up potatoes!";
      var receipt = await instance.methods.send(accounts[2], str).send({
        from: accounts[1]
      });
      assert.equal(receipt.status, true);

      str = "...and don't forget to pick up pork butt either!";
      receipt = await instance.methods.send(accounts[1], str).send({
        from: accounts[2]
      });
      assert.equal(receipt.status, true);

      str = "...and don't talk to that woman in the Red dress!";
      receipt = await instance.methods.send(accounts[1], str).send({
        from: accounts[3]
      });
      assert.equal(receipt.status, true);
    });

    it(`${contract.label}: simple p2p message get`, async () => {
      // ask for eight.. you will get at least two
      const result = await instance.methods.get_messages(accounts[1], accounts[2], 8).call();
      const resultingJSON = JSON.parse(new BN(result.substring(2), 16).toBuffer().toString());
      const message1 = resultingJSON['0'].message;
      const message2 = resultingJSON['1'].message;
      assert.strictEqual(message1, "Don't forget to pick up potatoes!");
      assert.strictEqual(message2, "...and don't forget to pick up pork butt either!");
    });

    it(`${contract.label}: get friends test`, async () => {
      var result = await instance.methods.get_friends(accounts[1]).call();
      const friendsOf1 = JSON.parse(new BN(result.substring(2), 16).toBuffer().toString());
      console.log('Friends of %s: %s', accounts[1], JSON.stringify(friendsOf1));
      assert.equal(friendsOf1.friends.length, 2);
      assert.equal(friendsOf1.friends.includes(accounts[2].substring(2).toLowerCase()), true);
      assert.equal(friendsOf1.friends.includes(accounts[3].substring(2).toLowerCase()), true);

      result = await instance.methods.get_friends(accounts[2]).call();
      const friendsOf2 = JSON.parse(new BN(result.substring(2), 16).toBuffer().toString());
      console.log('Friends of %s: %s', accounts[2], JSON.stringify(friendsOf2));
      assert.equal(friendsOf2.friends.length, 1);
      assert.equal(friendsOf2.friends.includes(accounts[1].substring(2).toLowerCase()), true);
    });

    it(`${contract.label}: get friends as a space separated string test`, async () => {
      const result = await instance.methods.get_friends_as_string(accounts[1]).call();
      const friends = result.trim().split(' ');
      console.log(friends);
      assert.equal(friends.length, 2);
      assert.equal(friends.includes(accounts[2].substring(2).toLowerCase()), true);
      assert.equal(friends.includes(accounts[3].substring(2).toLowerCase()), true);
    });

    it(`${contract.label}: broadcast over 280 characters`, async () => {
      const str = "Frightened by a thunderstorm, the Gauls, with the exception of Getafix, who is at the annual conference for druids at the Forest of Carnutes, are huddled in the chief's hut, when they are visited by a soothsayer, called Prolix, who predicts that -when the storm is over, the weather will improve-. Soothsayers and augurers are also known to predict by the flight of the swallows!";
      let receipt = await instance.methods.post(str).send();
      let data = receipt['events']['0']['raw']['data'];
      let message = new BN(data.substring(2), 16).toBuffer().toString();
      console.log(message);
      assert.equal(receipt.status, true);
      assert.strictEqual(message, 'The message is longer than the 280 character limit. Please shorten and re-send.');
    });

    it(`${contract.label}: multiple p2p message send`, async () => {
      var str = 'Did you read my monograph on Tapanuli Fever?';
      var receipt = await instance.methods.send(accounts[0], str).send();
      assert.equal(receipt.status, true);

      str = 'I was eating this delicious carbonara...';
      receipt = await instance.methods.send(accounts[0], str).send();
      assert.equal(receipt.status, true);

      str = 'Your potheaded pedestrian pursuits be damned!';
      receipt = await instance.methods.send(accounts[0], str).send();
      assert.equal(receipt.status, true);

      str = 'I was going to say.. your monograph was easier to digest :)';
      receipt = await instance.methods.send(accounts[0], str).send();
      assert.equal(receipt.status, true);
    });

    it(`${contract.label}: p2p get message by index`, async () => {
      var message = await instance.methods.get_message_by_index(accounts[0], accounts[0], 0).call();
      assert.strictEqual(message, '"I was going to say.. your monograph was easier to digest :)"');

      message = await instance.methods.get_message_by_index(accounts[0], accounts[0], 1).call();
      assert.strictEqual(message, '"Your potheaded pedestrian pursuits be damned!"');

      message = await instance.methods.get_message_by_index(accounts[0], accounts[0], 2).call();
      assert.strictEqual(message, '"I was eating this delicious carbonara..."');

      message = await instance.methods.get_message_by_index(accounts[0], accounts[0], 3).call();
      assert.strictEqual(message, '"Did you read my monograph on Tapanuli Fever?"');

      message = await instance.methods.get_message_by_index(accounts[0], accounts[0], 108).call();
      assert.strictEqual(message, '');
    });
  });
});
