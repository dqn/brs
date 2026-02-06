use super::bar::{Bar, SongBar};
use crate::database::song_db::SongDatabase;

/// Search songs in the database and return matching bars.
pub fn search_songs(song_db: &SongDatabase, query: &str) -> Vec<Bar> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Vec::new();
    }
    let songs = song_db.search_songs(trimmed).unwrap_or_default();
    SongBar::from_songs(songs)
}

/// Filter bars by a text query (client-side filter).
pub fn filter_bars<'a>(bars: &'a [Bar], query: &str) -> Vec<&'a Bar> {
    let lower = query.to_lowercase();
    bars.iter()
        .filter(|bar| {
            let title = bar.title().to_lowercase();
            if title.contains(&lower) {
                return true;
            }
            if let Bar::Song(sb) = bar {
                let artist = sb.song.full_artist().to_lowercase();
                let genre = sb.song.genre.to_lowercase();
                return artist.contains(&lower) || genre.contains(&lower);
            }
            false
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::models::SongData;

    #[test]
    fn search_returns_results() {
        let db = SongDatabase::open_in_memory().unwrap();
        db.upsert_song(&SongData {
            title: "Freedom Dive".to_string(),
            artist: "xi".to_string(),
            sha256: "s1".to_string(),
            path: "p1".to_string(),
            ..Default::default()
        })
        .unwrap();

        let results = search_songs(&db, "Freedom");
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_empty_query() {
        let db = SongDatabase::open_in_memory().unwrap();
        let results = search_songs(&db, "");
        assert!(results.is_empty());
    }

    #[test]
    fn filter_by_title() {
        let bars = vec![
            Bar::Song(Box::new(SongBar::new(SongData {
                title: "Colorful".to_string(),
                sha256: "s1".to_string(),
                ..Default::default()
            }))),
            Bar::Song(Box::new(SongBar::new(SongData {
                title: "Another".to_string(),
                sha256: "s2".to_string(),
                ..Default::default()
            }))),
        ];
        let filtered = filter_bars(&bars, "color");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].title(), "Colorful");
    }

    #[test]
    fn filter_by_artist() {
        let bars = vec![Bar::Song(Box::new(SongBar::new(SongData {
            title: "Song".to_string(),
            artist: "dj TAKA".to_string(),
            sha256: "s1".to_string(),
            ..Default::default()
        })))];
        let filtered = filter_bars(&bars, "TAKA");
        assert_eq!(filtered.len(), 1);
    }
}
