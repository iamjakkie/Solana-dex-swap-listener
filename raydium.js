"use strict";
var __awaiter = (this && this.__awaiter) || function (thisArg, _arguments, P, generator) {
    function adopt(value) { return value instanceof P ? value : new P(function (resolve) { resolve(value); }); }
    return new (P || (P = Promise))(function (resolve, reject) {
        function fulfilled(value) { try { step(generator.next(value)); } catch (e) { reject(e); } }
        function rejected(value) { try { step(generator["throw"](value)); } catch (e) { reject(e); } }
        function step(result) { result.done ? resolve(result.value) : adopt(result.value).then(fulfilled, rejected); }
        step((generator = generator.apply(thisArg, _arguments || [])).next());
    });
};
var __generator = (this && this.__generator) || function (thisArg, body) {
    var _ = { label: 0, sent: function() { if (t[0] & 1) throw t[1]; return t[1]; }, trys: [], ops: [] }, f, y, t, g;
    return g = { next: verb(0), "throw": verb(1), "return": verb(2) }, typeof Symbol === "function" && (g[Symbol.iterator] = function() { return this; }), g;
    function verb(n) { return function (v) { return step([n, v]); }; }
    function step(op) {
        if (f) throw new TypeError("Generator is already executing.");
        while (g && (g = 0, op[0] && (_ = 0)), _) try {
            if (f = 1, y && (t = op[0] & 2 ? y["return"] : op[0] ? y["throw"] || ((t = y["return"]) && t.call(y), 0) : y.next) && !(t = t.call(y, op[1])).done) return t;
            if (y = 0, t) op = [op[0] & 2, t.value];
            switch (op[0]) {
                case 0: case 1: t = op; break;
                case 4: _.label++; return { value: op[1], done: false };
                case 5: _.label++; y = op[1]; op = [0]; continue;
                case 7: op = _.ops.pop(); _.trys.pop(); continue;
                default:
                    if (!(t = _.trys, t = t.length > 0 && t[t.length - 1]) && (op[0] === 6 || op[0] === 2)) { _ = 0; continue; }
                    if (op[0] === 3 && (!t || (op[1] > t[0] && op[1] < t[3]))) { _.label = op[1]; break; }
                    if (op[0] === 6 && _.label < t[1]) { _.label = t[1]; t = op; break; }
                    if (t && _.label < t[2]) { _.label = t[2]; _.ops.push(op); break; }
                    if (t[2]) _.ops.pop();
                    _.trys.pop(); continue;
            }
            op = body.call(thisArg, _);
        } catch (e) { op = [6, e]; y = 0; } finally { f = t = 0; }
        if (op[0] & 5) throw op[1]; return { value: op[0] ? op[1] : void 0, done: true };
    }
};
Object.defineProperty(exports, "__esModule", { value: true });
var web3_js_1 = require("@solana/web3.js");
var raydium_sdk_1 = require("@raydium-io/raydium-sdk");
require("@solana/spl-token");
var promise_1 = require("mysql2/promise");
require('dotenv').config();
var dbConfig = {
    host: process.env.DB_HOST,
    user: process.env.DB_USER,
    password: process.env.DB_PASSWORD,
    database: "trading",
};
var connection;
function initDbConnection() {
    return __awaiter(this, void 0, void 0, function () {
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0: return [4 /*yield*/, (0, promise_1.createConnection)(dbConfig)];
                case 1:
                    connection = _a.sent();
                    return [2 /*return*/];
            }
        });
    });
}
// Constants for the Solana environment
var SOLANA_DECIMALS = 9;
var RAYDIUM_CURVE = "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1"; // Replace with actual Raydium curve address
var PUMPFUN_CURVE = "PumpFunCurveAddress"; // Replace with actual PumpFun curve address
var WSOL_ADDRESS = "WSOLAddress"; // Replace with actual Wrapped SOL address
var conn = new web3_js_1.Connection(process.env.RPC_URL, "confirmed"); // Initialize Solana connection
// Utility sleep function
function sleep(ms) {
    return new Promise(function (resolve) { return setTimeout(resolve, ms); });
}
function isParsedInstruction(instruction) {
    return instruction.parsed !== undefined;
}
function insertToDb(row) {
    return __awaiter(this, void 0, void 0, function () {
        var error_1;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4 /*yield*/, connection.query("INSERT INTO TRADES (token, address, sol_amount, token_amount, price, tx, timestamp) VALUES (?, ?, ?, ?, ?, ?, ?)", [row.token, row.address, row.sol_amount, row.token_amount, row.price, row.tx, row.timestamp])];
                case 1:
                    _a.sent();
                    return [3 /*break*/, 3];
                case 2:
                    error_1 = _a.sent();
                    console.error("Error inserting row:", error_1);
                    console.error(row);
                    return [3 /*break*/, 3];
                case 3: return [2 /*return*/];
            }
        });
    });
}
// Main function to process a signature
function processSignature(signature, address, lp) {
    var _a, _b, _c, _d, _e, _f, _g, _h, _j, _k;
    return __awaiter(this, void 0, void 0, function () {
        var tx, instructionTypes, tx_meta, accounts, base_index, quote_index, trader, timestamp, lpPreTokenBalance, lpPostTokenBalance, lpPreSolBalance, lpPostSolBalance, solChange, tokenChange, price, row;
        return __generator(this, function (_l) {
            switch (_l.label) {
                case 0: return [4 /*yield*/, conn.getParsedTransaction(signature.signature, {
                        maxSupportedTransactionVersion: 1,
                    })];
                case 1:
                    tx = _l.sent();
                    if (!tx) return [3 /*break*/, 4];
                    if ((_a = tx.meta) === null || _a === void 0 ? void 0 : _a.err) {
                        // console.log(`Error in transaction: ${signature.signature}`);
                        return [2 /*return*/];
                    }
                    instructionTypes = void 0;
                    try {
                        instructionTypes = tx.transaction.message.instructions
                            .filter(function (instruction) { var _a, _b; return isParsedInstruction(instruction) && (((_a = instruction.parsed) === null || _a === void 0 ? void 0 : _a.info.mint) === address) && (((_b = instruction.parsed) === null || _b === void 0 ? void 0 : _b.type) === "burn"); })
                            .map(function (instruction) { return instruction.parsed; });
                    }
                    catch (error) {
                        // console.error(`Error extracting instructions for ${signature.signature}:`, error);
                        return [2 /*return*/];
                    }
                    // Check if the transaction was unsuccessful or involved a burn operation
                    if (instructionTypes.length > 0) {
                        // console.log(`burn error: ${signature.signature}`);
                        return [2 /*return*/];
                    }
                    tx_meta = tx.meta;
                    accounts = tx.transaction.message.accountKeys;
                    base_index = accounts.findIndex(function (account) { return account.pubkey.toString() === lp.base_vault.toString(); });
                    quote_index = accounts.findIndex(function (account) { return account.pubkey.toString() === lp.quote_vault.toString(); });
                    trader = accounts[0];
                    return [4 /*yield*/, conn.getBlockTime(tx.slot)];
                case 2:
                    timestamp = (_b = _l.sent()) !== null && _b !== void 0 ? _b : 0;
                    lpPreTokenBalance = void 0, lpPostTokenBalance = void 0;
                    lpPreTokenBalance = ((_d = (_c = tx_meta === null || tx_meta === void 0 ? void 0 : tx_meta.preTokenBalances) === null || _c === void 0 ? void 0 : _c.filter(function (balance) { return balance.owner === RAYDIUM_CURVE && balance.mint === address; }).pop()) === null || _d === void 0 ? void 0 : _d.uiTokenAmount.uiAmount) || 0;
                    lpPostTokenBalance = ((_f = (_e = tx_meta === null || tx_meta === void 0 ? void 0 : tx_meta.postTokenBalances) === null || _e === void 0 ? void 0 : _e.filter(function (balance) { return balance.owner === RAYDIUM_CURVE && balance.mint === address; }).pop()) === null || _f === void 0 ? void 0 : _f.uiTokenAmount.uiAmount) || 0;
                    if ((lpPreTokenBalance === 0 && lpPostTokenBalance === 0) || lpPreTokenBalance === lpPostTokenBalance) {
                        return [2 /*return*/];
                    }
                    lpPreSolBalance = void 0, lpPostSolBalance = void 0;
                    if (lp.base_mint.toString() === WSOL_ADDRESS) {
                        lpPreSolBalance = ((_g = tx_meta === null || tx_meta === void 0 ? void 0 : tx_meta.preBalances) === null || _g === void 0 ? void 0 : _g[quote_index]) || 0;
                        lpPostSolBalance = ((_h = tx_meta === null || tx_meta === void 0 ? void 0 : tx_meta.postBalances) === null || _h === void 0 ? void 0 : _h[quote_index]) || 0;
                    }
                    else {
                        lpPreSolBalance = ((_j = tx_meta === null || tx_meta === void 0 ? void 0 : tx_meta.preBalances) === null || _j === void 0 ? void 0 : _j[base_index]) || 0;
                        lpPostSolBalance = ((_k = tx_meta === null || tx_meta === void 0 ? void 0 : tx_meta.postBalances) === null || _k === void 0 ? void 0 : _k[base_index]) || 0;
                    }
                    lpPreSolBalance = lpPreSolBalance / Math.pow(10, SOLANA_DECIMALS);
                    lpPostSolBalance = lpPostSolBalance / Math.pow(10, SOLANA_DECIMALS);
                    solChange = -(lpPostSolBalance - lpPreSolBalance);
                    tokenChange = -(lpPostTokenBalance - lpPreTokenBalance);
                    price = Math.abs(solChange / tokenChange);
                    row = {
                        token: address.toString(),
                        address: trader.pubkey.toString(),
                        sol_amount: solChange,
                        token_amount: tokenChange,
                        price: price,
                        tx: tx.transaction.signatures[0],
                        timestamp: timestamp,
                    };
                    return [4 /*yield*/, insertToDb(row)];
                case 3:
                    _l.sent();
                    _l.label = 4;
                case 4: return [2 /*return*/];
            }
        });
    });
}
function fetchPoolInfo(mint1) {
    return __awaiter(this, void 0, void 0, function () {
        var url, response, data, error_2;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    url = "https://api-v3.raydium.io/pools/info/mint?mint1=".concat(mint1, "&poolType=all&poolSortField=default&sortType=desc&pageSize=10&page=1");
                    _a.label = 1;
                case 1:
                    _a.trys.push([1, 4, , 5]);
                    return [4 /*yield*/, fetch(url, {
                            method: 'GET',
                            headers: {
                                'Accept': 'application/json'
                            }
                        })];
                case 2:
                    response = _a.sent();
                    if (!response.ok) {
                        throw new Error("HTTP error! Status: ".concat(response.status));
                    }
                    return [4 /*yield*/, response.json()];
                case 3:
                    data = _a.sent();
                    return [2 /*return*/, data.data.data]; // Adjust this path based on the exact JSON structure
                case 4:
                    error_2 = _a.sent();
                    console.error("Error fetching pool info:", error_2);
                    return [2 /*return*/, []];
                case 5: return [2 /*return*/];
            }
        });
    });
}
function getOldestSignatureProcessed(token_address) {
    return __awaiter(this, void 0, void 0, function () {
        var rows, timestamps, error_3;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    _a.trys.push([0, 2, , 3]);
                    return [4 /*yield*/, connection.query("SELECT tx FROM TRADES WHERE token = ? ORDER BY timestamp LIMIT 1", [token_address])];
                case 1:
                    rows = (_a.sent())[0];
                    timestamps = rows;
                    return [2 /*return*/, timestamps[0].tx]; // Access the timestamp of the first row
                case 2:
                    error_3 = _a.sent();
                    console.error("Error fetching oldest signature:", error_3);
                    return [3 /*break*/, 3];
                case 3: return [2 /*return*/];
            }
        });
    });
}
function processSignaturesInParallel(signatures, address, lp) {
    return __awaiter(this, void 0, void 0, function () {
        var promises;
        return __generator(this, function (_a) {
            switch (_a.label) {
                case 0:
                    promises = signatures.map(function (signature) { return processSignature(signature, address, lp); });
                    return [4 /*yield*/, Promise.all(promises)];
                case 1:
                    _a.sent();
                    return [2 /*return*/];
            }
        });
    });
}
function main() {
    return __awaiter(this, void 0, void 0, function () {
        var TOKEN_ADDRESS, pools, pool, info, decoded_info, lp, lastSignature, signatures, signaturesCount, batchSize, cnt, signatures_1, _a, i, error_4;
        return __generator(this, function (_b) {
            switch (_b.label) {
                case 0:
                    _b.trys.push([0, 17, , 18]);
                    TOKEN_ADDRESS = "Em4rcuhX6STfB7mxb66dUXDmZPYCjDiQFthvzSzpump";
                    return [4 /*yield*/, initDbConnection()];
                case 1:
                    _b.sent();
                    return [4 /*yield*/, fetchPoolInfo(TOKEN_ADDRESS)];
                case 2:
                    pools = (_b.sent()).pop();
                    pool = new web3_js_1.PublicKey(pools.id);
                    return [4 /*yield*/, conn.getAccountInfo(pool)];
                case 3:
                    info = _b.sent();
                    decoded_info = raydium_sdk_1.LIQUIDITY_STATE_LAYOUT_V4.decode(info.data);
                    lp = {
                        base_mint: new web3_js_1.PublicKey(decoded_info.baseMint),
                        base_vault: new web3_js_1.PublicKey(decoded_info.baseVault),
                        quote_mint: new web3_js_1.PublicKey(decoded_info.quoteMint),
                        quote_vault: new web3_js_1.PublicKey(decoded_info.quoteVault),
                    };
                    return [4 /*yield*/, getOldestSignatureProcessed(TOKEN_ADDRESS)];
                case 4:
                    lastSignature = _b.sent();
                    signatures = [];
                    signaturesCount = 0;
                    batchSize = 100;
                    cnt = 0;
                    _b.label = 5;
                case 5:
                    if (!lastSignature) return [3 /*break*/, 7];
                    return [4 /*yield*/, conn.getSignaturesForAddress(pool, { limit: 1000, before: lastSignature }, "confirmed")];
                case 6:
                    _a = _b.sent();
                    return [3 /*break*/, 9];
                case 7: return [4 /*yield*/, conn.getSignaturesForAddress(pool, { limit: 1000 }, "confirmed")];
                case 8:
                    _a = _b.sent();
                    _b.label = 9;
                case 9:
                    signatures_1 = _a;
                    signaturesCount = signatures_1.length;
                    cnt += signaturesCount;
                    i = 0;
                    _b.label = 10;
                case 10:
                    if (!(i < signatures_1.length)) return [3 /*break*/, 14];
                    return [4 /*yield*/, processSignaturesInParallel(signatures_1.slice(i, i + batchSize), TOKEN_ADDRESS, lp)];
                case 11:
                    _b.sent();
                    return [4 /*yield*/, sleep(1200)];
                case 12:
                    _b.sent();
                    _b.label = 13;
                case 13:
                    i += batchSize;
                    return [3 /*break*/, 10];
                case 14:
                    // Check if we need to continue with the next batch
                    lastSignature = signatures_1[signatures_1.length - 1].signature;
                    _b.label = 15;
                case 15:
                    if (signaturesCount === 1000) return [3 /*break*/, 5];
                    _b.label = 16;
                case 16:
                    console.log("Processed ".concat(cnt, " signatures."));
                    console.log("All signatures processed.");
                    return [3 /*break*/, 18];
                case 17:
                    error_4 = _b.sent();
                    console.error("Error in main:", error_4);
                    return [3 /*break*/, 18];
                case 18: return [2 /*return*/];
            }
        });
    });
}
// Call main function to start processing
main().catch(console.error);
