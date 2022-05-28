use shells::bash;
use std::{env, fs::File, io::prelude::*, path::Path};

// Check spotifyd settings
pub fn init_spotifyd() -> Result<(), String> {
    let mut found_settings = 0;
    for (key, _) in env::vars() {
        match key.as_str() {
            "SPOTIFYD_USERNAME" => found_settings += 1,
            "SPOTIFYD_PASSWORD" => found_settings += 1,
            "SPOTIFYD_DEVICE_NAME" => found_settings += 1,
            "SPOTIFYD_CONFIG_PATH" => found_settings += 1,
            _ => {}
        }
    }
    match found_settings {
        4 => {}
        _ => return Err(String::from("Failed parsing spotifyd settings")),
    }

    if !Path::new(env::var("SPOTIFYD_CONFIG_PATH").unwrap().as_str()).exists() {
        match make_config() {
            Ok(_) => (),
            Err(_) => return Err(String::from("Failed to make spotifyd config file")),
        }
    }

    start_spotifyd();

    Ok(())
}

fn make_config() -> std::io::Result<()> {
    let mut config_file = File::create(env::var("SPOTIFYD_CONFIG_PATH").unwrap().as_str())?;

    config_file.write_all(b"[global]\n")?;
    config_file.write_all(
        &[
            b"username = \"",
            env::var("SPOTIFYD_USERNAME").unwrap().as_bytes(),
            b"\"\n",
        ]
        .concat(),
    )?;
    config_file.write_all(
        &[
            b"password = \"",
            env::var("SPOTIFYD_PASSWORD").unwrap().as_bytes(),
            b"\"\n",
        ]
        .concat(),
    )?;
    config_file.write_all(
        &[
            b"device_name = \"",
            env::var("SPOTIFYD_DEVICE_NAME").unwrap().as_bytes(),
            b"\"\n",
        ]
        .concat(),
    )?;

    Ok(())
}

pub fn start_spotifyd() {
    bash!(
        "spotifyd --config-path {}",
        env::var("SPOTIFYD_CONFIG_PATH").unwrap().as_str()
    );
}

pub fn _stop_spotifyd() {
    bash!(
        "kill $(pgrep spotifyd --config-path {})",
        env::var("SPOTIFYD_CONFIG_PATH").unwrap().as_str()
    );
}
