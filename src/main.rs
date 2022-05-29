//////////
// DOCS // https://docs.rs/rspotify/latest/rspotify
//////////

/////////////
// Imports //
/////////////

// Extern imports
use dotenv;
use online::sync::check;
use rspotify::{
    model::{AdditionalType, Country, Market, SimplifiedPlaylist},
    prelude::*,
};
use std::{env, process::exit};

// Self made files
mod auth;
mod spotifyd;
use auth::*;
use spotifyd::*;

///////////////
// Functions //
///////////////

// Useful functions

// Get if connected to internet
fn online() -> bool {
    match check(None) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// Print type of variable
// https://stackoverflow.com/questions/21747136/how-do-i-print-the-type-of-a-variable#answer-58119924
#[allow(dead_code)] // TODO Used for debugging
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

// Api interaction

// Get playlists
async fn get_playlists(
    client: &rspotify::AuthCodeSpotify,
) -> Result<Vec<SimplifiedPlaylist>, String> {
    // Make buffer variables
    let offset = client.config.pagination_chunks;
    let mut playlists = Vec::new();

    // Get all playlists in a vector
    let mut index = 0;
    loop {
        // Request next playlists
        let response = match online() {
            true => client
                .current_user_playlists_manual(Some(offset), Some(offset * index))
                .await
                .unwrap(),
            false => return Err(String::from("Failed to connect to the internet")),
        };

        // Put received playlists in vector
        for playlist in response.items {
            playlists.push(playlist)
        }

        // If none playlist are left break
        if response.next == None {
            break;
        }

        index += 1;
    }
    Ok(playlists)
}

// Program functions

// Get playlist to play
async fn get_playlist(client: &rspotify::AuthCodeSpotify) -> Result<SimplifiedPlaylist, String> {
    // Get playlist to play
    match online() {
        true => {
            let playlists = match get_playlists(&client).await {
                Ok(playlists) => playlists,
                Err(_) => return Err(String::from("Failed to connect to the internet")),
            };
            let mut playlist_found = false;
            let mut afk_playlist = playlists.first().unwrap();
            match env::var("PLAYLIST_NAME") {
                Ok(_) => {
                    for playlist in &playlists {
                        if playlist.name == env::var("PLAYLIST_NAME").unwrap() {
                            afk_playlist = &playlist;
                            playlist_found = true;
                            break;
                        }
                    }
                }
                Err(_) => return Err(String::from("Failed parsing playing settings")),
            }
            match playlist_found {
                true => (),
                false => return Err(String::from("Failed parsing playing settings")),
            }
            Ok(afk_playlist.clone())
        }
        false => Err(String::from("Failed to connect to the internet")),
    }
}

// Can i play?
async fn _is_playing(
    // TODO
    client: &rspotify::AuthCodeSpotify,
    user_market: &Market,
    content_types: &[AdditionalType; 2],
) -> Result<bool, String> {
    match online() {
        true => {
            let playing = client
                .current_playing(Some(user_market), Some(content_types))
                .await
                .unwrap();
            print_type_of(&playing);
            match playing {
                None => Ok(true),
                Some(_) => Ok(false),
            }
        }
        false => Err(String::from("Failed to connect to the internet")),
    }
}

/////////////
// Program //
/////////////

// Real entry point
async fn real_main() -> Result<String, String> {
    // Check for internet connection
    if !online() {
        return Err(String::from("Failed to connect to the internet"));
    }

    // Get config variables
    dotenv::from_filename(".env").ok();

    // Check spotifyd
    match init_spotifyd() {
        Ok(_) => (),
        Err(e) => match e.as_str() {
            "Failed parsing spotifyd settings" => {
                return Err(String::from("Failed parsing spotifyd settings"))
            }
            "Failed to make spotifyd config file" => {
                return Err(String::from("Failed to make spotifyd config file"))
            }
            "Failed to start spotifyd" => return Err(String::from("Failed to start spotifyd")),
            _ => return Err(String::from("Unexpected exit_code")),
        },
    };

    // First authorization and checks if everything works
    let client = match auth_client().await {
        Ok(client) => client,
        Err(e) => match e.as_str() {
            "Authorization failed" => return Err(String::from("Authorization failed")),
            "Failed parsing spotify client" => {
                return Err(String::from("Failed parsing spotify client"))
            }
            "Failed parsing spotify api" => return Err(String::from("Failed parsing spotify api")),
            _ => return Err(String::from("Unexpected exit_code")),
        },
    };

    // Getting data of current user
    let mut user_country = Country::Netherlands;
    #[allow(unused_assignments)]
    let mut _user_market = Market::Country(user_country.clone());
    let _content_types = [AdditionalType::Track, AdditionalType::Episode];
    match online() {
        true => {
            match client.me().await {
                Ok(me) => {
                    user_country = me.country.unwrap();
                    _user_market = Market::Country(user_country.clone());
                }
                // Check client prefix is correct in .env
                Err(_) => return Err(String::from("Failed parsing spotify client")),
            }
        }
        false => return Err(String::from("Failed to connect to the internet")),
    }

    // Get playlist to play
    let _playlist = match get_playlist(&client).await {
        Ok(playlist) => playlist,
        Err(e) => match e.as_str() {
            "Failed parsing playing settings" => {
                return Err(String::from("Failed parsing playing settings"))
            }
            "Failed to connect to the internet" => {
                return Err(String::from("Failed to connect to the internet"))
            }
            _ => return Err(String::from("Unexpected exit_code")),
        },
    };

    // End of program
    match stop_spotifyd() {
        Ok(_) => (),
        Err(e) => match e.as_str() {
            "Failed getting pid of spotifyd" => {
                return Err(String::from("Failed getting pid of spotifyd"))
            }
            "Failed stopping spotifyd" => return Err(String::from("Failed stopping spotifyd")),
            _ => return Err(String::from("Unexpected exit_code")),
        },
    }
    Ok(String::from("Program finished successfully"))
}

// Entry point
#[tokio::main]
async fn main() {
    dotenv::from_filename(".env").ok();
    // Run application and match on exit codes
    exit(match real_main().await.unwrap().as_str() {
        "Program finished successfully" => {
            println!("Program finished successfully");
            0
        }
        "Authorization failed" => {
            println!("Authorization failed; please try again");
            1
        }
        "Failed parsing spotify client" => {
            println!("Failed parsing spotify client. Please check your .env file");
            1
        }
        "Failed to connect to the internet" => {
            println!("Failed to connect to the internet, please check your connection");
            1
        }
        "Failed parsing playing settings" => {
            println!("Failed parsing playing settings. Please check your .env file");
            1
        }
        "Failed parsing spotify api" => {
            println!("Failed parsing spotify api, Please check your .env file");
            1
        }
        "Failed parsing spotifyd settings" => {
            println!("Failed parsing spotifyd settings, Please check your .env file");
            1
        }
        "Failed to make spotifyd config file" => {
            println!("Failed to make spotifyd config file, please try again");
            1
        }
        "Failed to start spotifyd" => {
            println!(
                "Failed to start spotifyd, make sure spotifyd is installed and added to your PATH"
            );
            1
        }
        "Failed getting pid of spotifyd" => {
            println!("Failed getting pid of spotifyd, make sure pgrep is installed");
            1
        }
        "Failed stopping spotifyd" => {
            println!("Failed stopping spotifyd, make sure kill is installed");
            1
        }
        _ => {
            println!("Unexpected exit_code");
            -1
        }
    });
}
