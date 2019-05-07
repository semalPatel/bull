#[macro_use]
extern crate serde_derive;
extern crate reqwest;
use reqwest::Error;

#[derive(Deserialize, Debug)]
struct Stock {
    symbol: String,
    open: String,
    high: String,
    low: String,
    price: String,
    previousClose: String,
    change: String,
    changePercent: String
}

fn getPrice(stockSymbol: String) -> Result<(), Error> {
    let api_endpoint = format!("https://www.alphavantage.co/query?function={quote}&symbol={symbol}&apikey={apikey}",
                                quote = "GLOBAL_QUOTE",
                                symbol = stockSymbol,
                                apikey = "UVLIY3BWZW09Z67G");
    println!("{}",api_endpoint);
    let mut response = reqwest::get(&api_endpoint)?;
    let stock: Vec<Stock> = response.json()?;
    println!("{:?}", stock);
    Ok(())
}

