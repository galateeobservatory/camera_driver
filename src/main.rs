use camera_driver::hyt_221::Hyt221;
use camera_driver::servo_motor::ServoMotor;
use config_file::FromConfigFile;
use serde::Deserialize;
use std::str::FromStr;
use std::{env, fs};
use tiny_http::{Header, Method, Request, Response, Server};

const CAMERA_URL_HTML_PATTERN: &'static str = "{CAMERA_URL}";

fn main() {
    if env::args().len() < 2 {
        eprintln!("Usage: {} <config_file.toml>", env::args().nth(0).unwrap());
        return;
    }
    let config = Config::from_config_file(env::args().nth(1).unwrap()).unwrap();

    let html_file_content = fs::read_to_string(config.html_file_path).unwrap();

    let mut hyt221 = Hyt221::new(config.hyt221_i2c_address).unwrap();

    let mut vertical_servo_motor = ServoMotor::new(
        config.vertical_servo_motor_gpio_pin,
        config.vertical_servo_motor_angle_percent_min,
        config.vertical_servo_motor_angle_percent_max,
    )
    .unwrap();
    let mut horizontal_servo_motor = ServoMotor::new(
        config.horizontal_servo_motor_gpio_pin,
        config.horizontal_servo_motor_angle_percent_min,
        config.horizontal_servo_motor_angle_percent_max,
    )
    .unwrap();
    vertical_servo_motor
        .move_to_angle_percent(config.vertical_servo_motor_initial_angle)
        .unwrap();
    horizontal_servo_motor
        .move_to_angle_percent(config.horizontal_servo_motor_initial_angle)
        .unwrap();
    let vertical_motor_inverter = match config.is_vertical_servo_motor_inverted {
        true => -1,
        false => 1,
    };
    let horizontal_motor_inverter = match config.is_horizontal_servo_motor_inverted {
        true => -1,
        false => 1,
    };

    let server = Server::http(config.camera_binding_network_port).unwrap();
    for request in server.incoming_requests() {
        if *request.method() != Method::Get {
            let response =
                Response::from_string("Only GET requests are supported").with_status_code(405);
            let _ = request.respond(response);
            continue;
        }

        config.ip_stream_url.starts_with("http://");
        let request_host_header_value = request
            .headers()
            .iter()
            .find(|header| header.field.to_string() == "Host")
            .unwrap()
            .value
            .to_string();
        println!("{} {}", request.method(), request_host_header_value);
        let requested_ip_addr_as_str = request_host_header_value.split(":").nth(0).unwrap();
        let new_ip_stream_url = config
            .ip_stream_url
            .replace("//localhost", &format!("//{}", requested_ip_addr_as_str));

        match request.url() {
            "/" => {
                let html_content =
                    html_file_content.replace(CAMERA_URL_HTML_PATTERN, &*new_ip_stream_url);
                let response = Response::from_string(html_content)
                    .with_header(Header::from_str("Content-Type: text/html").unwrap());
                let _ = request.respond(response);
            }
            "/humiditytemp" => match hyt221.read() {
                Ok((humidity, temperature)) => {
                    let response = Response::from_string(format!(
                        "{{\"humidity\":{},\"temperature\":{}}}",
                        humidity, temperature
                    ));
                    let _ = request.respond(response);
                }
                Err(e) => {
                    let response = Response::from_string(format!("{{\"error\":\"{}\"}}", e));
                    let _ = request.respond(response);
                }
            },
            "/up" => {
                shift_servo_motor_pos(
                    &mut vertical_servo_motor,
                    vertical_motor_inverter * 2,
                    request,
                );
            }
            "/down" => {
                shift_servo_motor_pos(
                    &mut vertical_servo_motor,
                    vertical_motor_inverter * -2,
                    request,
                );
            }
            "/left" => {
                shift_servo_motor_pos(
                    &mut horizontal_servo_motor,
                    horizontal_motor_inverter * 2,
                    request,
                );
            }
            "/right" => {
                shift_servo_motor_pos(
                    &mut horizontal_servo_motor,
                    horizontal_motor_inverter * -2,
                    request,
                );
            }
            _ => {
                let response = Response::from_string("404 Not Found").with_status_code(404);
                let _ = request.respond(response);
            }
        }
    }
}

fn shift_servo_motor_pos(
    servo_motor: &mut ServoMotor,
    angle_percent_shift: i8,
    request_callback: Request,
) {
    if let Some(current_angle) = servo_motor.current_angle_percent {
        match servo_motor.move_to_angle_percent((current_angle as i8 + angle_percent_shift) as u8) {
            Ok(_) => {
                let _ = request_callback.respond(Response::from_string("{\"status\": \"OK\"}"));
            }
            Err(error_message) => {
                let _ = request_callback.respond(Response::from_string(error_message));
            }
        }
    }
}

#[derive(Deserialize)]
struct Config {
    camera_binding_network_port: String,
    hyt221_i2c_address: u16,
    vertical_servo_motor_gpio_pin: u8,
    horizontal_servo_motor_gpio_pin: u8,
    vertical_servo_motor_angle_percent_max: u8,
    horizontal_servo_motor_angle_percent_max: u8,
    vertical_servo_motor_angle_percent_min: u8,
    horizontal_servo_motor_angle_percent_min: u8,
    is_horizontal_servo_motor_inverted: bool,
    is_vertical_servo_motor_inverted: bool,
    vertical_servo_motor_initial_angle: u8,
    horizontal_servo_motor_initial_angle: u8,
    html_file_path: String,
    ip_stream_url: String, // note that "//localhost" will be replaced with the actual IP address
}

// $gpio readall to retrieve pin layout
// Use anyhow for error handling
