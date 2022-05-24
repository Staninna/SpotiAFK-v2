/////////////
// Imports //
/////////////
use dotenv;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use std::{env, process};
use urlshortener::{client::UrlShortener, providers::Provider};

//////////////////
// Real program //
//////////////////

// Real entry point
async fn real_main() -> Result<i32, i32> {
    // Get config variables
    dotenv::from_filename(".env").ok();

    // First authorization
    let client = match init_client("user-read-currently-playing").await {
        Some(client) => client,
        None => {
            return Err(1); // Authorization failed
        }
    };
    Ok(0) // Program finished successfully
}

/////////////////////////
// Auth to spotify API //
/////////////////////////
async fn init_client(scopes: &str) -> Option<rspotify::AuthCodeSpotify> {
    // initialization of client
    let credentials = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes!(scopes)).unwrap();
    let mut client = AuthCodeSpotify::new(credentials, oauth);

    // Get authorize url
    let url = get_authorize_url(&client);

    // Let user login
    match client.prompt_for_token(&url).await {
        Ok(_) => Some(client),
        Err(_) => None,
    }
}

////////////////////////////////
// Get url to open in browser //
////////////////////////////////
fn get_authorize_url(client: &rspotify::AuthCodeSpotify) -> String {
    // Check if bitly api key is provided
    let bitly_key = env::var("BITLY_API_TOKEN");
    let bitly = match bitly_key {
        Ok(_) => true,
        Err(_) => false,
    };

    // Generate urls with or without bitly shortener
    let url = if bitly {
        let short_url = UrlShortener::new().unwrap().generate(
            &client.get_authorize_url(true).unwrap(),
            &Provider::BitLy {
                token: env::var("BITLY_API_TOKEN").unwrap().to_owned(),
            },
        );
        match short_url {
            // If shortener successful return short url
            Ok(_) => {
                assert!(short_url.is_ok());
                return short_url.unwrap();
            }
            // If failed return long url
            Err(_) => return client.get_authorize_url(true).unwrap(),
        }
    } else {
        // If no bitly api key is provided return
        client.get_authorize_url(true).unwrap()
    };

    // Return url
    url
}

/////////////////
// Entry point //
/////////////////
#[tokio::main]
async fn main() {
    // Run application and match on exit codes
    process::exit(match real_main().await {
        Ok(0) => {
            println!("Program finished successfully");
            0
        }
        Err(1) => {
            println!("Authorization failed please try again");
            1
        }
        _ => {
            println!("Unexpected exit_code");
            -1
        }
    });
}

// TODO refresh token https://github.com/ramsayleung/rspotify/blob/master/examples/with_refresh_token.rs
