use std::path::Path;

/// Image file extensions supported for BGA
pub static PIC_EXTENSION: &[&str] = &["jpg", "jpeg", "gif", "bmp", "png", "tga"];

/// BG image resource manager
pub struct BGImageProcessor {
    bgamap: Vec<Option<Vec<u8>>>,
    bgacache_ids: Vec<i32>,
    cache_size: usize,
}

impl BGImageProcessor {
    pub fn new(size: usize, _maxgen: i32) -> Self {
        BGImageProcessor {
            bgamap: vec![None; 1000],
            bgacache_ids: vec![-1; size],
            cache_size: size,
        }
    }

    pub fn put(&mut self, id: usize, _path: &Path) {
        // TODO: Phase 7+ dependency - requires PixmapResourcePool, Texture loading
        if id >= self.bgamap.len() {
            self.bgamap.resize(id + 1, None);
        }
        // Stub: store empty data to mark as loaded
        self.bgamap[id] = Some(Vec::new());
    }

    pub fn clear(&mut self) {
        for item in self.bgamap.iter_mut() {
            *item = None;
        }
    }

    pub fn dispose_old(&mut self) {
        // TODO: Phase 7+ dependency - requires PixmapResourcePool
    }

    pub fn prepare(&mut self, _timelines: &[()]) {
        // TODO: Phase 7+ dependency - requires TimeLine, Texture caching
        for id in self.bgacache_ids.iter_mut() {
            *id = -1;
        }
    }

    pub fn get_texture(&mut self, id: usize) -> bool {
        // Returns true if texture data exists for this id
        let cid = id % self.cache_size;
        if self.bgacache_ids[cid] == id as i32 {
            return true;
        }
        if id < self.bgamap.len() && self.bgamap[id].is_some() {
            self.bgacache_ids[cid] = id as i32;
            return true;
        }
        false
    }

    pub fn dispose(&mut self) {
        self.bgamap.clear();
        self.bgacache_ids.clear();
    }
}
