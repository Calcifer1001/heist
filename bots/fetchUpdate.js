const nearAPI = require("near-api-js");
const { keyStores, connect, Contract } = nearAPI;




const contractId = "dev-1676002720329-38070052854506"
// const url = "https://api.stats.ref.finance/api/top-tokens"
const url = "https://indexer.ref.finance/list-token-price"
const tokens = [
    "meta-pool.near",
    "meta-token.near",
    "wrap.near"
]
let contract

async function connectToNear() {
    const homedir = require("os").homedir();
    const CREDENTIALS_DIR = ".near-credentials";
    const credentialsPath = require("path").join(homedir, CREDENTIALS_DIR);
    const myKeyStore = new keyStores.UnencryptedFileSystemKeyStore(credentialsPath);

    const connectionConfig = {
        networkId: "testnet",
        keyStore: myKeyStore, // first create a key store 
        nodeUrl: "https://rpc.testnet.near.org",
        walletUrl: "https://wallet.testnet.near.org",
        helperUrl: "https://helper.testnet.near.org",
        explorerUrl: "https://explorer.testnet.near.org",
      };
      const nearConnection = await connect(connectionConfig);

      const account = await nearConnection.account("silkking.testnet");

      contract = new Contract(
        account, // the account object that is connecting
        contractId,
        {
          // name of contract you're connecting to
          viewMethods: [""], // view methods do not change state but usually return a value
          changeMethods: ["set_current_price_for_token"], // change methods modify state
        }
      );
}

async function run() {
    await connectToNear()

    const pricesResponse = await fetch(url)
    const prices = await pricesResponse.json()

    const relevantPrices = tokens.map(t => {
        return {token_id: t, price: Math.floor(parseFloat(prices[t].price) * 1000000000000)}
    })
    console.log(relevantPrices)
    
    const promises = relevantPrices.map(d => {
        contract.set_current_price_for_token({
            token: d.token_id, 
            price: d.price
        })
    })

    await Promise.all(promises)
}

run()


