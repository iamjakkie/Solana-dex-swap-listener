import { PublicKey, Connection, ParsedTransactionWithMeta, ParsedInstruction, TokenBalance, ConfirmedSignatureInfo } from "@solana/web3.js";
import { TokenAccount,
    SPL_ACCOUNT_LAYOUT,
    LIQUIDITY_STATE_LAYOUT_V4 } from "@raydium-io/raydium-sdk";
import 'dotenv/config';
import "@solana/spl-token";
import mysql from "mysql2/promise";

const dbConfig = {
    host: process.env.DB_HOST,
    user: process.env.DB_USER,
    password: process.env.DB_PASSWORD,
    database: "trading",
}

let connection :mysql.Connection;

async function initDbConnection() {
    connection = await mysql.createConnection(dbConfig);
}


// Constants for the Solana environment
const SOLANA_DECIMALS = 9;
const RAYDIUM_CURVE = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"; // Replace with actual Raydium curve address
const PUMPFUN_CURVE = "PumpFunCurveAddress"; // Replace with actual PumpFun curve address
const WSOL_ADDRESS = "WSOLAddress"; // Replace with actual Wrapped SOL address
const conn: Connection = new Connection(process.env.QN_RPC_URL!, "confirmed"); // Initialize Solana connection

// Define types for row data
interface Row {
    token: string;
    address: string;
    sol_amount: number;
    token_amount: number;
    price: number;
    tx: string;
    timestamp: number;
}

type LP = {
    base_vault: PublicKey;
    base_mint: PublicKey;
    quote_vault: PublicKey;
    quote_mint: PublicKey;
};

interface PoolInfo {
    type: string;
    programId: string;
    id: string;
    mintA: Object,
    mintB: Object,
    price: number;
    mintAmountA: number;
    mintAmountB: number;
    feeRate: number;
    openTime: string,
    tvl: number;
    day: Object,
    week: Object,
    month: Object,
    poolType: string[],
    rewardDefaultInfos: Object[],
    farmUpcomingCount: number,
    farmOngoingCount: number,
    farmFinishedCount: number,
    marketId: string,
    lpMint: Object,
    lpPrice: number,
    lpAmount: number,
    burnPercent: number,
}

interface TradeRow {
    tx: string;
}

// Utility sleep function
function sleep(ms: number) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

function isParsedInstruction(instruction: any): instruction is ParsedInstruction {
    return (instruction as ParsedInstruction).parsed !== undefined;
}

async function insertToDb(row: Row) {
    try {
        await connection.query(
            `INSERT INTO TRADES (token, address, sol_amount, token_amount, price, tx, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?)`,
            [row.token, row.address, row.sol_amount, row.token_amount, row.price, row.tx, row.timestamp]
        );
    } catch (error) {
        console.error("Error inserting row:", error);
        console.error(row);
    }
}

// Main function to process a signature
async function processSignature(signature: { signature: string }, address: string, lp: LP): Promise<void> {
    const tx: ParsedTransactionWithMeta | null = await conn.getParsedTransaction(signature.signature, {
        maxSupportedTransactionVersion: 1,
    });

    if (tx) {
        if (tx.meta?.err) {
            console.log(`Error in transaction: ${signature.signature}`);
            return
        }
        // Extract instructions related to burn type for this token address
        let instructionTypes: ParsedInstruction[];
        try {
            instructionTypes = tx.transaction.message.instructions
                .filter((instruction) => isParsedInstruction(instruction) && (instruction.parsed?.info.mint === address) && (instruction.parsed?.type === "burn"))
                .map((instruction) => (instruction as ParsedInstruction).parsed);
        } catch (error) {
            console.error(`Error extracting instructions for ${signature.signature}:`, error);
            return;
        }
        

        // Check if the transaction was unsuccessful or involved a burn operation
        if (instructionTypes.length > 0) {
            console.log(`burn error: ${signature.signature}`);
            return;
        }

        const tx_meta = tx.meta;
        const accounts = tx.transaction.message.accountKeys;

        // get index of base and quote vaults
        const base_index = accounts.findIndex((account) => account.pubkey.toString() === lp.base_vault.toString());
        const quote_index = accounts.findIndex((account) => account.pubkey.toString() === lp.quote_vault.toString());

        // Determine index for the Raydium or PumpFun curve
        const trader = accounts[0];
        const timestamp = await conn.getBlockTime(tx.slot) ?? 0;

        let lpPreTokenBalance, lpPostTokenBalance;

        lpPreTokenBalance = tx_meta?.preTokenBalances?.filter((balance) => balance.owner === RAYDIUM_CURVE && balance.mint === address).pop()?.uiTokenAmount.uiAmount || 0;
        lpPostTokenBalance = tx_meta?.postTokenBalances?.filter((balance) => balance.owner === RAYDIUM_CURVE && balance.mint === address).pop()?.uiTokenAmount.uiAmount || 0;

        if ((lpPreTokenBalance === 0 && lpPostTokenBalance === 0) || lpPreTokenBalance === lpPostTokenBalance) {
            return;
        }
        let lpPreSolBalance, lpPostSolBalance

        if (lp.base_mint.toString() === WSOL_ADDRESS) {
            lpPreSolBalance = tx_meta?.preBalances?.[quote_index] || 0;
            lpPostSolBalance = tx_meta?.postBalances?.[quote_index] || 0;
        } else {
            lpPreSolBalance = tx_meta?.preBalances?.[base_index] || 0;
            lpPostSolBalance = tx_meta?.postBalances?.[base_index] || 0;
        }

        lpPreSolBalance = lpPreSolBalance / Math.pow(10, SOLANA_DECIMALS);
        lpPostSolBalance = lpPostSolBalance / Math.pow(10, SOLANA_DECIMALS);

        
        const solChange = -(lpPostSolBalance - lpPreSolBalance);
        const tokenChange = -(lpPostTokenBalance - lpPreTokenBalance);

        const price = Math.abs(solChange / tokenChange);

        const row: Row = {
            token: address.toString(),
            address: trader.pubkey.toString(),
            sol_amount: solChange,
            token_amount: tokenChange,
            price: price,
            tx: tx.transaction.signatures[0],
            timestamp: timestamp,
        };

        await insertToDb(row);

        // Delay between operations
        // await sleep(100);
    }
}

async function fetchPoolInfo(mint1: string): Promise<PoolInfo[]> {
    const url = `https://api-v3.raydium.io/pools/info/mint?mint1=${mint1}&poolType=all&poolSortField=default&sortType=desc&pageSize=10&page=1`;
    
    try {
        const response = await fetch(url, {
            method: 'GET',
            headers: {
                'Accept': 'application/json'
            }
        });

        if (!response.ok) {
            throw new Error(`HTTP error! Status: ${response.status}`);
        }

        const data = await response.json();
        return data.data.data as PoolInfo[]; // Adjust this path based on the exact JSON structure
    } catch (error) {
        console.error("Error fetching pool info:", error);
        return [];
    }
}

async function getOldestSignatureProcessed(token_address: string): Promise<string | undefined> {
    try {
        const [rows] = await connection.query(
            `SELECT tx FROM TRADES WHERE token = ? ORDER BY timestamp LIMIT 1`,
            [token_address]
        );

        const timestamps = rows as TradeRow[];

        return timestamps[0].tx; // Access the timestamp of the first row
    } catch (error) {
        console.error("Error fetching oldest signature:", error);
        // return undefined;
    }
}

async function processSignaturesInParallel(signatures: ConfirmedSignatureInfo[], address: string, lp: LP): Promise<void> {
    const promises = signatures.map((signature) => processSignature(signature, address, lp));
    await Promise.all(promises);
}


async function main() {
    try {
        const TOKEN_ADDRESS = "AtakVE4hj5KgbS58YzmCYrUwRqMNCnwaamUckk2Zpump";

        await initDbConnection();

        const pools = (await fetchPoolInfo(TOKEN_ADDRESS)).pop();
        const pool = new PublicKey(pools!.id);
        const info = await conn.getAccountInfo(pool);

        const decoded_info = LIQUIDITY_STATE_LAYOUT_V4.decode(info!.data);

        const lp: LP = {
            base_mint: new PublicKey(decoded_info.baseMint),
            base_vault: new PublicKey(decoded_info.baseVault),
            quote_mint: new PublicKey(decoded_info.quoteMint),
            quote_vault: new PublicKey(decoded_info.quoteVault),
        }

        let lastSignature = await getOldestSignatureProcessed(TOKEN_ADDRESS)
        
        let signatures: ConfirmedSignatureInfo[] = [];


        // Continue fetching until there are fewer than MAX_SIGNATURES signatures in the batch
        do {
            let signatures = lastSignature ? 
                await conn.getSignaturesForAddress(pool, { limit: 1000, before: lastSignature }, "confirmed") : 
                await conn.getSignaturesForAddress(pool, { limit: 1000 }, "confirmed");

            // time it
            
            // this should be split into batches processed in parallel
            // console.time("Processing batch of signatures");
            // for (const signature of signatures) {
            //     console.log(`Processing signature: ${signature.signature}`);
            //     // await processSignature(signature, TOKEN_ADDRESS, lp);
            // }
            // console.timeEnd("Processing batch of signatures");

            await processSignaturesInParallel(signatures, TOKEN_ADDRESS, lp);

            console.log("Processed batch of signatures.");
            break;

            // Check if we need to continue with the next batch
            lastSignature = signatures[signatures.length - 1].signature;

        } while (signatures.length === 1000);

        // console.log("All signatures processed.");
    } catch (error) {
        console.error("Error in main:", error);
    }
}

// Call main function to start processing
main().catch(console.error);