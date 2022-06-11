use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

#[derive(Debug)]
pub struct ServoMotor {
    gpio_pin: OutputPin,
    pub pin_number: u8,
    pub min_angle_percent: u8,
    pub max_angle_percent: u8,
    pub current_angle_percent: Option<u8>, // can ben unknown at the beginning
}

impl ServoMotor {
    const SERVO_MOTOR_MIN_ANGLE_PERCENT: u8 = 0;
    const SERVO_MOTOR_MAX_ANGLE_PERCENT: u8 = 100;
    const SERVO_MOTOR_MIN_PULSE_WIDTH: u16 = 500;
    const SERVO_MOTOR_PULSE_PERIOD: u16 = 20000;
    const PERIOD_NUMBER_FOR_UNKNOWN_PREVIOUS_ANGLE: u16 = 150;
    const PERIOD_NUMBER_PER_ANGLE_DEGREE: u16 = 5;
    const SERVO_MIN_MICRO_SECONDS_SENSITIVITY: u16 = 20;

    pub fn new(pin_number: u8, min_angle_percent: u8, max_angle_percent: u8) -> Result<Self, &'static str> {
        if min_angle_percent > max_angle_percent {
            return Err("min_angle_percent must be less than max_angle_percent");
        }
        if min_angle_percent < Self::SERVO_MOTOR_MIN_ANGLE_PERCENT || max_angle_percent > Self::SERVO_MOTOR_MAX_ANGLE_PERCENT {
            return Err("min_angle_percent and max_angle_percent must be between 0 and 180");
        }
        Ok(ServoMotor {
            gpio_pin: Gpio::new().map_err(|_| "Failed to create gpio")?.get(pin_number).map_err(|_| "Failed to get pin")?.into_output(),
            pin_number,
            min_angle_percent,
            max_angle_percent,
            current_angle_percent: None,
        })
    }

    pub fn move_to_angle_percent(&mut self, angle_percent: u8) -> Result<(), &'static str> {
        println!("ServoMotor::move_to_angle_percent: angle_percent: {}", angle_percent);
        if !(self.min_angle_percent..self.max_angle_percent).contains(&angle_percent) {
            return Err("Angle out of range");
        }
        match self.current_angle_percent {
            Some(current_angle_percent) => {
                for intermediate_angle_percent in current_angle_percent..=angle_percent {
                    self.send_angle_position_for_period_number(intermediate_angle_percent, Self::PERIOD_NUMBER_PER_ANGLE_DEGREE);
                }
                self.current_angle_percent = Some(angle_percent);
            },
            None => {
                self.send_angle_position_for_period_number(angle_percent, Self::PERIOD_NUMBER_FOR_UNKNOWN_PREVIOUS_ANGLE);
                self.current_angle_percent = Some(angle_percent);
            }
        }
        return Ok(());
    }

    fn send_angle_position_for_period_number(&mut self, angle_percent: u8, period_number: u16) {
        let servo_delay_high_us = (angle_percent as u64 * Self::SERVO_MIN_MICRO_SECONDS_SENSITIVITY as u64) + Self::SERVO_MOTOR_MIN_PULSE_WIDTH as u64;
        let servo_delay_low_us = Self::SERVO_MOTOR_PULSE_PERIOD as u64 - servo_delay_high_us;
        for _ in 0..period_number {
            self.gpio_pin.set_high();
            thread::sleep(Duration::from_micros(servo_delay_high_us));
            self.gpio_pin.set_low();
            thread::sleep(Duration::from_micros(servo_delay_low_us));
        }
    }
}