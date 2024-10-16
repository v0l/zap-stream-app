use egui::Image;
use log::{error, info};
use nostr_sdk::util::hex;
use sha2::digest::Update;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::hash::Hash;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct ImageCache {
    ctx: egui::Context,
    dir: PathBuf,
    fetch_lock: Arc<Mutex<HashSet<String>>>,
}

impl ImageCache {
    pub fn new(ctx: egui::Context) -> Self {
        let out = PathBuf::from("./cache/images");
        fs::create_dir_all(&out).unwrap();
        Self {
            ctx,
            dir: out,
            fetch_lock: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn find<U>(&self, url: U) -> PathBuf
    where
        U: Into<String>,
    {
        let mut sha = Sha256::new();
        sha2::digest::Update::update(&mut sha, url.into().as_bytes());
        let hash = hex::encode(sha.finalize());
        self.dir
            .join(PathBuf::from(hash[0..2].to_string()))
            .join(PathBuf::from(hash))
    }

    pub fn load<'a, U>(&self, url: U) -> Image<'a>
    where
        U: Into<String>,
    {
        let u = url.into();
        let path = self.find(&u);
        if !path.exists() {
            let path = path.clone();
            let fl = self.fetch_lock.clone();
            let ctx = self.ctx.clone();
            tokio::spawn(async move {
                if fl.lock().await.insert(u.clone()) {
                    info!("Fetching image: {}", &u);
                    if let Ok(data) = reqwest::get(&u)
                        .await {
                        tokio::fs::create_dir_all(path.parent().unwrap()).await.unwrap();
                        if let Err(e) = tokio::fs::write(path, data.bytes().await.unwrap()).await {
                            error!("Failed to write file: {}", e);
                        }
                        ctx.request_repaint();
                    }
                }
            });
        }
        Image::from_uri(format!("file://{}", path.to_str().unwrap()))
    }
}