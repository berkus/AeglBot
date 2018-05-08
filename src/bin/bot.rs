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
    use aegl_bot::schema::activities::dsl::*;
    use aegl_bot::schema::alerts::dsl::*;
    use aegl_bot::schema::guardians::dsl::*;

    let connection = aegl_bot::establish_connection();

    let results = guardians
        // .filter(published.eq(true))
        // .limit(5)
        .load::<Guardian>(&connection)
        .expect("Error loading guardians");

    println!("Displaying {} guardians", results.len());
    for guar in results {
        println!("{}", guar);
    }

    let results2 = activities
        .load::<Activity>(&connection)
        .expect("Error loading activities");

    println!("Displaying {} activities", results2.len());
    for act in results2 {
        println!("{}", act.format_name());
    }

    let results3 = alerts
        .limit(5)
        .load::<Alert>(&connection)
        .expect("Error loading alerts");

    println!("Displaying {} alerts", results3.len());
    for alrt in results3 {
        println!("{}", alrt.title);
    }

    let guar = guardians
        .find(1)
        .first::<Guardian>(&connection)
        .expect("Guardian with id 1 not found");
    let results4 = PlannedActivity::belonging_to(&guar)
        .load::<PlannedActivity>(&connection)
        .expect("Error loading activities");

    println!("Displaying {} planned activities", results4.len());
    for act in results4 {
        println!("{}", act);
    }
}
