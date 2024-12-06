const unzipper = require('unzipper');
const mysql = require('mysql2/promise');
const fs = require('fs');
const path = require('path');
const readline = require('readline');
const request = require('request');

const datasets = [
    { 
        path: "https://data.binance.vision/data/spot/daily/klines/PEPEUSDC/1m/", 
        filenamePrefix: "PEPEUSDC-1m",
        exchange: "BINANCE",
        token: "PEPE",
        type: "SPOT",
    },
    // {   
    //     path: "https://data.binance.vision/data/futures/um/daily/klines/1000PEPEUSDC/1m/", 
    //     filenamePrefix: "1000PEPEUSDC-1m",
    //     exchange: "BINANCE",
    //     token: "PEPE",
    //     type: "PERP"
    // },
    // { 
    //     path: "https://data.binance.vision/data/futures/um/daily/metrics/1000PEPEUSDC/", 
    //     filenamePrefix: "1000PEPEUSDC-metrics",
    //     exchange: "BINANCE",
    //     token: "PEPE",
    //     type: "METRICS"
    // }
];

const startYear = 2024;
const startMonth = 9; // September (1-based index)

// MySQL configuration
const dbConfig = {
    host: process.env.DB_HOST,
    user: process.env.DB_USER,
    password: process.env.DB_PASSWORD,
    database: "trading",
};

// Global MySQL connection
let connection;

// Initialize the database connection
async function initDbConnection() {
    connection = await mysql.createConnection(dbConfig);
    console.log('Database connection established.');
}

function generateUrlsFromDateRange(datasets) {
    const urls = [];
    const today = new Date();
    let year = startYear;
    let month = startMonth;

    while (year < today.getFullYear() || (year === today.getFullYear() && month <= today.getMonth() + 1)) {
        const monthStr = month.toString().padStart(2, '0');
        const daysInMonth = new Date(year, month, 0).getDate();

        // Generate daily URLs for each dataset
        for (const dataset of datasets) {
            const tableName = dataset.type === 'METRICS' ? 'METRICS' : 'KLINES_'+dataset.type;
            for (let day = 1; day <= daysInMonth; day++) {
                const dayStr = day.toString().padStart(2, '0');
                const dateStr = `${year}-${monthStr}-${dayStr}`;
                const filename = `${dataset.filenamePrefix}-${dateStr}.zip`;
                // check if the file exists (change zip to csv)
                if (fs.existsSync(filename.replace('.zip', '.csv'))) {
                    console.log(`File ${filename} already exists`);
                    continue;
                }
                urls.push({
                    url: `${dataset.path}${filename}`,
                    tableName: tableName,
                    exchange: dataset.exchange,
                    token: dataset.token,
                });
            }
        }

        month++;
        if (month > 12) {
            month = 1;
            year++;
        }
    }
    return urls;
}

function downloadZip(url) {
    console.log(`Starting download from ${url}`);

    const output = path.join(process.cwd(), url.split('/').pop());
    console.log(`Output path: ${output}`);

    return new Promise((resolve, reject) => {
        request({ url: url, encoding: null }, (err, resp, body) => {
            if (err) {
                return reject(err);
            }

            fs.writeFile(output, body, (err) => {
                if (err) {
                    return reject(err);
                }
                console.log("File downloaded and written to disk.");
                resolve(output);
            });
        });
    });
}

function unzipFile(filePath) {
    // Determine the expected CSV file name by replacing .zip with .csv
    const csvFileName = path.basename(filePath, '.zip') + '.csv';
    const extractDir = path.join(process.cwd(), path.basename(filePath, '.zip'));

    return new Promise((resolve, reject) => {
        fs.createReadStream(filePath)
            .pipe(unzipper.Extract({ path: "./" }))
            .on("close", () => {
                console.log("Files unzipped successfully.");

                // Delete the zip file after unzipping
                fs.unlinkSync(filePath);

                // Resolve with the CSV file name (not the path)
                resolve(csvFileName);
            })
            .on("error", (err) => reject(err));
    });
}

async function loadCsvToDatabase(csvFilePath, tableName, exchange, token) {
    console.log(`Loading data from ${csvFilePath}`);

    const rl = readline.createInterface({
        input: fs.createReadStream(csvFilePath),
        crlfDelay: Infinity
    });

    for await (const line of rl) {
        let data = line.split(',');  // Adjust based on CSV column structure
        data.unshift(token);
        data.unshift(exchange);
        data.pop();
        const query = `INSERT INTO ${tableName} (
                        exchange, token, open_time, open, high, low, close, volume, close_time,
                        quote_asset_volume, number_of_trades, taker_buy_base_asset_volume, 
                        taker_buy_quote_asset_volume
                    ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`
        try {
            await connection.query(query, data);
        } catch {
            console.log(`Error loading data from ${csvFilePath}`);
            break;
        }
    }
}

async function processData(url, tableName, exchange, token) {
    try {
        const zipFilePath = await downloadZip(url);
        const extractPath = await unzipFile(zipFilePath);

        const csvFilePath = path.join("./", extractPath);
        await loadCsvToDatabase(csvFilePath, tableName, exchange, token);
        console.log(`Data from ${csvFile} loaded successfully.`);

    } catch (error) {
        console.error(`Error processing data from ${url}: ${error.message}`);
    }
}

async function main() {
    try {
        await initDbConnection();

        // Generate URLs for each day from September onwards
        const urls = generateUrlsFromDateRange(datasets);

        // Process each generated URL
        for (const { url, tableName, exchange, token } of urls) {
            await processData(url, tableName, exchange, token);
        }

    } catch (error) {
        console.error('Error:', error.message);
    } finally {
        if (connection) {
            await connection.end();
            console.log('Database connection closed.');
        }
    }
}

// Run the main function
main().catch(console.error);