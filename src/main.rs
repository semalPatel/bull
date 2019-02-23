extern crate clap;

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
        let stock_name = matches.value_of("stock name").unwrap();
        println!("{}", stock_name);
}
