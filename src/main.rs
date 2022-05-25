/////////////
// Imports //
/////////////
use dotenv;
use online::sync::check;
use rspotify::{
    model::SimplifiedPlaylist, prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth,
    DEFAULT_API_PREFIX, DEFAULT_CACHE_PATH, DEFAULT_PAGINATION_CHUNKS,
};
use std::{env, mem, path, process};
use urlshortener::{client::UrlShortener, providers::Provider};

///////////////
// Functions //
///////////////

// Useful functions

// Get first variable of vector
// https://stackoverflow.com/questions/36876570/return-first-item-of-vector#answer-36876741
fn first<T>(v: &Vec<T>) -> Option<&T> {
    v.first()
}

// Authentication

// Auth to spotify API
async fn auth_client() -> Result<rspotify::AuthCodeSpotify, i32> {
    let mut found_variables = 0;
    // Check if environment variable are present
    for (key, _) in env::vars() {
        match key.as_str() {
            "RSPOTIFY_CLIENT_ID" => found_variables += 1,
            "RSPOTIFY_CLIENT_SECRET" => found_variables += 1,
            "RSPOTIFY_REDIRECT_URI" => found_variables += 1,
            _ => (),
        }
    }
    if found_variables != 3 {
        return Err(5); // Failed parsing spotify api
    }

    // Api scopes
    let scopes = scopes!(
        "user-modify-playback-state",
        "playlist-read-private",
        "user-read-playback-state"
    );

    // initialization of client
    let config = match parse_client_config() {
        Some(config) => config,
        None => return Err(2), // Failed parsing spotify client
    };
    let credentials = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes).unwrap();
    let mut client = AuthCodeSpotify::with_config(credentials, oauth, config);

    // Get authorize url
    let url = get_authorize_url(&client);

    // Let user login
    // TODO add check for internet connection
    match client.prompt_for_token(&url).await {
        Ok(_) => Ok(client),
        Err(_) => Err(1), // Authorization failed
    }
}

// Parse spotify client settings
fn parse_client_config() -> Option<Config> {
    // Make buffer variables
    let mut rspotify_client_prefix = String::new();
    let mut rspotify_client_cache_path = path::PathBuf::new();
    let mut rspotify_client_pagination_chunks = 50;
    let mut rspotify_client_token_cached = true;
    let mut rspotify_client_token_refreshing = true;
    let mut wrong_config = false;
    let mut found_settings = 0;

    // Loop over all environment variables
    for (key, value) in env::vars() {
        match key.as_str() {
            // Set client prefix from .env
            "RSPOTIFY_CLIENT_PREFIX" => {
                match value.as_str() {
                    "default" => rspotify_client_prefix = String::from(DEFAULT_API_PREFIX),
                    _ => rspotify_client_prefix = value,
                };
                found_settings += 1;
            }

            // Set client cache path from .env
            "RSPOTIFY_CLIENT_CACHE_PATH" => {
                match value.as_str() {
                    "default" => {
                        rspotify_client_cache_path = path::PathBuf::from(DEFAULT_CACHE_PATH)
                    }
                    _ => rspotify_client_cache_path = path::PathBuf::from(value),
                };
                found_settings += 1
            }

            // Check client pagination chunks if correct in .env
            "RSPOTIFY_CLIENT_PAGINATION_CHUNKS" => {
                match value.as_str() {
                    "default" => rspotify_client_pagination_chunks = DEFAULT_PAGINATION_CHUNKS,
                    _ => {
                        let pagination_chunks: u32 = value.parse().unwrap();
                        if pagination_chunks <= 50 {
                            rspotify_client_pagination_chunks = pagination_chunks
                        } else {
                            wrong_config = true
                        }
                    }
                };
                found_settings += 1
            }

            // Check client cached if correct in .env
            "RSPOTIFY_CLIENT_TOKEN_CACHED" => {
                match value.as_str() {
                    "true" => rspotify_client_token_cached = true,
                    "false" => rspotify_client_token_cached = false,
                    _ => wrong_config = true,
                };
                found_settings += 1
            }

            // Check client refresh if correct in .env
            "RSPOTIFY_CLIENT_TOKEN_REFRESHING" => {
                match value.as_str() {
                    "true" => rspotify_client_token_refreshing = true,
                    "false" => rspotify_client_token_refreshing = false,
                    _ => wrong_config = true,
                };
                found_settings += 1
            }
            _ => (),
        }
    }

    // Check if there was a fail in the config
    match wrong_config {
        true => None,
        false => match found_settings {
            5 => Some(Config {
                prefix: rspotify_client_prefix,
                cache_path: rspotify_client_cache_path,
                pagination_chunks: rspotify_client_pagination_chunks,
                token_cached: rspotify_client_token_cached,
                token_refreshing: rspotify_client_token_refreshing,
            }),
            _ => None,
        },
    }
}

// Get url to open in browser
fn get_authorize_url(client: &rspotify::AuthCodeSpotify) -> String {
    // Check if bitly api key is provided
    let long_url = client.get_authorize_url(true).unwrap();
    let bitly_key = env::var("BITLY_API_TOKEN");
    let bitly = match bitly_key {
        Ok(_) => true,
        Err(_) => false,
    };

    // Generate urls with or without bitly shortener
    let url = if bitly {
        let short_url = UrlShortener::new().unwrap().generate(
            &long_url,
            &Provider::BitLy {
                token: bitly_key.unwrap().to_owned(),
            },
        );
        match short_url {
            // If shortener successful return short url
            Ok(_) => {
                assert!(short_url.is_ok());
                return short_url.unwrap();
            }
            // If failed return long url
            Err(_) => return long_url,
        }
    } else {
        // If no bitly api key is provided return
        long_url
    };

    // Return url
    url
}

// Api interaction

// Get playlists
async fn get_playlists(client: &rspotify::AuthCodeSpotify) -> Vec<SimplifiedPlaylist> {
    // Make buffer variables
    let offset = client.config.pagination_chunks;
    let mut playlists = Vec::new();

    // Get all playlists in a vector
    let mut index = 0;
    loop {
        // Request next playlists
        let response = client
            .current_user_playlists_manual(Some(offset), Some(offset * index))
            .await
            .unwrap();

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
    playlists
}

/////////////
// Program //
/////////////

// Real entry point
async fn real_main() -> Result<i32, i32> {
    // Check for internet connection
    if check(None).is_err() {
        return Err(3); // No internet connection
    }

    // Get config variables
    dotenv::from_filename(".env").ok();

    // First authorization and checks if everything works
    let client = match auth_client().await {
        Ok(client) => client,
        Err(1) => return Err(1),  // Authorization failed
        Err(2) => return Err(2),  // Failed parsing spotify client
        Err(5) => return Err(5),  // Failed parsing spotify api
        Err(_) => return Err(-1), // Unexpected exit_code
    };

    // Check client prefix is correct in .env
    match client.device().await {
        Ok(_) => (),
        Err(_) => return Err(2), // Failed parsing spotify client
    }

    // Get playlist to play
    let playlists = get_playlists(&client).await;
    let mut playlist_found = false;
    let mut afk_playlist = first(&playlists).unwrap();
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
        Err(_) => return Err(4), // Failed parsing playing settings
    }
    match playlist_found {
        true => (),
        false => return Err(4), // Failed parsing playing settings
    }
    let afk_playlist = afk_playlist.clone();
    mem::drop(playlists);

    // End of program
    Ok(0) // Program finished successfully
}

// Entry point
#[tokio::main]
async fn main() {
    // Run application and match on exit codes
    process::exit(match real_main().await {
        Ok(0) => {
            println!("Program finished successfully");
            0
        }
        Err(1) => {
            println!("Authorization failed; please try again");
            1
        }
        Err(2) => {
            println!("Failed parsing spotify client. Please check your .env file");
            2
        }
        Err(3) => {
            println!("Failed to connect to the internet, please check your connection");
            3
        }
        Err(4) => {
            println!("Failed parsing playing settings. Please check your .env file");
            4
        }
        Err(5) => {
            println!("Failed parsing spotify api, Please check your .env file");
            5
        }
        _ => {
            println!("Unexpected exit_code");
            -1
        }
    });
}
