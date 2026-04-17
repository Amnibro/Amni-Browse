use std::collections::HashMap;

pub struct CachedImage {
    pub width: u32,
    pub height: u32,
    pub rgba_data: Vec<u8>,
}

pub struct ImageCache {
    entries: HashMap<String, CachedImage>,
    max_entries: usize,
}

impl ImageCache {
    pub fn new(max_entries: usize) -> Self {
        Self { entries: HashMap::new(), max_entries }
    }

    pub fn decode_bytes(&mut self, url: &str, data: &[u8]) -> Result<(), String> {
        if self.entries.len() >= self.max_entries {
            if let Some(oldest) = self.entries.keys().next().cloned() {
                self.entries.remove(&oldest);
            }
        }
        let img = image::load_from_memory(data).map_err(|e| format!("decode: {}", e))?;
        let rgba = img.to_rgba8();
        let (w, h) = rgba.dimensions();
        self.entries.insert(url.to_string(), CachedImage {
            width: w, height: h, rgba_data: rgba.into_raw(),
        });
        Ok(())
    }

    pub fn decode_and_resize(&mut self, url: &str, data: &[u8], max_dim: u32) -> Result<(), String> {
        let img = image::load_from_memory(data).map_err(|e| format!("decode: {}", e))?;
        let resized = if img.width() > max_dim || img.height() > max_dim {
            img.resize(max_dim, max_dim, image::imageops::FilterType::Triangle)
        } else { img };
        let rgba = resized.to_rgba8();
        let (w, h) = rgba.dimensions();
        if self.entries.len() >= self.max_entries {
            if let Some(oldest) = self.entries.keys().next().cloned() { self.entries.remove(&oldest); }
        }
        self.entries.insert(url.to_string(), CachedImage {
            width: w, height: h, rgba_data: rgba.into_raw(),
        });
        Ok(())
    }

    pub fn get(&self, url: &str) -> Option<&CachedImage> { self.entries.get(url) }

    pub fn contains(&self, url: &str) -> bool { self.entries.contains_key(url) }

    pub fn remove(&mut self, url: &str) { self.entries.remove(url); }

    pub fn clear(&mut self) { self.entries.clear(); }

    pub fn len(&self) -> usize { self.entries.len() }

    pub fn decode_file(&mut self, path: &str) -> Result<(), String> {
        let data = std::fs::read(path).map_err(|e| format!("read: {}", e))?;
        self.decode_bytes(path, &data)
    }

    pub fn create_solid_color(width: u32, height: u32, r: u8, g: u8, b: u8, a: u8) -> CachedImage {
        let mut rgba_data = Vec::with_capacity((width * height * 4) as usize);
        for _ in 0..width * height {
            rgba_data.extend_from_slice(&[r, g, b, a]);
        }
        CachedImage { width, height, rgba_data }
    }

    pub fn supported_formats() -> &'static [&'static str] {
        &["png", "jpg", "jpeg", "gif", "webp", "bmp", "ico"]
    }
}

pub fn detect_format(data: &[u8]) -> Option<&'static str> {
    if data.len() < 4 { return None; }
    match &data[..4] {
        [0x89, 0x50, 0x4E, 0x47] => Some("png"),
        [0xFF, 0xD8, 0xFF, ..] => Some("jpeg"),
        [0x47, 0x49, 0x46, 0x38] => Some("gif"),
        [0x52, 0x49, 0x46, 0x46] if data.len() > 12 && &data[8..12] == b"WEBP" => Some("webp"),
        [0x42, 0x4D, ..] => Some("bmp"),
        _ => None,
    }
}
