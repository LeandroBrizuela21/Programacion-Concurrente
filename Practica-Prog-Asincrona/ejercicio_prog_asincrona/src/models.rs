use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct SearchAlbumsResponse {
    pub albums: AlbumPage,
} 

#[derive(Deserialize, Debug)]
pub struct AlbumPage {
    pub items: Vec<AlbumItem>,
}

#[derive(Deserialize, Debug)]
pub struct AlbumItem {
    pub id: String,
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct TrackResponse {
    pub items: Vec<Track>,
}

#[derive(Deserialize, Debug)]
pub struct Track {
    pub name: String,
    track_number: u32,
}

#[derive(Deserialize, Debug)]
pub struct AuthResponse {
    pub access_token: String,
}