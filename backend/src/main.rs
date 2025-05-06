use actix_web::{web, App, HttpServer, HttpRequest, HttpResponse};
use actix_web::web::Data;
use actix_web_actors::ws;
use std::sync::{Arc, Mutex};

mod websocket;
mod game;
mod crypto;

use websocket::WsSession;
use game::Game;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let game = Data::new(Arc::new(Mutex::new(Game::default())));

    HttpServer::new(move || {
        App::new()
            .app_data(game.clone())
            .route("/ws/", web::get().to(ws_route))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn ws_route(
    req: HttpRequest,
    stream: web::Payload,
    game: web::Data<Arc<Mutex<Game>>>,
) -> HttpResponse {
    let game_clone = game.get_ref().clone(); // Clone the inner Arc<Mutex<Game>>
    let session = WsSession::new(game_clone); // Pass the cloned Arc<Mutex<Game>> to WsSession::new
    ws::start(session, &req, stream).unwrap()
}
