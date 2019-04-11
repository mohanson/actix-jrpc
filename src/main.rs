use actix_web::http::Method;
use actix_web::{
    middleware, server, App, AsyncResponder, Error, HttpMessage, HttpRequest, HttpResponse,
};
use futures::{Future, Stream};
use serde_json;
use serde_json::Value;
use std::sync::Arc;

mod convention;

fn rpc_main(req: HttpRequest<AppState>) -> impl Future<Item = HttpResponse, Error = Error> {
    req.payload()
        .concat2()
        .from_err()
        .and_then(move |body| {
            let reqjson: convention::Request = match serde_json::from_slice(body.as_ref()) {
                Ok(ok) => ok,
                Err(_) => {
                    let r = convention::Response {
                        jsonrpc: String::from(convention::JSONRPC_VERSION),
                        result: Value::Null,
                        error: Some(convention::ErrorData::std(-32700)),
                        id: Value::Null,
                    };
                    return Ok(HttpResponse::Ok()
                        .content_type("application/json")
                        .body(&serde_json::to_string(&r).unwrap()));
                }
            };
            println!("{:?}", reqjson);

            let app_state = req.state();
            let mut result = convention::Response::default();
            result.id = reqjson.id.clone();

            match reqjson.method.as_str() {
                "peerCount" => {
                    let r = app_state.network.peer_count();
                    result.result = Value::from(r);
                }
                _ => {
                    result.error = Some(convention::ErrorData::std(-32601));
                }
            };

            Ok(HttpResponse::Ok()
                .content_type("application/json")
                .body(result.dump()))
        })
        .responder()
}

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

fn main() {
    std::env::set_var("RUST_LOG", "info");
    env_logger::init();

    let network = Arc::new(ObjNetwork {});

    let sys = actix::System::new("actix_jrpc");
    server::new(move || {
        let app_state = AppState::new(network.clone());
        App::with_state(app_state)
            .middleware(middleware::Logger::default())
            .resource("/", |r| r.method(Method::POST).with_async(rpc_main))
    })
    .bind("127.0.0.1:8080")
    .unwrap()
    .workers(1)
    .start();

    let _ = sys.run();
}
