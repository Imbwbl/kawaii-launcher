use crate::commands::downloader;
use crate::commands::version::get_version;
use std::fs;
use std::path::{absolute, Path};
use std::process::Command;

pub static GAME_FOLDER: &str = "minecraft";
pub static LIBRARIES_FOLDER: &str = "minecraft/libraries";
pub static ASSETS_FOLDER: &str = "minecraft/assets";
pub static VERSIONS_FOLDER: &str = "minecraft/versions";
pub static NATIVES_FOLDER: &str = "minecraft/bin";

fn create_folders() {
    let folders = [
        GAME_FOLDER,
        LIBRARIES_FOLDER,
        ASSETS_FOLDER,
        VERSIONS_FOLDER,
        NATIVES_FOLDER,
    ];
    for folder in folders {
        if !Path::new(folder).exists() {
            fs::create_dir(folder).expect("Could not create folder");
        }
    }
}

fn get_libraries(dir: &Path) -> Vec<String> {
    let mut libraries = Vec::new();
    let dir = fs::read_dir(dir).expect("Failed to read dir");
    for file in dir {
        let path = file.unwrap().path();
        if path.is_dir() {
            libraries.extend(get_libraries(&path));
        } else {
            let absolute = absolute(&path).unwrap().to_string_lossy().to_string();
            libraries.push(absolute);
        }
    }
    libraries
}

#[tauri::command]
pub async fn launch_game(username: String, version: String) {
    create_folders();
    downloader::start_download(get_version(version.clone()).await).await;
    println!(
        "Start launching the game as {} in {}!",
        username,
        version.clone()
    );
    let version = get_version(version).await;
    let libraries = get_libraries(Path::new(LIBRARIES_FOLDER));
    let _separator = ":";
    #[cfg(target_os = "windows")]
    let _separator = ";";

    let classpath = format!(
        "{}/{}/{}.jar{}{}",
        VERSIONS_FOLDER,
        &version.id,
        &version.id,
        _separator,
        libraries.join(_separator)
    );

    let _command = "java";
    #[cfg(target_os = "windows")]
    let _command = "javaw";

    Command::new(_command)
    .args(&[
        #[cfg(target_os = "macos")]
        "-XstartOnFirstThread",
        #[cfg(target_os = "windows")]
        "-XX:HeapDumpPath=MojangTricksIntelDriversForPerformance_javaw.exe_minecraft.exe.heapdump", 
        #[cfg(target_arch = "x86")]
        "-Xss1M",
        &format!("-Djava.library.path={}", NATIVES_FOLDER),
        &format!("-Djna.tmpdir={}", NATIVES_FOLDER),
        &format!("-Dorg.lwjgl.system.SharedLibraryExtractPath={}", NATIVES_FOLDER),
        &format!("-Dio.netty.native.workdir={}", NATIVES_FOLDER),
        &format!("-Dminecraft.launcher.brand={}", "Kawaii"),
        &format!("-Dminecraft.launcher.version={}", 100),
        "-cp",
        &classpath,
        &version.main_class,
        "--username",
        &username, 
        "--version",
        &version.id,
        "--gameDir",
        GAME_FOLDER,
        "--assetsDir",
        ASSETS_FOLDER,
        "--assetIndex",
        &version.asset_index.id,
        "--uuid",
        "00000000-0000-0000-0000-000000000000",
        "--accessToken",
        "0",
        "--versionType",
        &version.r#type,
    ])
    .spawn()
    .expect("Failed to run the game");
}
