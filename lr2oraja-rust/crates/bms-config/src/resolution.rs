use std::fmt;

use serde::{Deserialize, Serialize};

/// Display resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "UPPERCASE")]
pub enum Resolution {
    Sd,
    Svga,
    Xga,
    #[default]
    Hd,
    Quadvga,
    Fwxga,
    Sxgaplus,
    Hdplus,
    Uxga,
    Wsxgaplus,
    Fullhd,
    Wuxga,
    Qxga,
    Wqhd,
    Ultrahd,
}

impl Resolution {
    pub fn width(self) -> i32 {
        match self {
            Self::Sd => 640,
            Self::Svga => 800,
            Self::Xga => 1024,
            Self::Hd => 1280,
            Self::Quadvga => 1280,
            Self::Fwxga => 1366,
            Self::Sxgaplus => 1400,
            Self::Hdplus => 1600,
            Self::Uxga => 1600,
            Self::Wsxgaplus => 1680,
            Self::Fullhd => 1920,
            Self::Wuxga => 1920,
            Self::Qxga => 2048,
            Self::Wqhd => 2560,
            Self::Ultrahd => 3840,
        }
    }

    pub fn height(self) -> i32 {
        match self {
            Self::Sd => 480,
            Self::Svga => 600,
            Self::Xga => 768,
            Self::Hd => 720,
            Self::Quadvga => 960,
            Self::Fwxga => 768,
            Self::Sxgaplus => 1050,
            Self::Hdplus => 900,
            Self::Uxga => 1200,
            Self::Wsxgaplus => 1050,
            Self::Fullhd => 1080,
            Self::Wuxga => 1200,
            Self::Qxga => 1536,
            Self::Wqhd => 1440,
            Self::Ultrahd => 2160,
        }
    }
}

impl fmt::Display for Resolution {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = serde_json::to_string(self).unwrap_or_default();
        let name = name.trim_matches('"');
        write!(f, "{} ({} x {})", name, self.width(), self.height())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_is_hd() {
        assert_eq!(Resolution::default(), Resolution::Hd);
    }

    #[test]
    fn test_dimensions() {
        assert_eq!(Resolution::Sd.width(), 640);
        assert_eq!(Resolution::Sd.height(), 480);
        assert_eq!(Resolution::Fullhd.width(), 1920);
        assert_eq!(Resolution::Fullhd.height(), 1080);
        assert_eq!(Resolution::Ultrahd.width(), 3840);
        assert_eq!(Resolution::Ultrahd.height(), 2160);
    }

    #[test]
    fn test_serde_round_trip() {
        let r = Resolution::Hd;
        let json = serde_json::to_string(&r).unwrap();
        assert_eq!(json, "\"HD\"");
        let back: Resolution = serde_json::from_str(&json).unwrap();
        assert_eq!(back, Resolution::Hd);
    }

    #[test]
    fn test_display() {
        assert_eq!(Resolution::Hd.to_string(), "HD (1280 x 720)");
        assert_eq!(Resolution::Sd.to_string(), "SD (640 x 480)");
    }

    #[test]
    fn test_all_variants_serde() {
        let variants = [
            (Resolution::Sd, "SD"),
            (Resolution::Svga, "SVGA"),
            (Resolution::Xga, "XGA"),
            (Resolution::Hd, "HD"),
            (Resolution::Quadvga, "QUADVGA"),
            (Resolution::Fwxga, "FWXGA"),
            (Resolution::Sxgaplus, "SXGAPLUS"),
            (Resolution::Hdplus, "HDPLUS"),
            (Resolution::Uxga, "UXGA"),
            (Resolution::Wsxgaplus, "WSXGAPLUS"),
            (Resolution::Fullhd, "FULLHD"),
            (Resolution::Wuxga, "WUXGA"),
            (Resolution::Qxga, "QXGA"),
            (Resolution::Wqhd, "WQHD"),
            (Resolution::Ultrahd, "ULTRAHD"),
        ];
        for (variant, name) in variants {
            let json = serde_json::to_string(&variant).unwrap();
            assert_eq!(json, format!("\"{}\"", name));
        }
    }
}
