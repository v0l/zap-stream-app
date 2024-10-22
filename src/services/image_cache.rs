use crate::services::ffmpeg_loader::FfmpegLoader;
use crate::theme::NEUTRAL_800;
use anyhow::Error;
use eframe::epaint::Color32;
use egui::load::SizedTexture;
use egui::{ColorImage, Context, Image, ImageData, TextureHandle, TextureOptions};
use itertools::Itertools;
use log::{error, info};
use lru::LruCache;
use nostr_sdk::util::hex;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::num::NonZeroUsize;
use std::ops::Deref;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

type ImageCacheStore = Arc<Mutex<LruCache<String, TextureHandle>>>;

pub struct ImageCache {
    ctx: Context,
    dir: PathBuf,
    placeholder: TextureHandle,
    cache: ImageCacheStore,
    fetch_cache: Arc<Mutex<HashSet<String>>>,
}

impl ImageCache {
    pub fn new(data_path: PathBuf, ctx: Context) -> Self {
        let out = data_path.join("cache/images");
        fs::create_dir_all(&out).unwrap();

        let placeholder = ctx.load_texture(
            "placeholder",
            ImageData::from(ColorImage::new([1, 1], NEUTRAL_800)),
            TextureOptions::default(),
        );
        Self {
            ctx,
            dir: out,
            placeholder,
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()))),
            fetch_cache: Arc::new(Mutex::new(HashSet::new())),
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
        if let Ok(mut c) = self.cache.lock() {
            if let Some(i) = c.get(&u) {
                return Image::from_texture(i);
            }
        }
        let path = self.find(&u);
        if !path.exists() && !u.is_empty() {
            let path = path.clone();
            let cache = self.cache.clone();
            let ctx = self.ctx.clone();
            let fetch_cache = self.fetch_cache.clone();
            let placeholder = self.placeholder.clone();
            tokio::spawn(async move {
                if fetch_cache.lock().unwrap().insert(u.clone()) {
                    info!("Fetching image: {}", &u);
                    if let Ok(data) = reqwest::get(&u).await {
                        tokio::fs::create_dir_all(path.parent().unwrap())
                            .await
                            .unwrap();
                        let img_data = data.bytes().await.unwrap();
                        if let Err(e) = tokio::fs::write(path.clone(), img_data).await {
                            error!("Failed to write file: {}", e);
                        }
                        let t = Self::load_image(&ctx, path, &u)
                            .await
                            .unwrap_or(placeholder);
                        cache.lock().unwrap().put(u.clone(), t);
                        ctx.request_repaint();
                    }
                }
            });
        } else if path.exists() {
            let path = path.clone();
            let ctx = self.ctx.clone();
            let cache = self.cache.clone();
            let placeholder = self.placeholder.clone();
            tokio::spawn(async move {
                let t = Self::load_image(&ctx, path, &u)
                    .await
                    .unwrap_or(placeholder);
                cache.lock().unwrap().put(u.clone(), t);
                ctx.request_repaint();
            });
        }
        Image::from_texture(&self.placeholder)
    }

    async fn load_image(ctx: &Context, path: PathBuf, key: &str) -> Option<TextureHandle> {
        let mut loader = FfmpegLoader::new();
        match loader.load_image(path) {
            Ok(i) => Some(ctx.load_texture(key, ImageData::from(i), TextureOptions::default())),
            Err(e) => {
                println!("Failed to load image: {}", e);
                None
            }
        }
    }
}
