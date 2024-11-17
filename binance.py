import requests
import zipfile
import os
import shutil
import pandas as pd
import mysql.connector
from multiprocessing import Pool, cpu_count
from urllib.parse import urlparse

# Database configuration
db_config = {
    'host': 'localhost',
    'user': 'root',
    'password': 'password',
    'database': 'binance_data'
}

# List of URLs
file_urls = [
    'https://data.binance.vision/data/spot/daily/klines/PEPEUSDC/1m/PEPEUSDC-1m-',
    # Add more URLs here
]

def download_and_unzip(url):
    # Parse the file name from the URL
    file_name = os.path.basename(urlparse(url).path)
    zip_path = os.path.join('downloads', file_name)
    extract_dir = zip_path.replace('.zip', '')

    # Download the file
    print(f"Downloading {url}")
    response = requests.get(url, stream=True)
    response.raise_for_status()
    with open(zip_path, 'wb') as f:
        for chunk in response.iter_content(chunk_size=8192):
            f.write(chunk)

    # Unzip the file
    print(f"Unzipping {zip_path}")
    with zipfile.ZipFile(zip_path, 'r') as zip_ref:
        zip_ref.extractall(extract_dir)

    # Remove the zip file after extraction
    os.remove(zip_path)

    # Return the path to the CSV file
    csv_file = os.path.join(extract_dir, file_name.replace('.zip', '.csv'))
    return csv_file

def load_csv_to_database(csv_file, exchange, token):
    print(f"Loading data from {csv_file} to database")
    connection = mysql.connector.connect(**db_config)
    cursor = connection.cursor()

    # Read CSV into pandas for processing
    data = pd.read_csv(csv_file)
    for _, row in data.iterrows():
        sql = """INSERT INTO klines_spot (exchange, token, open_time, open, high, low, close, volume, close_time,
                 quote_asset_volume, number_of_trades, taker_buy_base_asset_volume, taker_buy_quote_asset_volume)
                 VALUES (%s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s, %s)
                 ON DUPLICATE KEY UPDATE open=open"""  # Adjust the update clause as needed
        cursor.execute(sql, (exchange, token, *row))

    connection.commit()
    cursor.close()
    connection.close()
    print(f"Data loaded from {csv_file}")

def process_file(url):
    try:
        csv_file = download_and_unzip(url)
        load_csv_to_database(csv_file, 'Binance', 'PEPEUSDC')
    except Exception as e:
        print(f"Failed to process {url}: {e}")

def main():
    os.makedirs('downloads', exist_ok=True)

    # Use multiprocessing Pool to process files in parallel
    with Pool(cpu_count()) as pool:
        pool.map(process_file, file_urls)

if __name__ == '__main__':
    main()