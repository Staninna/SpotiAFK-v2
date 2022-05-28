# Configuration

Rules

- Anything has te be a string (Between double quotes `"EXAMPLE"`)

Required
Spotify API
| Options                | Default                          | Info                                                             |
|------------------------|----------------------------------|------------------------------------------------------------------|
| RSPOTIFY_CLIENT_ID     | XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX | Make spotify app on  <https://developer.spotify.com/dashboard>   |
| RSPOTIFY_CLIENT_SECRET | XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX | Make spotify app on  <https://developer.spotify.com/dashboard>   |
| RSPOTIFY_REDIRECT_URI  | <https://localhost:8888/>          | Make spotify app on <https://developer.spotify.com/dashboard>  |

Required
Documentation <https://docs.rs/rspotify/latest/rspotify/struct.Config.html>
Spotify client config recommended to keep as defaults not tested changed
| Options                           | Default | Info                          |
|-----------------------------------|---------|-------------------------------|
| RSPOTIFY_CLIENT_PREFIX            | default | DO NOT CHANGE                 |
| RSPOTIFY_CLIENT_CACHE_PATH        | default | Enter valid path              |
| RSPOTIFY_CLIENT_PAGINATION_CHUNKS | default | Not higher than 50 or default |
| RSPOTIFY_CLIENT_TOKEN_CACHED      | true    | True or false                 |
| RSPOTIFY_CLIENT_TOKEN_REFRESHING  | true    | True or false                 |

Required
Playing settings
| Options       | Default      | Info                     |
|---------------|--------------|--------------------------|
| PLAYLIST_NAME | AFK_PLAYLIST | Name of playlist to paly |

Required
Documentation <https://github.com/Spotifyd/spotifyd>
Documentation <https://spotifyd.github.io/spotifyd/Introduction.html>
Spotifyd settings
| Options              | Default                   | Info                                                                                 |
|----------------------|---------------------------|--------------------------------------------------------------------------------------|
| SPOTIFYD_CONFIG_PATH | .spotifyd.conf            | Path where the spotifyd config file temporary get stored                             |
| SPOTIFYD_USERNAME    | XXXXXXXXXXXXXXXXXXXXXXXXX | Your spotify username found on this page <https://www.spotify.com/account/overview/> |
| SPOTIFYD_PASSWORD    | XXXXXXXXXXXXXXXXXXXXXXXXX | Your spotify password**                                                              |
| SPOTIFYD_DEVICE_NAME | AFK_DEVICE                | The name of the device the program will use to afk with                              |

Optional
Bitly API
| Options         | Default                                 | Info                                         |
|-----------------|-----------------------------------------|----------------------------------------------|
| BITLY_API_TOKEN | XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX | Get your API key on <https://dev.bitly.com/> |

Rename this example.env to .env
