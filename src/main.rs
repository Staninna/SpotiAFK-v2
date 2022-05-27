/////////////
// Imports //
/////////////
use dotenv;
use online::sync::check;
use rspotify::{
    model::{AdditionalType, Country, Market, SimplifiedPlaylist},
    prelude::*,
    scopes, AuthCodeSpotify, Config, Credentials, OAuth, DEFAULT_API_PREFIX, DEFAULT_CACHE_PATH,
    DEFAULT_PAGINATION_CHUNKS,
};
use std::{env, path, process};
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

// Get if connected to internet
fn online() -> bool {
    match check(None) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// Print type of variable
// https://stackoverflow.com/questions/21747136/how-do-i-print-the-type-of-a-variable#answer-58119924
#[allow(dead_code)] // Used for debugging
fn print_type_of<T>(_: &T) {
    println!("{}", std::any::type_name::<T>())
}

// Authentication

// Auth to spotify API
async fn auth_client() -> Result<rspotify::AuthCodeSpotify, String> {
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
        return Err(String::from("Failed parsing spotify api"));
    }

    // Api scopes
    let scopes = scopes!(
        "user-modify-playback-state",
        "playlist-read-private",
        "user-read-playback-state"
    );

    // initialization of client
    let config = match parse_client_config() {
        Ok(config) => config,
        Err(_) => return Err(String::from("Failed parsing spotify client")),
    };
    let credentials = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes).unwrap();
    let mut client = AuthCodeSpotify::with_config(credentials, oauth, config);

    // Get authorize url
    let url = get_authorize_url(&client);

    // Let user login
    match client.prompt_for_token(&url).await {
        Ok(_) => Ok(client),
        Err(_) => Err(String::from("Authorization failed")),
    }
}

// Parse spotify client settings
fn parse_client_config() -> Result<Config, String> {
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
        true => Err(String::from("Failed parsing spotify client")),
        false => match found_settings {
            5 => Ok(Config {
                prefix: rspotify_client_prefix,
                cache_path: rspotify_client_cache_path,
                pagination_chunks: rspotify_client_pagination_chunks,
                token_cached: rspotify_client_token_cached,
                token_refreshing: rspotify_client_token_refreshing,
            }),
            _ => Err(String::from("Failed parsing spotify client")),
        },
    }
}

// Get url to open in browser
fn get_authorize_url(client: &rspotify::AuthCodeSpotify) -> String {
    // Get destination url
    let long_url = client.get_authorize_url(true).unwrap();
    #[allow(unused_assignments)] // Compiler complains
    let mut url = String::new();

    // Check if in WM/DE to spare bitly links you only have 100
    match env::var("DISPLAY") {
        Ok(_) => {
            // Check if bitly api key is provided
            let bitly_key = env::var("BITLY_API_TOKEN");
            let bitly = match bitly_key {
                Ok(_) => true,
                Err(_) => false,
            };
            // Generate urls with or without bitly shortener
            url = if bitly {
                let short_url = UrlShortener::new().unwrap().generate(
                    &long_url,
                    &Provider::BitLy {
                        token: bitly_key.unwrap().to_owned(),
                    },
                );
                match short_url {
                    // If shortener successful return short url
                    Ok(short_url) => match short_url.as_str() {
                        "INVALID_ARG_ACCESS_TOKEN" | "MONTHLY_RATE_LIMIT_EXCEEDED" => {
                            return long_url
                        }
                        _ => {
                            return short_url;
                        }
                    },
                    // If failed return long url
                    Err(_) => return long_url,
                }
            } else {
                // If no bitly api key is provided return
                long_url
            };
        }
        Err(_) => url = long_url,
    }

    // Return url
    url
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
async fn is_playing(
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
    #[allow(unused_assignments)] // For some reason the compiler complains
    let mut user_market = Market::Country(user_country.clone());
    let content_types = [AdditionalType::Track, AdditionalType::Episode];
    match online() {
        true => {
            match client.me().await {
                Ok(me) => {
                    user_country = me.country.unwrap();
                    user_market = Market::Country(user_country.clone());
                }
                // Check client prefix is correct in .env
                Err(_) => return Err(String::from("Failed parsing spotify client")),
            }
        }
        false => return Err(String::from("Failed to connect to the internet")),
    }

    // Get playlist to play
    let playlist = match get_playlist(&client).await {
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
    Ok(String::from("Program finished successfully"))
}

// Entry point
#[tokio::main]
async fn main() {
    // Run application and match on exit codes
    process::exit(match real_main().await.unwrap().as_str() {
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
            2
        }
        "Failed to connect to the internet" => {
            println!("Failed to connect to the internet, please check your connection");
            3
        }
        "Failed parsing playing settings" => {
            println!("Failed parsing playing settings. Please check your .env file");
            4
        }
        "Failed parsing spotify api" => {
            println!("Failed parsing spotify api, Please check your .env file");
            5
        }
        _ => {
            println!("Unexpected exit_code");
            -1
        }
    });
}
