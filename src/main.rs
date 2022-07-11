mod matrix_api;
mod matrix_body;
mod matrix_event_handlers;
mod store;
mod vectors;

use std::{env, process::exit};

use crate::matrix_event_handlers::login_and_sync;

extern crate anyhow;

extern crate tokio;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if let (Some(homeserver_url), Some(username), Some(password), Some(device_id)) = (
        env::args().nth(1),
        env::args().nth(2),
        env::args().nth(3),
        env::args().nth(4),
    ) {
        login_and_sync(homeserver_url, &username, &password, &device_id).await?;
        Ok(())
    } else {
        eprintln!("Required arguments: «homeserver URL» «username» «password» «device ID»");
        exit(1)
    }
}
