// Parallel Rust implementation of the bot
//
// To make it usable it misses natty parsing lib implementation in rust
// (yeah, i'd prefer native, although there are ways to use natty through jlink
// or take python equivalent from https://dateparser.readthedocs.io/en/latest/)
extern crate aegl_bot;
extern crate diesel;

use aegl_bot::models::*;
use diesel::prelude::*;

fn main() {
    use aegl_bot::schema::guardians::dsl::*;

    let connection = aegl_bot::establish_connection();

    let results = guardians
        // .filter(published.eq(true))
        .limit(5)
        .load::<Guardian>(&connection)
        .expect("Error loading guardians");

    println!("Displaying {} guardians", results.len());
    for guar in results {
        println!("{}", guar.psn_name);
        println!("-----------\n");
        println!("{}", guar.telegram_name);
    }
}
