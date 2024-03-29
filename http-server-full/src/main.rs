use anyhow::Result;
use core::str;
use embedded_svc::{
    http::{Headers, Method},
    io::{Read, Write},
};

use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::{
        i2c::{I2cConfig, I2cDriver},
        prelude::*,
    },
    http::server::{EspHttpServer},
};
use shtcx::{self, shtc3, PowerMode};
use std::{
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};
use wifi::wifi;

use serde::Deserialize;


#[toml_cfg::toml_config]
pub struct Config {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_psk: &'static str,
}
static INDEX_HTML: &str = include_str!("http_server_page.html");
// Max payload length
const MAX_LEN: usize = 128;

// Need lots of stack to parse JSON
const STACK_SIZE: usize = 10240;

#[derive(Deserialize)]
struct FormData<'a> {
    command_to_run: &'a str,
    show_temperature:&'a str,
    connection_type: &'a str,
}


fn main() -> Result<()> {
    esp_idf_svc::sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    // The constant `CONFIG` is auto-generated by `toml_config`.
    let app_config = CONFIG;

    // Connect to the Wi-Fi network
    let _wifi = wifi(
        app_config.wifi_ssid,
        app_config.wifi_psk,
        peripherals.modem,
        sysloop,
    )?;

    // Initialize temperature sensor
    let sda = peripherals.pins.gpio10;
    let scl = peripherals.pins.gpio8;
    let i2c = peripherals.i2c0;
    let config = I2cConfig::new().baudrate(100.kHz().into());
    let i2c = I2cDriver::new(i2c, sda, scl, &config)?;
    let temp_sensor_main = Arc::new(Mutex::new(shtc3(i2c)));
    let temp_sensor = temp_sensor_main.clone();
    let temp_sensor_for_get = temp_sensor.clone();
    temp_sensor
        .lock()
        .unwrap()
        .start_measurement(PowerMode::NormalMode)
        .unwrap();

    // Set the HTTP server
    let server_configuration = esp_idf_svc::http::server::Configuration {
        stack_size: STACK_SIZE,
        ..Default::default()
    };
    //let mut server = EspHttpServer::new(&Configuration::default())?;
    let mut server = EspHttpServer::new(&server_configuration)?;
    
    server.fn_handler("/", Method::Get, |request| {
          let mut response = request.into_ok_response()?;
        response.write_all(INDEX_HTML.as_bytes())?;
        Ok(())
    })?;
    

    // http://<sta ip>/temperature handler
    server.fn_handler("/temperature", Method::Get, move |request| {
        let temp_val = temp_sensor
            .lock()
            .unwrap()
            .get_measurement_result()
            .unwrap()
            .temperature
            .as_degrees_celsius();
        let html = temperature(temp_val);
        let mut response = request.into_ok_response()?;
        response.write_all(html.as_bytes())?;
        Ok(())
    })?;

server.fn_handler("/post", Method::Post, |mut req| {
    let len = req.content_len().unwrap_or(0) as usize;

    if len > MAX_LEN {
        req.into_status_response(413)?
            .write_all("Request too big".as_bytes())?;
        return Ok(());
    }

    let mut buf = vec![0; len];
    req.read_exact(&mut buf)?;
    let mut resp = req.into_ok_response()?;

    if let Ok(form) = serde_json::from_slice::<FormData>(&buf) {
        let temp_val = temp_sensor_for_get
            .lock()
            .unwrap()
            .get_measurement_result()
            .unwrap()
            .temperature
            .as_degrees_celsius();


        write!(
            resp,
            "You sent, command_to_run: {}, show_temperature {} connection_type {}! The temp is {:.2}°C.",
            form.command_to_run, form.show_temperature, form.connection_type, temp_val
        )?;
    } else {
    
        let buf_str = std::str::from_utf8(&buf).unwrap_or("<invalid UTF-8>");
        let error_message = format!("JSON error. Buffer content: {}", buf_str);
        resp.write_all(error_message.as_bytes())?;        
    }

    Ok(())
})?;


    println!("Server awaiting connection");

    // Prevent program from exiting
    loop {
        temp_sensor_main
            .lock()
            .unwrap()
            .start_measurement(PowerMode::NormalMode)
            .unwrap();
        sleep(Duration::from_millis(1000));
    }
}

fn templated(content: impl AsRef<str>) -> String {
    format!(
        r#"
<!DOCTYPE html>
<html>
    <head>
        <meta charset="utf-8">
        <title>esp-rs web server</title>
    </head>
    <body>
        {}
    </body>
</html>
"#,
        content.as_ref()
    )
}

fn temperature(val: f32) -> String {
    templated(format!("Chip temperature: {:.2}°C", val))
}

