/////////////
// Imports //
/////////////

// Extern imports
use online::sync::check;
use rspotify::{model::SimplifiedPlaylist, prelude::*};
use std::env;

// Get if connected to internet
pub fn online() -> bool {
    match check(None) {
        Ok(_) => true,
        Err(_) => false,
    }
}

// Get playlist to play
pub async fn get_playlist(
    client: &rspotify::AuthCodeSpotify,
) -> Result<SimplifiedPlaylist, String> {
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

// Can i play?
pub async fn is_playing(client: &rspotify::AuthCodeSpotify) -> Result<bool, String> {
    match online() {
        true => {
            let is_playing = match client.current_user_playing_item().await.unwrap() {
                Some(playing) => playing.is_playing,
                None => false,
            };

            match is_playing {
                true => {
                    let devices = client.device().await.unwrap();
                    if devices.len() == 0 {
                        return Err(String::from("Failed finding devices"));
                    }
                    for device in devices {
                        if device.name == env::var("SPOTIFYD_DEVICE_NAME").unwrap() {
                            if device.is_active {
                                return Ok(true);
                            } else {
                                return Ok(false);
                            }
                        } else {
                            if device.is_active {
                                return Ok(false);
                            } else {
                            }
                        }
                    }
                    Ok(false)
                }
                false => return Ok(true),
            }
        }
        false => Err(String::from("Failed to connect to the internet")),
    }
}
