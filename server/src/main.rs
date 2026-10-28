
use mafia_server::{log, websocket_connections::websocket_server::create_ws_server};
use std::{thread, time::Duration};


///
/// The Main function
/// 
/// # Examples
/// ![image](https://user-images.githubusercontent.com/64770632/217148805-aa33cad8-f1b8-45ff-954c-c57e5fdb54c9.png)
/// 
#[tokio::main]
async fn main() -> ! {

    // .env.local (untracked) takes precedence over .env, e.g. to give each git worktree its own port
    dotenv::from_filename(".env.local").ok();
    dotenv::dotenv().ok();
    let address = std::env::var("WS_ADDRESS").expect("Missing environment variabled WS_ADDRESS");

    loop {
        create_ws_server(&address).await;
        // This delay is only to make sure disconnect messages are sent before the server restarts
        thread::sleep(Duration::from_secs(1));
        log!(important "Server"; "Restarting...");
    }
}
