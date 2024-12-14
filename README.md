# Camera driver

A web interface to control raspberry pi camera's servo motors and I2C HYT 221 sensors.

***

![Screenshot of the web interface](images/web_interface.png)

## How to use ?

### Retrieve components parameters:

You will need to install `i2c-tools` and wiringpi (https://github.com/WiringPi/WiringPi/releases).

First get the I2C HYT 221 sensor address (in hexadecimal format):

    $ i2cdetect -y 1

Then Retrieve the **BCM** GPIO pin of the servo motors:

    $ gpio readall


### Create the configuration file:

Create a **config.toml** file with the following content:

```
camera_binding_network_port = [Binding address and port, ie: "0.0.0.0:8000"]
hyt221_i2c_address = [HYT221 I2C address]
vertical_servo_motor_gpio_pin = [Vertical servo motor GPIO pin]
horizontal_servo_motor_gpio_pin = [Horizontal servo motor GPIO pin]
vertical_servo_motor_angle_percent_max = [Vertical servo motor GPIO max position (in %)]
horizontal_servo_motor_angle_percent_max = [Horizontal servo motor GPIO max position (in %)]
vertical_servo_motor_angle_percent_min = [Vertical servo motor GPIO min position (in %)]
horizontal_servo_motor_angle_percent_min = [Horizontal servo motor GPIO min position (in %)]
is_horizontal_servo_motor_inverted = [true or false]
is_vertical_servo_motor_inverted = [true or false]
vertical_servo_motor_initial_angle = [Vertical servo motor initial angle (in %)]
horizontal_servo_motor_initial_angle = [Horizontal servo motor initial angle (in %)]
html_file_path = [Frontend HTML file path (you can use the default one in frontend/index.html git directory)]
ip_stream_url = [MJPEG stream URL, ie: "http://192.168.1.231:7777/video/" (notice that "//localhost" will be replaced by "//[current ip address]")]
```


### Start the server:

    $ ./camera_driver config.toml

Or if the listening port is below 1024:

    $ sudo ./camera_driver config.toml
