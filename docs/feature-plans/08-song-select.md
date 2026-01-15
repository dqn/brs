# Song Select Implementation

## Overview

The song select screen is where players browse and choose songs to play. It needs to support large libraries with efficient filtering, sorting, and searching.

## Core Features

### Folder Navigation

```
📁 BMS
├── 📁 ★01 (Normal 1)
│   ├── 🎵 Song A [N] [H] [A]
│   └── 🎵 Song B [N] [H]
├── 📁 ★02 (Normal 2)
├── 📁 Insane
│   ├── 📁 ★1-5
│   └── 📁 ★6-10
└── 📁 Favorites ⭐
```

### Display Information

For each song entry:
- Title / Subtitle
- Artist
- Level (☆)
- Clear Lamp (color indicator)
- Best Score / Grade
- Play Count
- BPM (min-max if variable)

## Filter System

### By Clear Lamp

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LampFilter {
    All,
    NoPlay,       // Never played
    Failed,       // Failed only
    NotCleared,   // No play + Failed
    Cleared,      // Any clear
    HardCleared,  // HARD or better
    FullCombo,    // FC or better
}

impl LampFilter {
    pub fn matches(&self, lamp: ClearLamp) -> bool {
        match self {
            Self::All => true,
            Self::NoPlay => lamp == ClearLamp::NoPlay,
            Self::Failed => lamp == ClearLamp::Failed,
            Self::NotCleared => lamp <= ClearLamp::Failed,
            Self::Cleared => lamp >= ClearLamp::AssistEasy,
            Self::HardCleared => lamp >= ClearLamp::Hard,
            Self::FullCombo => lamp >= ClearLamp::FullCombo,
        }
    }
}
```

### By Level

```rust
pub struct LevelFilter {
    pub min_level: Option<u8>,
    pub max_level: Option<u8>,
}

impl LevelFilter {
    pub fn matches(&self, level: u8) -> bool {
        let above_min = self.min_level.map_or(true, |min| level >= min);
        let below_max = self.max_level.map_or(true, |max| level <= max);
        above_min && below_max
    }
}
```

### By Grade

```rust
pub enum GradeFilter {
    All,
    BelowA,     // F-B
    AOrAbove,   // A+
    AAOrAbove,  // AA+
    AAAOrAbove, // AAA+
}
```

### Combined Filter

```rust
pub struct SongFilter {
    pub lamp: LampFilter,
    pub level: LevelFilter,
    pub grade: Option<GradeFilter>,
    pub keyword: Option<String>,
    pub mode: Option<u8>,  // Key mode (7, 5, 14, etc.)
}

impl SongFilter {
    pub fn matches(&self, entry: &SongEntry) -> bool {
        if !self.lamp.matches(entry.clear_lamp) {
            return false;
        }
        if !self.level.matches(entry.level) {
            return false;
        }
        if let Some(keyword) = &self.keyword {
            let keyword_lower = keyword.to_lowercase();
            if !entry.title.to_lowercase().contains(&keyword_lower) &&
               !entry.artist.to_lowercase().contains(&keyword_lower) {
                return false;
            }
        }
        true
    }
}
```

## Sort System

### Sort Keys

```rust
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Title,
    Artist,
    Level,
    ClearLamp,
    ExScore,
    ScoreRate,
    PlayCount,
    LastPlayed,
    BPM,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

pub struct SortConfig {
    pub key: SortKey,
    pub direction: SortDirection,
}

impl SortConfig {
    pub fn compare(&self, a: &SongEntry, b: &SongEntry) -> Ordering {
        let ord = match self.key {
            SortKey::Title => a.title.cmp(&b.title),
            SortKey::Artist => a.artist.cmp(&b.artist),
            SortKey::Level => a.level.cmp(&b.level),
            SortKey::ClearLamp => a.clear_lamp.cmp(&b.clear_lamp),
            SortKey::ExScore => a.ex_score.cmp(&b.ex_score),
            SortKey::ScoreRate => a.score_rate.partial_cmp(&b.score_rate)
                .unwrap_or(Ordering::Equal),
            SortKey::PlayCount => a.play_count.cmp(&b.play_count),
            SortKey::LastPlayed => a.last_played.cmp(&b.last_played),
            SortKey::BPM => a.bpm_max.partial_cmp(&b.bpm_max)
                .unwrap_or(Ordering::Equal),
        };

        match self.direction {
            SortDirection::Ascending => ord,
            SortDirection::Descending => ord.reverse(),
        }
    }
}
```

## Search

### Keyword Search

```rust
pub struct SearchEngine {
    index: HashMap<String, Vec<String>>,  // keyword -> sha256 list
}

impl SearchEngine {
    pub fn index_song(&mut self, song: &SongEntry) {
        let sha256 = &song.sha256;

        // Index title words
        for word in song.title.split_whitespace() {
            self.add_to_index(word.to_lowercase(), sha256.clone());
        }

        // Index artist
        for word in song.artist.split_whitespace() {
            self.add_to_index(word.to_lowercase(), sha256.clone());
        }

        // Index romaji/kana if available
        if let Some(title_romaji) = &song.title_romaji {
            for word in title_romaji.split_whitespace() {
                self.add_to_index(word.to_lowercase(), sha256.clone());
            }
        }
    }

    pub fn search(&self, query: &str) -> Vec<String> {
        let query_lower = query.to_lowercase();
        let mut results = HashSet::new();

        for (keyword, songs) in &self.index {
            if keyword.contains(&query_lower) {
                for sha256 in songs {
                    results.insert(sha256.clone());
                }
            }
        }

        results.into_iter().collect()
    }
}
```

## Favorites

```rust
pub struct FavoriteManager {
    favorites: HashSet<String>,  // Set of sha256
    db: ScoreManager,
}

impl FavoriteManager {
    pub fn toggle_favorite(&mut self, sha256: &str) {
        if self.favorites.contains(sha256) {
            self.favorites.remove(sha256);
        } else {
            self.favorites.insert(sha256.to_string());
        }
        self.save_favorites();
    }

    pub fn is_favorite(&self, sha256: &str) -> bool {
        self.favorites.contains(sha256)
    }

    pub fn get_favorite_entries(&self) -> Vec<String> {
        self.favorites.iter().cloned().collect()
    }
}
```

## Preview Playback

### Audio Preview

```rust
pub struct PreviewPlayer {
    audio_manager: AudioManager,
    current_preview: Option<PreviewState>,
    preview_delay_ms: u32,      // Delay before starting preview
    preview_duration_ms: u32,   // How long to play
}

struct PreviewState {
    sha256: String,
    handle: SoundHandle,
    start_time: Instant,
}

impl PreviewPlayer {
    pub fn new(audio_manager: AudioManager) -> Self {
        Self {
            audio_manager,
            current_preview: None,
            preview_delay_ms: 500,
            preview_duration_ms: 15000,
        }
    }

    pub fn on_selection_change(&mut self, entry: &SongEntry) {
        // Stop current preview
        self.stop_preview();

        // Schedule new preview after delay
        // (Actual scheduling handled by update loop)
    }

    pub fn update(&mut self, selected: &SongEntry, selection_time: Instant) {
        let elapsed = selection_time.elapsed().as_millis() as u32;

        // Start preview after delay
        if self.current_preview.is_none() && elapsed >= self.preview_delay_ms {
            self.start_preview(selected);
        }

        // Stop preview after duration
        if let Some(ref state) = self.current_preview {
            if state.start_time.elapsed().as_millis() as u32 >= self.preview_duration_ms {
                self.stop_preview();
            }
        }
    }

    fn start_preview(&mut self, entry: &SongEntry) {
        if let Some(preview_path) = &entry.preview_path {
            if let Ok(handle) = self.audio_manager.play_preview(preview_path) {
                self.current_preview = Some(PreviewState {
                    sha256: entry.sha256.clone(),
                    handle,
                    start_time: Instant::now(),
                });
            }
        }
    }

    fn stop_preview(&mut self) {
        if let Some(state) = self.current_preview.take() {
            self.audio_manager.stop(state.handle);
        }
    }
}
```

## Folder Scanning

### Initial Scan

```rust
pub struct FolderScanner {
    db: ScoreManager,
    bms_folders: Vec<PathBuf>,
}

impl FolderScanner {
    pub async fn scan_all(&self) -> anyhow::Result<ScanResult> {
        let mut result = ScanResult::default();

        for folder in &self.bms_folders {
            self.scan_folder(folder, &mut result).await?;
        }

        Ok(result)
    }

    async fn scan_folder(&self, path: &Path, result: &mut ScanResult) -> anyhow::Result<()> {
        for entry in walkdir::WalkDir::new(path)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();

            if let Some(ext) = path.extension() {
                match ext.to_str() {
                    Some("bms") | Some("bme") | Some("bml") | Some("pms") => {
                        match self.process_bms_file(path).await {
                            Ok(song) => {
                                result.songs.push(song);
                                result.found += 1;
                            }
                            Err(e) => {
                                result.errors.push((path.to_path_buf(), e.to_string()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    async fn process_bms_file(&self, path: &Path) -> anyhow::Result<SongEntry> {
        let content = tokio::fs::read_to_string(path).await?;

        // Quick header parse (full parse deferred to play time)
        let header = parse_bms_header(&content)?;

        let sha256 = calculate_chart_hash(content.as_bytes());

        Ok(SongEntry {
            sha256,
            path: path.to_path_buf(),
            title: header.title.unwrap_or_default(),
            subtitle: header.subtitle,
            artist: header.artist.unwrap_or_default(),
            level: header.playlevel.unwrap_or(0),
            bpm_min: header.bpm,
            bpm_max: header.bpm,
            ..Default::default()
        })
    }
}
```

### Incremental Rescan

```rust
impl FolderScanner {
    pub async fn rescan_changed(&self) -> anyhow::Result<ScanResult> {
        let mut result = ScanResult::default();

        // Get existing songs from DB
        let existing = self.db.get_all_song_paths()?;

        // Check for modified/new files
        for folder in &self.bms_folders {
            for entry in walkdir::WalkDir::new(folder)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                let path = entry.path();

                if let Some(existing_time) = existing.get(path) {
                    let modified = entry.metadata()?.modified()?;
                    if modified <= *existing_time {
                        continue;  // Not modified
                    }
                }

                // New or modified file
                if let Ok(song) = self.process_bms_file(path).await {
                    result.songs.push(song);
                    result.found += 1;
                }
            }
        }

        // Check for deleted files
        for (path, _) in existing {
            if !path.exists() {
                self.db.remove_song(&path)?;
                result.deleted += 1;
            }
        }

        Ok(result)
    }
}
```

## UI State

```rust
pub struct SongSelectState {
    /// All songs (filtered and sorted)
    pub entries: Vec<SongEntry>,
    /// Currently selected index
    pub selected_index: usize,
    /// Current folder path
    pub current_folder: Vec<String>,
    /// Active filter
    pub filter: SongFilter,
    /// Active sort
    pub sort: SortConfig,
    /// Scroll offset for display
    pub scroll_offset: usize,
    /// Number of visible entries
    pub visible_count: usize,
}

impl SongSelectState {
    pub fn move_selection(&mut self, delta: i32) {
        let new_index = (self.selected_index as i32 + delta)
            .clamp(0, self.entries.len() as i32 - 1) as usize;
        self.selected_index = new_index;

        // Adjust scroll to keep selection visible
        if self.selected_index < self.scroll_offset {
            self.scroll_offset = self.selected_index;
        } else if self.selected_index >= self.scroll_offset + self.visible_count {
            self.scroll_offset = self.selected_index - self.visible_count + 1;
        }
    }

    pub fn enter_folder(&mut self, folder_name: &str) {
        self.current_folder.push(folder_name.to_string());
        self.refresh_entries();
        self.selected_index = 0;
        self.scroll_offset = 0;
    }

    pub fn exit_folder(&mut self) {
        if !self.current_folder.is_empty() {
            self.current_folder.pop();
            self.refresh_entries();
        }
    }

    pub fn get_selected_entry(&self) -> Option<&SongEntry> {
        self.entries.get(self.selected_index)
    }

    fn refresh_entries(&mut self) {
        // Re-apply filter and sort
        // ... implementation
    }
}
```

## Reference Links

- [beatoraja Song Select](https://github.com/exch-bms2/beatoraja)
- [beatoraja Configuration - Music Select](https://github.com/wcko87/beatoraja-english-guide/wiki/Configuration)
- [walkdir crate](https://docs.rs/walkdir/)
