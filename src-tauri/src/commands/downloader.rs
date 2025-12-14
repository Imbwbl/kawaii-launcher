use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::commands::version::Version;

async fn download_library(version: Version) -> tauri::async_runtime::JoinHandle<()>  {
    // new thread with a handle to wait on
    tauri::async_runtime::spawn(async move {

        // library path
        let library_folder = Path::new("minecraft/libraries");

        // iter on all libraries to download
        for lib in version.libraries {
            // list of artifacts to download
            let mut artifacts = Vec::new();

            // if there is an artifact we push it in the list
            if let Some(ref art) = lib.downloads.artifact {
                artifacts.push(art);
            }

            // if there is classifiers we push them in the list
            if let Some(ref classifiers) = lib.downloads.classifiers {
                for native_artifact in classifiers.values() {
                    artifacts.push(native_artifact);
                }
            }

            // iter on all artifacts
            for artifact in artifacts {

                // path to download the artifact
                let path = library_folder.join(&artifact.path);

                // create all dir to the path
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).expect("Failed to create parent directory");
                    }
                }

                // if the path doesn't already exist
                if !path.as_path().exists() {
                    println!("Downloading {}", artifact.url);

                    // request the bytes
                    let req = reqwest::get(&artifact.url)
                        .await
                        .expect("Failed to download library")
                        .bytes()
                        .await
                        .expect("Failed to download binary");

                    // create the file
                    let mut file = File::create(path).expect("Failed to create file");

                    // write the bytes into the file
                    file.write_all(&req).expect("Failed to write to file");
                }
            }
        }
    })
}

pub async fn start_download(version: Version) {
    // get the handle of the thread
    let library_handle = download_library(version).await;

    // wait on the thread
    library_handle.await.expect("Failed to join library handle");
}