use std::env;

// Check spotifyd settings
pub fn init_spotifyd() -> Result<(), String> {
    let mut found_settings = 0;
    for (key, _) in env::vars() {
        match key.as_str() {
            "SPOTIFYD_USERNAME" => found_settings += 1,
            "SPOTIFYD_PASSWORD" => found_settings += 1,
            "SPOTIFYD_DEVICE_NAME" => found_settings += 1,
            "SPOTIFYD_CONFIG_PATH" => found_settings += 1,
            _ => (),
        }
    }
    match found_settings {
        4 => Ok(()),
        _ => return Err(String::from("Failed parsing spotifyd settings")),
    }
}

// TODO make config file
