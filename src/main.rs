extern crate clap;
extern crate reqwest;

mod stock;

use clap::{Arg, App};

fn main() {
        let matches = App::new("bull")
                    .version("0.1.0")
                    .author("Semal Patel <semalpatel2596@gmail.com>")
                    .about("stocks from the cli")
                    .arg(Arg::with_name("stock name")
                                .required(true)
                                .takes_value(true)
                                .index(1)
                                .help("get price by name"))
                    .get_matches();
        let stock_name = matches.value_of("stock name").unwrap().to_string();
        let stock = stock::Stock::get_stock(stock_name);
        println!("{} - ${}", stock.quote.symbol, stock.quote.price);
}
