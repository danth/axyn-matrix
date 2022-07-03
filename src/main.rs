mod store;
use crate::store::ResponseStore;

mod vectors;
use crate::vectors::load_vectors;

fn main() {
    println!("Loading database");
    let database = ResponseStore::load().expect("Loading database");

    println!("Loading vectors");
    let vectors = load_vectors().expect("Error loading vectors");

    let vector = vectors.get("fish").expect("Getting vector");
    database.insert(vector, "Today's fish is trout á la créme; enjoy your meal.").expect("Inserting response");

    let response = database.respond(&vector).expect("Getting response");
    println!("{}", response);
}
