#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate prettytable;

extern crate clap;
extern crate reqwest;

mod stock;

use clap::{Arg, App};
use console::Style;
use std::str::FromStr;

fn main() {
        let cyan = Style::new().cyan();
        let green = Style::new().green();
        let red = Style::new().red();

        let matches = App::new("bull")
                    .version("0.1.0")
                    .author("Semal Patel <semalpatel2596@gmail.com>")
                    .about("stocks from the cli")
                    .arg(Arg::with_name("stock symbol")
                                .required(true)
                                .takes_value(true)
                                .index(1)
                                .help("get price by symbol"))
                    .get_matches();
        let stock_name = matches.value_of("stock symbol").unwrap().to_string();
        let stock = stock::Stock::get_stock(stock_name);

        let mut change_percent = String::from(stock.quote.change_percent);
        change_percent.retain(|c| c != '%');
        let change_percent_numeric = f32::from_str(&change_percent).unwrap();

        let price = format!("${price}", price=stock.quote.price);

        if change_percent_numeric < 0.0 {
                ptable!(["Symbol", "Price", " ", "Change", "% Change"],
                        [cyan.apply_to(stock.quote.symbol), red.apply_to(price), red.apply_to("\u{25BC}"), stock.quote.change, change_percent]);
        }
        else {
                ptable!(["Symbol", "Price", " ", "Change", "% Change"],
                        [cyan.apply_to(stock.quote.symbol), green.apply_to(price), green.apply_to("\u{25B2}"), stock.quote.change, change_percent]);
        }
}
