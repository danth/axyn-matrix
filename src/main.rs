mod store;
use crate::store::ResponseStore;

mod vectors;

fn main() {
    let mut database = ResponseStore::load().expect("Loading store");

    database.insert("Fish", "Today's fish is trout á la créme; enjoy your meal.").expect("Inserting response");

    let response = database.respond("Fish").expect("Getting response");
    println!("{}", response);
}
