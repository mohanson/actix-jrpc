use actix_web::http::Method;
use actix_web::{
    middleware, server, App, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse,
};
use futures::{Future, Stream};
use json::object;
use json::JsonValue;
use log::info;
use std::sync::Arc;

pub trait ImplNetwork {
    fn peer_count(&self) -> u32;
}
pub struct ObjNetwork {}
impl ImplNetwork for ObjNetwork {
    fn peer_count(&self) -> u32 {
        42
    }
}

#[derive(Clone)]
pub struct AppState {
    network: Arc<ImplNetwork>,
}

impl AppState {
    pub fn new(network: Arc<ImplNetwork>) -> Self {
        Self { network: network }
    }
}

// fn rpc_peer_count(req: HttpRequest<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
// }

fn echo(req: HttpRequest<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    req.payload()
        .concat2()
        .from_err()
        .and_then(move |body| {
            let result = json::parse(std::str::from_utf8(&body).unwrap());
            println!("{:?}", result);
            let injson: JsonValue = match result {
                Ok(v) => v,
                Err(e) => object! {"err" => e.to_string() },
            };
            let obj = match injson {
                JsonValue::Object(ref obj) => obj,
                _ => panic!("No entry"),
            };
            info!("{:?}", obj.get("jsonrpc").unwrap());
            info!("{:?}", obj.get("method").unwrap());
            info!("{:?}", obj.get("params").unwrap());
            info!("{:?}", obj.get("id").unwrap().as_i32().unwrap());

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(injson.dump()))
        })
        .responder()
}

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let network = Arc::new(ObjNetwork {});

    let sys = actix::System::new("actix_jrpc");
    server::new(move || {
        let app_state = AppState::new(network.clone());
        App::with_state(app_state)
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.method(Method::POST).with_async(echo))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .workers(1)
    .start();

    let _ = sys.run();
}
