mod websocket;

use warp::Filter;
use tokio::sync::broadcast;

#[tokio::main]
async fn main() {
    let (tx, _rx) = broadcast::channel(100);

    let chat = websocket::websocket_filter(tx.clone());

    let routes = warp::get().and(chat);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 3030))
        .await;
}