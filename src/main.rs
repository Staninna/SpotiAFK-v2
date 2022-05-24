/////////////
// Imports //
/////////////
use dotenv::from_filename;
use futures::executor::block_on;
use open;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use std::{env, io, process};
use urlshortener::{client::UrlShortener, providers::Provider};

//////////////////////
// Useful functions //
//////////////////////

// Gets users input
fn input(prompt: &str) -> String {
    // Prompt
    print!("{}", prompt);
    io::Write::flush(&mut io::stdout()).expect("flush failed!");

    // User input
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("error: unable to read user input");

    // Return input
    String::from(input.trim())
}

//////////////////
// Real program //
//////////////////

// Real entry point
async fn real_main() -> Result<i32, i32> {
    // Get config variables
    from_filename(".env").ok();

    // First authorization
    let client = match init_client().await {
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
async fn init_client() -> Option<rspotify::AuthCodeSpotify> {
    // Used scopes
    let scopes = scopes!("user-read-currently-playing");

    // initialization of client
    let credentials = Credentials::from_env().unwrap();
    let oauth = OAuth::from_env(scopes).unwrap();
    let mut client = AuthCodeSpotify::new(credentials, oauth);

    // Get authorize url
    let url = get_authorize_url(&client);

    // Print messages to user
    println!(
        "A webpage should open log into your spotify account and paste the redirected url here",
    );
    println!(
        "If no webpage opened, please open this url in your browser:\n\x1b[32m{}\x1b[0m",
        url.trim()
    );

    // Let user login
    let redirect = input("Paste the redirected URL: ");

    // Open url in default browser without shortener
    open::that(url).unwrap();

    // Get token
    let response = client.parse_response_code(&redirect);
    match response {
        #[allow(unused_must_use)]
        Some(code) => {
            client.request_token(code.as_str());
            client.read_token_cache(false);
            return Some(client);
        }
        None => None,
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
fn main() {
    // Run application and match on exit codes
    process::exit(match block_on(real_main()) {
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
