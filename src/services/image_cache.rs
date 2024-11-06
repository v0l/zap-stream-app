use crate::services::ffmpeg_loader::FfmpegLoader;
use crate::theme::NEUTRAL_800;
use anyhow::Error;
use egui::{ColorImage, Context, Image, ImageData, TextureHandle, TextureOptions};
use itertools::Itertools;
use log::{error, info};
use lru::LruCache;
use nostr_sdk::util::hex;
use resvg::usvg::Transform;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fs;
use std::num::NonZeroUsize;
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
            cache: Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1000).unwrap()))),
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

    fn load_bytes_impl(url: &str, bytes: &'static [u8]) -> Result<ColorImage, Error> {
        if url.ends_with(".svg") {
            Self::load_svg(bytes)
        } else {
            let loader = FfmpegLoader::new();
            loader.load_image_bytes(url, bytes)
        }
    }

    pub fn load_bytes<'a, U>(&self, url: U, bytes: &'static [u8]) -> Image<'a>
    where
        U: Into<String>,
    {
        let url = url.into();
        match Self::load_bytes_impl(&url, bytes) {
            Ok(i) => {
                let tex = self
                    .ctx
                    .load_texture(url, ImageData::from(i), TextureOptions::default());
                Image::from_texture(&tex)
            }
            Err(e) => {
                panic!("Failed to load image: {}", e);
            }
        }
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
        let loader = FfmpegLoader::new();
        match loader.load_image(path) {
            Ok(i) => Some(ctx.load_texture(key, ImageData::from(i), TextureOptions::default())),
            Err(e) => {
                println!("Failed to load image: {}", e);
                None
            }
        }
    }

    fn load_svg(svg: &[u8]) -> Result<ColorImage, Error> {
        use resvg::tiny_skia::Pixmap;
        use resvg::usvg::{Options, Tree};

        let opt = Options::default();
        let rtree = Tree::from_data(svg, &opt)
            .map_err(|err| err.to_string())
            .map_err(|e| Error::msg(e))?;

        let size = rtree.size().to_int_size();
        let (w, h) = (size.width(), size.height());

        let mut pixmap = Pixmap::new(w, h)
            .ok_or_else(|| Error::msg(format!("Failed to create SVG Pixmap of size {w}x{h}")))?;

        resvg::render(&rtree, Transform::default(), &mut pixmap.as_mut());
        let image = ColorImage::from_rgba_unmultiplied([w as _, h as _], pixmap.data());

        Ok(image)
    }
}
