mod matrix;
use crate::matrix::login_and_sync;

mod store;

mod vectors;

use std::env;
use std::process::exit;

extern crate anyhow;

extern crate tokio;

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
