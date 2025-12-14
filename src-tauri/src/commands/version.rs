use std::collections::HashMap;
// https://piston-meta.mojang.com/mc/game/version_manifest_v2.json
use reqwest;
use serde::{Serialize, Deserialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Versions>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Versions {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
    pub sha1: String,
    pub compliance_level: u32,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Version {
    pub arguments: Option<Value>,
    pub asset_index: Value,
    pub assets: String,
    pub compliance_level: u32,
    pub downloads: Value,
    pub id: String,
    pub java_version: Value,
    pub libraries: Vec<Library>,
    pub logging: Option<Value>,
    pub main_class: String,
    pub minimum_launcher_version: u32,
    pub release_time: String,
    pub time: String,
    pub r#type: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Library {
    pub downloads: Downloads,
    pub name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Downloads {
    pub artifact: Option<Artifact>,
    pub classifiers: Option<HashMap<String, Artifact>>
}

/*
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Classifiers {
    pub natives_linux: Option<Artifact>,
    pub natives_osx: Option<Artifact>,
    pub natives_windows: Option<Artifact>,
    pub natives_windows_64: Option<Artifact>,
    pub natives_window_32: Option<Artifact>,
} */

#[derive(Serialize, Deserialize, Debug)]
#[derive(Clone)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u32,
    pub url: String,
}

#[tauri::command]
pub async fn get_version(version_id: String) -> Version {
    // get the deserialized data form piston-meta
    let version_manifest = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .await
        .expect("Failed to get version manifest")
        .json::<VersionManifest>()
        .await
        .expect("Failed to parse version manifest");

    // find the chosen version
    let selected_version = version_manifest.versions.into_iter().find(|v| v.id == version_id).expect("Failed to find version in manifest");

    // request and deserialized the version
    let version = reqwest::get(selected_version.url)
        .await
        .expect("Failed to get version manifest")
        .json::<Version>()
        .await
        .expect("Failed to parse version manifest");

    return version
}

#[tauri::command]
pub async fn get_versions() -> VersionManifest {
    // get the deserialized data form piston-meta
    let version_manifest = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .await
        .expect("Failed to get version manifest")
        .json::<VersionManifest>()
        .await
        .expect("Failed to parse version manifest");

    // return the deserialized data
    version_manifest
}

#[tauri::command]
pub async fn get_versions_types() -> Vec<String> {
    // get the deserialized data form piston-meta
    let version_manifest = reqwest::get("https://piston-meta.mojang.com/mc/game/version_manifest_v2.json")
        .await
        .expect("Failed to get version manifest")
        .json::<VersionManifest>()
        .await
        .expect("Failed to parse version manifest");

    // get all types from all versions
    let mut ver_types: Vec<String> = version_manifest.versions.into_iter().map(|v| v.r#type.clone()).collect();

    // sort them
    ver_types.sort();

    // remove duplicate
    ver_types.dedup();

    return ver_types
}

