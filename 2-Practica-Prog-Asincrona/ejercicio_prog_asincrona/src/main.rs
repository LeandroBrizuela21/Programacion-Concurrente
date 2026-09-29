use futures::future::join_all;
use reqwest::Client;
mod models;
use models::*;
use async_std::task; 
use std::env;

async fn get_new_releases(client: &Client, token: &str) -> Result<Vec<AlbumItem>, reqwest::Error> {
    let url = "https://api.spotify.com/v1/search?q=tag:new&type=album&limit=10";


    let response = client
                    .get(url)
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await?
                    .json::<SearchAlbumsResponse>()
                    .await?;
    
    Ok(response.albums.items)
}

async fn get_album_tracks(client: &Client, token: &str, album: &AlbumItem) -> Result<Vec<Track>, reqwest::Error> {
    let url = format!("https://api.spotify.com/v1/albums/{}/tracks", album.id);
    
    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?
        .json::<TrackResponse>()
        .await?;
    
    Ok(response.items)
} 

async fn get_token(client: &Client, client_id: &str, client_password: &str) -> Result<String, reqwest::Error>{
    let url = "https://api.spotify.com/v1/albums/{}/tracks";
    let response = client
        .post(url)
        .basic_auth(client_id, Some(client_password))
        .form(&[("grant_type", "client_credentials")])
        .send()
        .await?
        .json::<AuthResponse>()
        .await?;
    Ok(response.access_token) 
}


async fn async_main(){
    let client = Client::new();
    
    let client_id = env::var("SPOTIFY_CLIENT_ID").expect("Falta definir la variable SPOTIFY_CLIENT_ID en el archivo .env");
    let client_password = env::var("SPOTIFY_CLIENT_PASSWORD").expect("Falta definir la variable SPOTIFY_CLIENT_PASSWORD en el archivo .env");

    let token = match get_token(&client, &client_id, &client_password).await {
        Ok(t) => t,
        Err(_) => {
            eprint!("Error de credenciales. Revisar datos.");
            return;
        }        
    };

    let albums = match get_new_releases(&client, &token).await {
        Ok(albums) => albums,
        Err(e) => {
            eprintln!("Error al obtener los albumes: {}", e);
            return;
        } 
    };

    let tracks_futures = albums.iter().map(|album| {
        get_album_tracks(&client, &token, album)
    });

    let track_futures_results = join_all(tracks_futures).await;

    for (album, tracks_result) in albums.iter().zip(track_futures_results) {
        println!("ALBUM: {}", album.name);
        
        match tracks_result {
            Ok(tracks) => {
                for track in tracks {
                    println!("- {}", track.name);
                }
            }
            Err(e) => eprintln!("Error al obtener las canciones: {}", e)
        }
        println!()
    }
}

fn main() {
    task::block_on(async_main());
}
