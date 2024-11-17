const solanaWeb3 = require('@solana/web3.js');
const fs = require('fs');
const path = require('path');
const mysql = require('mysql2/promise');

const conn = new solanaWeb3.Connection(process.env.QN_RPC_URL, 'confirmed');

const PUMPFUN_CURVE = "GN73pfqZSUY5zs1FkoJLuXmrKNrYYReH4XxDNZtN8JyV";

const WSOL_ADDRESS = "So11111111111111111111111111111111111111112"
const SOLANA_DECIMALS = 9;

// MySQL configuration
const dbConfig = {
    host: process.env.DB_HOST,
    user: process.env.DB_USER,
    password: process.env.DB_PASSWORD,
    database: "trading",
};

let connection;

// Initialize the database connection
async function initDbConnection() {
    connection = await mysql.createConnection(dbConfig);
    console.log('Database connection established.');
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function insertToDb(row) {
    const query = `INSERT INTO TRADES (token, address, sol_amount, token_amount, price, tx, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?)`;
    try {
        await connection.execute(query, [row.token, row.address, row.sol_amount, row.token_amount, row.price, row.tx, row.timestamp]);
    } catch {
        console.log('Error inserting row');
        console.log(row);
    }
}

async function processSignature(signature, address) {
    const tx = await conn.getParsedTransaction(signature.signature, { maxSupportedTransactionVersion: 1,});
    if (tx) {
        const instructionTypes = tx.transaction.message.instructions.filter(instruction => (instruction.parsed?.info.mint === address) && (instruction.parsed?.type === 'burn')).map(instruction => instruction.parsed );
        // check if tx was successful and was not burn
        if (tx.meta.err || instructionTypes.length > 0) {
            console.log(`burn error: ${signature.signature}`)
            return;
        }
        const tx_meta = tx.meta;
        const accounts = tx.transaction.message.accountKeys;
        const addresses = accounts.map(account => account.pubkey.toString());
        if (!addresses.includes(RAYDIUM_CURVE) && !addresses.includes(PUMPFUN_CURVE)) {
            console.log(`curve not found: ${signature.signature}`);
            return;
        }
        // get index of raydium or pumpfun curve
        const curveIndex = addresses.indexOf(RAYDIUM_CURVE) !== -1 ? addresses.indexOf(RAYDIUM_CURVE) : addresses.indexOf(PUMPFUN_CURVE);
        console.log(curveIndex);

        console.log(accounts[curveIndex]);
        const trader = accounts[0];
        const timestamp = await conn.getBlockTime(tx.slot);

        // Retrieve pre and post balances (SOL)
        console.log(tx_meta.preBalances);
        console.log(tx_meta.postBalances);
        const solPreBalance = tx_meta.preBalances[curveIndex] / Math.pow(10, SOLANA_DECIMALS);
        const solPostBalance = tx_meta.postBalances[curveIndex] / Math.pow(10, SOLANA_DECIMALS);

        let tokenPreBalance, tokenPostBalance;

        if (!tx_meta.preTokenBalances) {
            tokenPreBalance = 0;
        } else {
            try {
                const preTokenBalances = tx_meta.preTokenBalances.filter(balance => balance.mint === address);
                const traderTokenPreBalance = preTokenBalances.filter(balance => balance.owner === trader.pubkey.toString());
                if (traderTokenPreBalance.length === 0) {
                    tokenPreBalance = 0;
                } else {
                    tokenPreBalance = traderTokenPreBalance.pop().uiTokenAmount.uiAmount;
                }

            } catch {
                console.log('preTokenBalances');
                console.log(tx_meta.preTokenBalances);
            }
        }

        if (!tx_meta.postTokenBalances) {
            tokenPostBalance = 0;
        } else {
            try {
                const postTokenBalances = tx_meta.postTokenBalances.filter(balance => balance.mint === address || balance.mint === WSOL_ADDRESS);
                const traderTokenPostBalance = postTokenBalances.filter(balance => balance.owner === trader.pubkey.toString());
                if (traderTokenPostBalance.length === 0) {
                    tokenPostBalance = 0;
                } else {
                    tokenPostBalance = traderTokenPostBalance.pop().uiTokenAmount.uiAmount;
                }
            } catch {
                console.log('postTokenBalances');
                console.log(tx_meta.postTokenBalances);
            }
        }
        
        const tokenChange = tokenPostBalance - tokenPreBalance;
        if (tokenChange === 0) {
            return;
        }
        const solChange = tokenChange < 0 ? -(solPostBalance - solPreBalance) : solPostBalance - solPreBalance;
        const price = Math.abs(solChange / tokenChange);

        const row = {
            token: address.toString(),
            address: trader.pubkey.toString(),
            sol_amount: solChange,
            token_amount: tokenChange,
            price: price,
            tx: tx.transaction.signatures[0],
            timestamp: timestamp
        }

        console.log(row);

        // await insertToDb(row);

        // sleep
        await sleep(100);
    }
}

async function processToken(tokenAddress) {
    const mintPublicKey = new solanaWeb3.PublicKey(tokenAddress);

    let signatures;
    let cnt = 0;

    do {
        signatures = await conn.getSignaturesForAddress(mintPublicKey, { limit: 1000, before: signatures?.[signatures.length - 1]?.signature }, 'confirmed');
        cnt += signatures.length;
        console.log(`Processing ${cnt} signatures`);
        for (const signature of signatures) {
            try {
                await processSignature(signature, tokenAddress);
            } catch (error) {
                console.error(`Error processing signature ${signature.signature}:`, error);
            }
        }
        
    } while (signatures.length === 1000);


}

async function fetchPoolInfo(tokenMint) {
    const mint1 = tokenMint;
    const poolType = "standard";
    const poolSortField = "default";
    const sortType = "desc";
    const pageSize = 100;
    const page = 1;

    const url = `https://api-v3.raydium.io/pools/info/mint?mint1=${mint1}&poolType=all&poolSortField=${poolSortField}&sortType=${sortType}&pageSize=${pageSize}&page=${page}`;

    try {
        const response = await fetch(url);
        
        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        const data = await response.json();
        return data.data.data;
        
    } catch (error) {
        console.error("Error fetching pool info:", error);
    }
}

async function main() {
    // try {
        await initDbConnection();

        const token = "AtakVE4hj5KgbS58YzmCYrUwRqMNCnwaamUckk2Zpump";


        // await processToken(token);

        // console.log('Finished');

        const addresses = await fetchPoolInfo(token);

        console.log(addresses);

        // const sig = {signature: 'LuyRQ6QBfChWUTNBoHDzaZYs7ogwxBEqteictdA9akhgWAubsCxf8KrPHaye1c2FZHuFrKPWr4ipVAcSafMdd7x'};
        // await processSignature(sig, token);


    // } catch{
    //     console.log()
    //     console.log('Error processing token');
    // }
}

main();