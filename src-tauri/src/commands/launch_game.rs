use std::fs;
use std::path::Path;
use crate::commands::downloader;
use crate::commands::version::get_version;

fn create_folders() {
    let folders = [
        "minecraft",
        "minecraft/assets",
        "minecraft/libraries",
        "minecraft/versions",
    ];
    for folder in folders
    {
        if !Path::new(folder).exists() {
            fs::create_dir(folder).expect("Could not create folder");
        }
    }
}

#[tauri::command]
pub async fn launch_game(username: String, version: String) {
    create_folders();
    println!("Start launching the game as {} in {}!", username, version);
    downloader::start_download(get_version(version).await).await;
}