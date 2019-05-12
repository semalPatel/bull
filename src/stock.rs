extern crate reqwest;
extern crate serde;
extern crate serde_json;
use serde::{Serialize, Deserialize};

// static STRING_STOCK: &str = r#" 
//     {
//         "Global Quote": {
//             "01. symbol": "symbol",
//             "02. open": "open",
//             "03. high": "high",
//             "04. low": "low",
//             "05. price": "price",
//             "06. volume": "volume",
//             "07. latest trading day": "latest trading day",
//             "08. previous close": "previous close",
//             "09. change": "change",
//             "10. change percent": "change percent"
//         }
//     }"#;

#[derive(Serialize, Deserialize, Debug)]
pub struct StockInfo {
    #[serde(rename = "01. symbol")] pub symbol: String,
    #[serde(rename = "02. open")] pub open: String,
    #[serde(rename = "03. high")] pub high: String,
    #[serde(rename = "04. low")] pub low: String,
    #[serde(rename = "05. price")] pub price: String,
    #[serde(rename = "06. volume")] pub volume: String,
    #[serde(rename = "07. latest trading day")] pub latest_trading_day: String,
    #[serde(rename = "08. previous close")] pub previous_close: String,
    #[serde(rename = "09. change")] pub change: String,
    #[serde(rename = "10. change percent")] pub change_percent: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Stock {
    #[serde(rename = "Global Quote")] pub quote: StockInfo
}

impl Stock {

    pub fn get_stock(stock_symbol: String) -> Stock {
	    let stock_result = Stock::get_stock_result(stock_symbol);
        Stock::from(stock_result)
	}

	fn from(result_stock: Result<Stock, reqwest::Error>) -> Stock {
	    result_stock.unwrap()
	}

    fn get_stock_result(stock_symbol: String) -> Result<Stock, reqwest::Error> {
        let api_endpoint = format!("https://www.alphavantage.co/query?function={quote}&symbol={symbol}&apikey={apikey}",
                                    quote = "GLOBAL_QUOTE",
                                    symbol = stock_symbol,
                                    apikey = "UVLIY3BWZW09Z67G");
        Ok(reqwest::get(&api_endpoint)?.json()?)
    }
} 

