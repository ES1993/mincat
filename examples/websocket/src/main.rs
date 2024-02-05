use mincat::{
    extract::{
        websocket::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::{get, Response, Router},
};

#[derive(Clone, Default)]
struct AppState {
    name: String,
}

#[tokio::main]
async fn main() {
    let router = Router::new().route(hello);

    mincat::router(router)
        .state(AppState::default())
        .run("127.0.0.1:3000")
        .await;
}

#[get("/hello")]
async fn hello(wsu: WebSocketUpgrade, State(app_state): State<AppState>) -> Response {
    wsu.on_upgrade(|socket| handle_socket(socket, app_state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    dbg!(state.name);
    socket
        .send(Message::Text("hello word".into()))
        .await
        .unwrap();
    while let Some(msg) = socket.recv().await {
        let msg = if let Ok(msg) = msg {
            msg
        } else {
            // client disconnected
            return;
        };

        if socket.send(msg).await.is_err() {
            // client disconnected
            return;
        }
    }
}
