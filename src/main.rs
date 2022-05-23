// Imports
use dotenv::from_filename;
use open;
use rspotify::{prelude::*, scopes, AuthCodeSpotify, Credentials, OAuth};
use std::env;
use std::io;
use urlshortener::{client::UrlShortener, providers::Provider};

// Useful functions
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
    String::from(input.trim())
}

fn main() {
    // Get config variables
    from_filename(".env").ok();

    auth_client();
}

// Auth to spotify API
fn auth_client() {
    // Used scopes
    let scopes = scopes!(
        // "user-read-email",
        // "user-read-private",
        // "user-top-read",
        // "user-read-recently-played",
        // "user-follow-read",
        // "user-library-read",
        // "user-read-currently-playing",
        // "user-read-playback-state",
        // "user-read-playback-position",
        // "playlist-read-collaborative",
        // "playlist-read-private",
        // "user-follow-modify",
        // "user-library-modify",
        // "user-modify-playback-state",
        // "playlist-modify-public",
        // "playlist-modify-private",
        // "ugc-image-upload"
    );

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

    // Open url in default browser without shortener
    open::that(url).unwrap();

    // Let user login
    let redirect = input("Paste the redirected URL: ");

    // Get token
    let response = client.parse_response_code(&redirect);
    match response {
        #[allow(unused_must_use)]
        Some(code) => {
            client.request_token(code.as_str());
            client.read_token_cache(false);
            // TODO get token somehow
        }
        None => println!("Something went wrong. Please try again"),
    }
}

// Get url to open in browser
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
        assert!(short_url.is_ok());
        short_url.unwrap()
    } else {
        client.get_authorize_url(true).unwrap()
    };
    url
}
