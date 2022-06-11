use std::env;
use tiny_http::{Server, Response, Method};
use camera_driver::hyt_221::Hyt221;

const DEFAULT_CAMERA_BINDING_NETWORK_PORT: &'static str = "0.0.0.0:8000";

fn main() {
    dotenv::dotenv().ok();
    let camera_binding_network_port =
        env::var("CAMERA_BINDING_NETWORK_PORT").unwrap_or_else(|_| {
            println!("CAMERA_BINDING_NETWORK_PORT not set, using default: {}", DEFAULT_CAMERA_BINDING_NETWORK_PORT);
            DEFAULT_CAMERA_BINDING_NETWORK_PORT.to_string()
        });
    let mut hyt221 = Hyt221::new(0x28).unwrap();
    let server = Server::http(camera_binding_network_port).unwrap();
    for request in server.incoming_requests() {
        println!("received request! method: {:?}, url: {:?}, headers: {:?}",
                 request.method(),
                 request.url(),
                 request.headers()
        );
        if *request.method() != Method::Get {
            let response = Response::from_string("Only GET requests are supported").with_status_code(405);
            request.respond(response).unwrap();
            continue;
        }
        match request.url() {
            "/" => {
                let (humidity, temperature) = hyt221.read().unwrap();
                let response = format!("{{\"humidity\": {}, \"temperature\": {} }}", humidity, temperature);
                let response = Response::from_string(response);
                request.respond(response).unwrap();
            }
            _ => {
                let response = Response::from_string("404 Not Found").with_status_code(404);
                request.respond(response).unwrap();
            }
        }
    }
}