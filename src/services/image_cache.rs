use crate::services::ffmpeg_loader::FfmpegLoader;
use crate::theme::NEUTRAL_800;
use anyhow::{Error, Result};
use egui::{ColorImage, Context, Image, ImageData, TextureHandle, TextureOptions};
use itertools::Itertools;
use log::{info, warn};
use lru::LruCache;
use nostr_sdk::util::hex;
use resvg::usvg::Transform;
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::fs;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

type ImageCacheStore = Arc<Mutex<LruCache<String, TextureHandle>>>;

#[derive(PartialEq, Eq, Hash, Clone)]
struct LoadRequest(String);

pub struct ImageCache {
    ctx: Context,
    dir: PathBuf,
    placeholder: TextureHandle,
    cache: ImageCacheStore,
    fetch_queue: Arc<Mutex<VecDeque<LoadRequest>>>,
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
        let cache = Arc::new(Mutex::new(LruCache::new(NonZeroUsize::new(1_000).unwrap())));
        let fetch_queue = Arc::new(Mutex::new(VecDeque::<LoadRequest>::new()));
        let cc = cache.clone();
        let fq = fetch_queue.clone();
        let out_dir = out.clone();
        let ctx_clone = ctx.clone();
        let placeholder_clone = placeholder.clone();
        tokio::spawn(async move {
            loop {
                let next = fq.lock().unwrap().pop_front();
                if let Some(next) = next {
                    let path = Self::find(&out_dir, &next.0);
                    if path.exists() {
                        let th = Self::load_image_texture(&ctx_clone, path, &next.0)
                            .unwrap_or(placeholder_clone.clone());
                        cc.lock().unwrap().put(next.0, th);
                        ctx_clone.request_repaint();
                    } else {
                        match Self::download_image_to_disk(&path, &next.0).await {
                            Ok(()) => {
                                let th = Self::load_image_texture(&ctx_clone, path, &next.0)
                                    .unwrap_or(placeholder_clone.clone());
                                cc.lock().unwrap().put(next.0, th);
                                ctx_clone.request_repaint();
                            }
                            Err(e) => {
                                warn!("Failed to download image {}: {}", next.0, e);
                                cc.lock().unwrap().put(next.0, placeholder_clone.clone());
                                ctx_clone.request_repaint();
                            }
                        }
                    }
                } else {
                    tokio::time::sleep(std::time::Duration::from_millis(30)).await;
                }
            }
        });
        Self {
            ctx,
            dir: out,
            placeholder,
            cache,
            fetch_queue,
        }
    }

    pub fn find<U>(dir: &PathBuf, url: U) -> PathBuf
    where
        U: Into<String>,
    {
        let mut sha = Sha256::new();
        sha2::digest::Update::update(&mut sha, url.into().as_bytes());
        let hash = hex::encode(sha.finalize());
        dir.join(PathBuf::from(hash[0..2].to_string()))
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
        if let Ok(mut ql) = self.fetch_queue.lock() {
            let lr = LoadRequest(u.clone());
            if !ql.contains(&lr) {
                ql.push_back(lr);
            }
        }
        Image::from_texture(&self.placeholder)
    }

    /// Download an image to disk
    async fn download_image_to_disk(dst: &PathBuf, u: &str) -> Result<()> {
        info!("Fetching image: {}", &u);
        tokio::fs::create_dir_all(dst.parent().unwrap()).await?;

        let data = reqwest::get(u).await?;
        let img_data = data.bytes().await?;
        tokio::fs::write(dst, img_data).await?;
        Ok(())
    }

    /// Load an image from disk into an egui texture handle
    fn load_image_texture(ctx: &Context, path: PathBuf, key: &str) -> Option<TextureHandle> {
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
