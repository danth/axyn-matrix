mod matrix;
use crate::matrix::login_and_sync;

mod store;
use crate::store::ResponseStore;

mod vectors;

use std::env;
use std::process::exit;

extern crate anyhow;

extern crate tokio;

/*
fn main() {
    let mut database = ResponseStore::load().expect("Loading store");

    database.insert("Fish", "Today's fish is trout á la créme; enjoy your meal.").expect("Inserting response");

    let response = database.respond("Fish").expect("Getting response");
    println!("{}", response);
}
*/

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let (Some(homeserver_url), Some(username), Some(password), Some(device_id)) =
        (env::args().nth(1), env::args().nth(2), env::args().nth(3), env::args().nth(4)) {

        login_and_sync(homeserver_url, &username, &password, &device_id).await?;
        Ok(())
    } else {
        eprintln!("Required arguments: «homeserver URL» «username» «password» «device ID»");
        exit(1)
    }
}
