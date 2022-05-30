use std::{env, fs::File, io::prelude::*, path::Path, process::Command};

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

    match start_spotifyd() {
        Ok(_) => (),
        Err(e) => match e.as_str() {
            "Failed to start spotifyd" => return Err(String::from("Failed to start spotifyd")),
            _ => return Err(String::from("Unexpected exit_code")),
        },
    };

    Ok(())
}

// TODO maybe make more settings by matching on SPOTIFYD_{SETTING_NAME} from .env file
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

    // TODO add to .env because they are needed otherwise spotifyd crashes
    config_file.write_all(b"backend = \"pulseaudio\"\n")?;
    config_file.write_all(b"volume_controller = \"alsa\"\n")?;

    Ok(())
}

pub fn start_spotifyd() -> Result<(), String> {
    match Command::new("spotifyd")
        .args([
            "--config-path",
            env::var("SPOTIFYD_CONFIG_PATH").unwrap().as_str(),
        ])
        .spawn()
    {
        Ok(_) => Ok(()),
        Err(_) => Err(String::from("Failed to start spotifyd")),
    }
}

pub fn stop_spotifyd() -> Result<(), String> {
    let pids = match Command::new("pgrep")
        .args([
            "-f",
            format!(
                "spotifyd --config-path {}",
                env::var("SPOTIFYD_CONFIG_PATH").unwrap()
            )
            .as_str(),
        ])
        .output()
    {
        Ok(output) => String::from_utf8(output.stdout).unwrap(),
        Err(_) => return Err(String::from("Failed getting pid of spotifyd")),
    };
    for pid in pids.lines() {
        let kill = Command::new("kill").arg(pid).spawn();
        match kill {
            Ok(_) => (),
            Err(_) => return Err(String::from("Failed stopping spotifyd")),
        }
    }
    Ok(())
}
