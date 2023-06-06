use actix::{Actor, Addr, AsyncContext, Context, Handler, Message, StreamHandler};
use actix_files::Files;
use actix_web::{
    web::{self, Bytes},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder,
};
use actix_web_actors::ws;
use lib::{dac::encoding::PreimageHash, message::UserMessage, place::PlaceState};
use std::{
    fs::{self, File, OpenOptions},
    io::prelude::*,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

struct AppState {
    place: PlaceState,
    connections: Vec<Addr<WsActor>>,
    tx_queue: Vec<Vec<u8>>,
    tx_log: File,
    external_message_log: File,
}

struct TextMessage(bytestring::ByteString);

impl Message for TextMessage {
    type Result = ();
}

struct WsActor {
    app_state: Arc<Mutex<AppState>>,
}

impl Actor for WsActor {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let mut app_state = self.app_state.lock().unwrap();
        app_state.connections.push(ctx.address());
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WsActor {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("Received Text message");
                let bin: Bytes = bytestring::ByteString::into_bytes(text.clone());
                match std::str::from_utf8(bin.as_ref()) {
                    Err(_) => (),
                    Ok(message) => match serde_json_wasm::from_str(message) {
                        Err(err) => println!("Erro parsing message: {}", err),
                        Ok(message) => {
                            println!("Parsed message successfully");
                            let mut app_state = self.app_state.lock().unwrap();
                            let (_result, message) = app_state.place.set_pixel(message);
                            let json = serde_json_wasm::to_string(&message).unwrap();
                            writeln!(app_state.tx_log, "{}", json).unwrap();
                            app_state.tx_queue.push(json.into_bytes());
                            for connection in &app_state.connections {
                                let _ = connection.do_send(TextMessage(text.clone()));
                            }
                        }
                    },
                }
            }
            _ => (),
        }
    }
}

impl Handler<TextMessage> for WsActor {
    type Result = ();

    fn handle(&mut self, msg: TextMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.text(msg.0);
    }
}

async fn new_connection(
    app_state: web::Data<Arc<Mutex<AppState>>>,
    req: HttpRequest,
    stream: web::Payload,
) -> Result<HttpResponse, Error> {
    ws::start(
        WsActor {
            app_state: app_state.get_ref().clone(),
        },
        &req,
        stream,
    )
}

async fn get_image(place: web::Data<Arc<Mutex<AppState>>>) -> impl Responder {
    let mut app_state = place.lock().unwrap();
    let bytes = app_state.place.get_image_bytes();

    return HttpResponse::Ok()
        .append_header(("Access-Control-Allow-Origin", "*"))
        .append_header(("Content-Length", bytes.len().to_string()))
        .append_header(("Cache-Control", "no-cache, no-store"))
        .content_type("image/png")
        .body(bytes);
}

struct PrinterActor {
    app_state: Arc<Mutex<AppState>>,
}

impl Actor for PrinterActor {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let sequencer_secret_key = std::env::var("SEQUENCER_SECRET_KEY").unwrap();
        let _rollup_address = std::env::var("ROLLUP_ADDRESS").unwrap();
        let rollup_preimages_dir = std::env::var("ROLLUP_PREIMAGES_DIR").unwrap();
        let () = std::fs::create_dir_all(rollup_preimages_dir.clone()).unwrap();

        let sk = tezos_crypto_rs::hash::SecretKeyEd25519::from_base58_check(
            sequencer_secret_key.as_str(),
        )
        .unwrap();

        ctx.run_interval(Duration::from_secs(10), move |actor, _| {
            let mut app_state = actor.app_state.lock().unwrap();
            let queue_len = app_state.tx_queue.len();
            if queue_len > 0 {
                println!("flushing {} txs from queue", queue_len);
                let preimages_dir = Path::new(rollup_preimages_dir.as_str()); // FIXME:
                let save_preimages = |hash: PreimageHash, preimage: Vec<u8>| {
                    let name = hex::encode(hash.as_ref());
                    let path = preimages_dir.join(name);

                    if let Err(e) = fs::write(&path, preimage) {
                        eprintln!("Failed to write preimage to {:?} due to {}.", path, e);
                    }
                };

                let root_hash = lib::dac::encoding::prepare_preimages(
                    app_state.tx_queue.clone(),
                    save_preimages,
                )
                .unwrap();

                let mut unprefixed_merkel_root: [u8; 32] = [0; 32];
                unprefixed_merkel_root.copy_from_slice(&root_hash.as_ref()[1..]);

                let sk = sk.as_ref().as_slice();
                let sk = ed25519_compact::SecretKey::from_slice(sk).unwrap();
                let message = lib::message::Message::new(sk, unprefixed_merkel_root);
                let str = serde_json::to_string(&message).unwrap();
                writeln!(app_state.external_message_log, "{}", str).unwrap();
                app_state.tx_queue.clear();
            } else {
                println!("Queue empty, skipping flush")
            }
        });
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("starging server");
    let external_message_log_path = std::env::var("ROLLUP_EXTERNAL_MESSAGE_LOG").unwrap();
    let image_path = std::env::var("ROLLUP_IMAGE").unwrap();
    let tx_log_path = std::env::var("ROLLUP_TX_LOG").unwrap();
    let frontend_path = std::env::var("TZPLACE_FRONTEND").unwrap();

    let tx_log = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(tx_log_path)
        .unwrap();

    let external_message_log = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(external_message_log_path)
        .unwrap();
    // Note: web::Data created _outside_ HttpServer::new closure
    let app_state = AppState {
        place: PlaceState::new(PathBuf::from(image_path)),
        connections: vec![],
        tx_queue: vec![],
        tx_log,
        external_message_log,
    };
    let place = web::Data::new(Arc::new(Mutex::new(app_state)));

    let printer_actor = PrinterActor {
        app_state: place.get_ref().clone(),
    };
    let _ = printer_actor.start(); // Ignoring the returned Addr<PrinterActor>

    HttpServer::new(move || {
        // move counter into the closure
        App::new()
            .app_data(place.clone()) // <- register the created data
            .route("/ws", web::get().to(new_connection))
            .route("/place.png", web::get().to(get_image))
            .service(Files::new("/", frontend_path.clone()).index_file("index.html"))
        // Serve static files from the `static` folder
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
