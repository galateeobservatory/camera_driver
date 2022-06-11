use std::env;
use tiny_http::{Server, Response, Method, Request};
use camera_driver::hyt_221::Hyt221;
use camera_driver::servo_motor::ServoMotor;

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
    let mut vertical_servo_motor = ServoMotor::new(27, 25, 63).unwrap();
    let mut horizontal_servo_motor = ServoMotor::new(10, 25, 63).unwrap();
    vertical_servo_motor.move_to_angle_percent(40).unwrap();
    horizontal_servo_motor.move_to_angle_percent(40).unwrap();
    for request in server.incoming_requests() {
        if *request.method() != Method::Get {
            let response = Response::from_string("Only GET requests are supported").with_status_code(405);
            let _ = request.respond(response);
            continue;
        }
        match request.url() {
            "/" => {
                match hyt221.read() {
                    Ok((humidity, temperature)) => {
                        let response = Response::from_string(format!("{{\"humidity\":{},\"temperature\":{}}}", humidity, temperature));
                        let _ = request.respond(response);
                    }
                    Err(e) => {
                        let response = Response::from_string(format!("{{\"error\":\"{}\"}}", e));
                        let _ = request.respond(response);
                    }
                }
            },
            "/up" => {
                shift_servo_motor_pos(&mut vertical_servo_motor, 2, request);
            }
            "/down" => {
                shift_servo_motor_pos(&mut vertical_servo_motor, -2, request);
            }
            "/left" => {
                shift_servo_motor_pos(&mut horizontal_servo_motor, 2, request);
            }
            "/right" => {
                shift_servo_motor_pos(&mut horizontal_servo_motor, -2, request);
            }
            _ => {
                let response = Response::from_string("404 Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }
}

fn shift_servo_motor_pos(servo_motor: &mut ServoMotor, angle_percent_shift: i8, request_callback: Request) {
    if let Some(current_angle) = servo_motor.current_angle_percent {
        match servo_motor.move_to_angle_percent((current_angle as i8 + angle_percent_shift) as u8) {
            Ok(_) => {
                let _ = request_callback.respond(Response::from_string("OK"));
            },
            Err(error_message) => {
                let _ = request_callback.respond(Response::from_string(error_message));
            }
        }
    }
}

// $gpio readall to retrieve pin layout