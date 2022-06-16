use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) camera_binding_network_port: String,
    pub(crate) hyt221_i2c_address: u16,
    pub(crate) vertical_servo_motor_gpio_pin: u8,
    pub(crate) horizontal_servo_motor_gpio_pin: u8,
    pub(crate) vertical_servo_motor_angle_percent_max: u8,
    pub(crate) horizontal_servo_motor_angle_percent_max: u8,
    pub(crate) vertical_servo_motor_angle_percent_min: u8,
    pub(crate) horizontal_servo_motor_angle_percent_min: u8,
    pub(crate) is_horizontal_servo_motor_inverted: bool,
    pub(crate) is_vertical_servo_motor_inverted: bool,
    pub(crate) vertical_servo_motor_initial_angle: u8,
    pub(crate) horizontal_servo_motor_initial_angle: u8,
    pub(crate) html_file_path: String,
    pub(crate) ip_stream_url: String, // note that "//localhost" will be replaced with the actual IP address
}