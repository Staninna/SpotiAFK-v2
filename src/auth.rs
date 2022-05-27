// Imports
use rspotify::{
    prelude::*, scopes, AuthCodeSpotify, Config, Credentials, OAuth, DEFAULT_API_PREFIX,
    DEFAULT_CACHE_PATH, DEFAULT_PAGINATION_CHUNKS,
};
use std::{env, path};
use urlshortener::{client::UrlShortener, providers::Provider};

// Functions

// Auth to spotify API
pub async fn auth_client() -> Result<rspotify::AuthCodeSpotify, String> {
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
