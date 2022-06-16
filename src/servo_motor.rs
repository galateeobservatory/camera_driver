use anyhow::{anyhow, Result};
use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Invalid angle (expected between {min:?} and {max:?}, got {angle:?}")]
    InvalidAngle { min: u8, max: u8, angle: u8 },
    #[error("Failed to create GPIO object")]
    GpioCreationError,
    #[error("Failed to get GPIO pin {pin_number:?}")]
    GpioPinError { pin_number: u8 },
    #[error("min_angle_percent {min_angle_percent:?} must be less than max_angle_percent {max_angle_percent:?}")]
    MinMaxAngleError {
        min_angle_percent: u8,
        max_angle_percent: u8,
    },
    #[error("min_angle_percent {min_angle_percent:?} and max_angle_percent {max_angle_percent:?} must be between {min_allowed_angle:?} and {max_allowed_angle:?}")]
    MinMaxAngleRangeError {
        min_angle_percent: u8,
        max_angle_percent: u8,
        min_allowed_angle: u8,
        max_allowed_angle: u8,
    },
}

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
    const PERIOD_NUMBER_FOR_KNOWN_PREVIOUS_ANGLE: u16 = 50;
    const SERVO_MIN_MICRO_SECONDS_SENSITIVITY: u16 = 20;

    pub fn new(
        pin_number: u8,
        min_angle_percent: u8,
        max_angle_percent: u8,
    ) -> Result<Self, anyhow::Error> {
        if min_angle_percent > max_angle_percent {
            return Err(anyhow!(
                "min_angle_percent {} must be less than max_angle_percent {}",
                min_angle_percent,
                max_angle_percent
            ));
        }
        if min_angle_percent < Self::SERVO_MOTOR_MIN_ANGLE_PERCENT
            || max_angle_percent > Self::SERVO_MOTOR_MAX_ANGLE_PERCENT
        {
            return Err(anyhow!(
                "min_angle_percent {} and max_angle_percent {} must be between {} and {}",
                min_angle_percent,
                max_angle_percent,
                Self::SERVO_MOTOR_MIN_ANGLE_PERCENT,
                Self::SERVO_MOTOR_MAX_ANGLE_PERCENT
            ));
        }
        Ok(ServoMotor {
            gpio_pin: Gpio::new()
                .map_err(|_| Error::GpioCreationError)?
                .get(pin_number)
                .map_err(|_| anyhow!("Failed to get GPIO pin {}", pin_number))?
                .into_output(),
            pin_number,
            min_angle_percent,
            max_angle_percent,
            current_angle_percent: None,
        })
    }

    pub fn move_to_angle_percent(&mut self, angle_percent: u8) -> Result<(), anyhow::Error> {
        if !(self.min_angle_percent..self.max_angle_percent).contains(&angle_percent) {
            return Err(anyhow!(
                "Invalid angle (expected between {} and {}, got {}",
                self.min_angle_percent,
                self.max_angle_percent,
                angle_percent
            ));
        }
        match self.current_angle_percent {
            Some(_) => {
                self.send_angle_position_for_period_number(
                    angle_percent,
                    Self::PERIOD_NUMBER_FOR_KNOWN_PREVIOUS_ANGLE,
                );
                self.current_angle_percent = Some(angle_percent);
            }
            None => {
                self.send_angle_position_for_period_number(
                    angle_percent,
                    Self::PERIOD_NUMBER_FOR_UNKNOWN_PREVIOUS_ANGLE,
                );
                self.current_angle_percent = Some(angle_percent);
            }
        }
        return Ok(());
    }

    fn send_angle_position_for_period_number(&mut self, angle_percent: u8, period_number: u16) {
        let servo_delay_high_us = (angle_percent as u64
            * Self::SERVO_MIN_MICRO_SECONDS_SENSITIVITY as u64)
            + Self::SERVO_MOTOR_MIN_PULSE_WIDTH as u64;
        let servo_delay_low_us = Self::SERVO_MOTOR_PULSE_PERIOD as u64 - servo_delay_high_us;
        for _ in 0..period_number {
            self.gpio_pin.set_high();
            thread::sleep(Duration::from_micros(servo_delay_high_us));
            self.gpio_pin.set_low();
            thread::sleep(Duration::from_micros(servo_delay_low_us));
        }
    }
}
