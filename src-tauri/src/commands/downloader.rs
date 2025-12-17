use reqwest::Client;
use futures::{stream, StreamExt};
use crate::commands::version::{get_versions, Version};
use std::fs;
use std::fs::File;
use std::io::{Write, copy};
use std::path::Path;
use crate::commands::asset::get_assets;

async fn download_assets(version: Version) -> tauri::async_runtime::JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        let assets = Path::new("minecraft/assets");
        let assets_index = assets.join(format!("indexes/{}.json", version.assets));
        let assets_objects = assets.join("objects");

        let client = Client::new();

        if let Some(parent) = assets_index.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create parent directory");
            }
        }

        let path = assets_objects.as_path();
        if !path.exists() {
            fs::create_dir_all(path).expect("Failed to create parent directory");
        }
        if !assets_index.as_path().exists() {
            println!("Downloading {}", version.asset_index.url);

            // request the bytes
            let req = client.get(&version.asset_index.url)
                .send()
                .await
                .expect("Failed to download library")
                .bytes()
                .await
                .expect("Failed to download binary");

            // create the file
            let mut file = File::create(&assets_index).expect("Failed to create file");

            // write the bytes into the file
            file.write_all(&req).expect("Failed to write to file");
        }

        let asset_list = get_assets(version).await;
        let bodies = stream::iter(asset_list.objects.into_values())
        .map(|asset| {
            let client = client.clone();
            let hash = asset.hash.clone();
            let value = assets_objects.clone();
            async move {
                let two = hash.split_at(2).0;
                let path = value.join(format!("{}/{}", two, asset.hash));
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).expect("Failed to create parent directory");
                    }
                }   
                let url = format!("https://resources.download.minecraft.net/{}/{}", two, hash);
                let resp = client.get(&url).send().await.unwrap();
                resp.bytes().await
            }
        })
        .buffer_unordered(50);
        bodies
        .for_each(|b| async {
            match b {
                Ok(b) => {
                        if b.is_empty() {
                            println!("Skipped existing file");
                        } else {
                            println!("Downloaded {} bytes", b.len());
                            let mut file = File::create(&assets_index).expect("Failed to create file");
                            file.write_all(&b).expect("Failed to write to file")
                        }
                    },
                Err(e) => eprintln!("Got an error: {}", e),
            }
        })
        .await;

        /*for asset in asset_list.objects.values() {
            let two = asset.hash.split_at(2).0;
            let path = assets_objects.join(format!("{}/{}", two, asset.hash));
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).expect("Failed to create parent directory");
                }
            }   
            if !path.as_path().exists() {
                let url = format!("https://resources.download.minecraft.net/{}/{}", two, asset.hash);
                println!("Downloading {}", url);

                // request the bytes
                let req = reqwest::get(&url)
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
        }*/
    })
}

async fn download_library(version: Version) -> tauri::async_runtime::JoinHandle<()> {
    // new thread with a handle to wait on
    tauri::async_runtime::spawn(async move {
        // library path
        let library_folder = Path::new("minecraft/libraries");
        let native_folder = Path::new("minecraft/bin");

        // iter on all libraries to download
        for lib in version.libraries {
            // list of artifacts to download
            let mut natives = Vec::new();

            // if there is classifiers we push them in the list
            if let Some(ref classifiers) = lib.downloads.classifiers {
                for native_artifact in classifiers.values() {
                    natives.push(native_artifact);
                }
            }

            // iter on all artifacts
            if lib.downloads.artifact.is_some() {
                let art = lib.downloads.artifact.unwrap(); 

                // path to download the artifact
                let path = library_folder.join(&art.path);

                // create all dir to the path
                if let Some(parent) = path.parent() {
                    if !parent.exists() {
                        fs::create_dir_all(parent).expect("Failed to create parent directory");
                    }
                }

                // if the path doesn't already exist
                if !path.as_path().exists() {
                    println!("Downloading {}", art.url);

                    // request the bytes
                    let req = reqwest::get(&art.url)
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

            if lib.downloads.classifiers.is_some() {
                for classifier in lib.downloads.classifiers.expect("failed to get classifiers").values() {
                    // path to download the artifact
                    let archive_path = native_folder.join("archive.zip");
                    // create all dir to the path
                    if let Some(parent) = archive_path.parent() {
                        if !parent.exists() {
                            fs::create_dir_all(parent).expect("Failed to create parent directory");
                        }
                    }
                    // if the path doesn't already exist
                    if !archive_path.exists() {
                        println!("Downloading {}", classifier.url);

                        // request the bytes
                        let req = reqwest::get(&classifier.url)
                            .await
                            .expect("Failed to download library")
                            .bytes()
                            .await
                            .expect("Failed to download binary");

                        // create the file
                        let mut file = File::create(archive_path).expect("Failed to create file");

                        //// ERROR TO FIX : CHANGE PERMISSION
                        
                        // write the bytes into the file
                        file.write_all(&req).expect("Failed to write to file");

                        let mut archive = zip::ZipArchive::new(file).unwrap();
                        for i in 0..archive.len() {
                            let mut f = archive.by_index(i).unwrap();
                            let outpath = match f.enclosed_name() {
                                Some(path) => path,
                                None => continue,
                            };

                            {
                                let comment = f.comment();
                                if !comment.is_empty() {
                                    println!("File {i} comment: {comment}");
                                }
                            }

                            if f.is_dir() {
                                println!("File {} extracted to \"{}\"", i, outpath.display());
                                fs::create_dir_all(&outpath).unwrap();
                            } else {
                                println!(
                                    "File {} extracted to \"{}\" ({} bytes)",
                                    i,
                                    outpath.display(),
                                    f.size()
                                );
                                if let Some(p) = outpath.parent() {
                                    if !p.exists() {
                                        fs::create_dir_all(p).unwrap();
                                    }
                                }
                                let mut outfile = fs::File::create(&outpath).unwrap();
                                copy(&mut f, &mut outfile).unwrap();
                            }

                            // Get and Set permissions
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::PermissionsExt;

                                if let Some(mode) = f.unix_mode() {
                                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode)).unwrap();
                                }
                            }
                        }
                                        
                    }
                }
            }
        }
    })
}

async fn download_version(version: Version) -> tauri::async_runtime::JoinHandle<()> {
    // new thread with a handle to wait on
    tauri::async_runtime::spawn(async move {
        let version_folder = Path::new("minecraft/versions");
        let path = version_folder.join(format!("{}/{}.jar", version.id, version.id));
        let json_path = version_folder.join(format!("{}/{}.json", version.id, version.id));

        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create parent directory");
            }
        }

        if let Some(parent) = json_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).expect("Failed to create parent directory");
            }
        }

        if !path.as_path().exists() {
            println!("Downloading {}", version.id);

            // request the bytes
            let req = reqwest::get(&version.downloads.get("client").unwrap().url)
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

        if !json_path.as_path().exists() {
            println!("Downloading {}", version.id);

            let versions = get_versions().await;
            let selected_version = versions
                .versions
                .into_iter()
                .find(|v| v.id == version.id)
                .expect("Failed to find version in manifest");

            let json_version = reqwest::get(selected_version.url)
                .await
                .expect("Failed to get version manifest")
                .bytes()
                .await
                .expect("Failed to parse version manifest");

            // create the file
            let mut file = File::create(json_path).expect("Failed to create file");

            // write the bytes into the file
            file.write_all(&json_version)
                .expect("Failed to write to file");
        }
    })
}

pub async fn start_download(version: Version) {
    // get the handle of the thread
    let assets_handle = download_assets(version.clone()).await;

    let library_handle = download_library(version.clone()).await;

    let version_handle = download_version(version.clone()).await;

    // wait on the thread
    assets_handle.await.expect("Failed to join assets handle");
    library_handle.await.expect("Failed to join library handle");
    version_handle.await.expect("Failed to join version handle");
}
