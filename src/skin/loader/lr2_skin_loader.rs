use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::warn;

use crate::skin::{
    Destination, ImageDef, Skin, SkinHeader, SkinObjectData, SkinObjectType, SkinSource, SkinType,
};

pub struct Lr2SkinLoader;

impl Default for Lr2SkinLoader {
    fn default() -> Self {
        Self::new()
    }
}

impl Lr2SkinLoader {
    pub fn new() -> Self {
        Self
    }

    pub fn load(&self, path: &Path, options: &HashMap<i32, i32>) -> Result<Skin> {
        let base_dir = path.parent().unwrap_or(Path::new("."));
        let mut parser = Lr2Parser::new(base_dir.to_path_buf(), options.clone());
        parser.parse_file(path)?;
        Ok(parser.finish())
    }

    pub fn load_header(&self, path: &Path) -> Result<SkinHeader> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read skin file: {}", path.display()))?;
        Ok(parse_header_only(&content))
    }
}

struct Lr2Parser {
    base_dir: PathBuf,
    options: HashMap<i32, i32>,
    skin: Skin,
    objects: HashMap<String, SkinObjectData>,
    condition_stack: Vec<(bool, bool)>,
    condition_active: bool,
    in_header: bool,
}

impl Lr2Parser {
    fn new(base_dir: PathBuf, options: HashMap<i32, i32>) -> Self {
        Self {
            base_dir,
            options,
            skin: Skin::new(SkinHeader::default()),
            objects: HashMap::new(),
            condition_stack: Vec::new(),
            condition_active: true,
            in_header: false,
        }
    }

    fn finish(mut self) -> Skin {
        self.skin.objects = self.objects.into_values().collect();
        self.skin
    }

    fn parse_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read skin file: {}", path.display()))?;
        let current_dir = path.parent().unwrap_or(&self.base_dir).to_path_buf();

        for raw_line in content.lines() {
            self.parse_line(raw_line, &current_dir)?;
        }

        Ok(())
    }

    fn parse_line(&mut self, raw_line: &str, current_dir: &Path) -> Result<()> {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            return Ok(());
        }

        if trimmed.starts_with("#INFORMATION") {
            self.in_header = true;
            return Ok(());
        }

        if trimmed.starts_with("#ENDOFHEADER") {
            self.in_header = false;
            return Ok(());
        }

        if self.in_header {
            self.parse_header_line(trimmed);
            return Ok(());
        }

        if !trimmed.starts_with('#') {
            return Ok(());
        }

        if trimmed.starts_with("#IF") {
            let condition = eval_condition(trimmed, &self.options);
            let parent_active = self.condition_active;
            self.condition_stack.push((parent_active, condition));
            self.condition_active = parent_active && condition;
            return Ok(());
        }

        if trimmed.starts_with("#ELSE") {
            if let Some((parent_active, condition)) = self.condition_stack.last() {
                self.condition_active = *parent_active && !*condition;
            }
            return Ok(());
        }

        if trimmed.starts_with("#ENDIF") {
            if let Some((parent_active, _)) = self.condition_stack.pop() {
                self.condition_active = parent_active;
            }
            return Ok(());
        }

        if !self.condition_active {
            return Ok(());
        }

        if trimmed.starts_with("#INCLUDE") {
            if let Some(path) = parse_include_path(trimmed) {
                let include_path = current_dir.join(path);
                self.parse_file(&include_path)?;
            }
            return Ok(());
        }

        if trimmed.starts_with("#IMAGE") {
            self.parse_image_line(trimmed);
            return Ok(());
        }

        if trimmed.starts_with("#SRC_") {
            self.parse_src_line(trimmed);
            return Ok(());
        }

        if trimmed.starts_with("#DST_") {
            self.parse_dst_line(trimmed);
            return Ok(());
        }

        Ok(())
    }

    fn parse_header_line(&mut self, line: &str) {
        let tokens: Vec<&str> = line.split_whitespace().collect();
        if tokens.is_empty() {
            return;
        }

        match tokens[0].to_uppercase().as_str() {
            "TYPE" => {
                if let Some(value) = tokens.get(1).and_then(|v| v.parse::<i32>().ok()) {
                    if let Some(skin_type) = SkinType::from_i32(value) {
                        self.skin.header.skin_type = skin_type;
                    }
                }
            }
            "RESOLUTION" => {
                if let (Some(w), Some(h)) = (
                    tokens.get(1).and_then(|v| v.parse::<u32>().ok()),
                    tokens.get(2).and_then(|v| v.parse::<u32>().ok()),
                ) {
                    self.skin.header.width = w;
                    self.skin.header.height = h;
                }
            }
            "TITLE" | "NAME" => {
                if tokens.len() > 1 {
                    self.skin.header.name = tokens[1..].join(" ");
                }
            }
            "AUTHOR" => {
                if tokens.len() > 1 {
                    self.skin.header.author = tokens[1..].join(" ");
                }
            }
            "LOADEND" => {
                self.skin.header.loadend = tokens
                    .get(1)
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
            }
            "PLAYSTART" => {
                self.skin.header.playstart = tokens
                    .get(1)
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
            }
            "SCENE" => {
                self.skin.header.scene = tokens
                    .get(1)
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
            }
            "INPUT" => {
                self.skin.header.input = tokens
                    .get(1)
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
            }
            "CLOSE" => {
                self.skin.header.close = tokens
                    .get(1)
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
            }
            "FADEOUT" => {
                self.skin.header.fadeout = tokens
                    .get(1)
                    .and_then(|v| v.parse::<i32>().ok())
                    .unwrap_or(0);
            }
            _ => {}
        }
    }

    fn parse_image_line(&mut self, line: &str) {
        let (body, comment) = split_comment(line);
        let tokens = parse_tokens(body);
        if tokens.len() < 2 {
            return;
        }

        let path = normalize_path(&tokens[1]);
        let id = tokens
            .get(2)
            .and_then(|v| v.parse::<i32>().ok())
            .or_else(|| parse_comment_id(comment))
            .unwrap_or(-1);

        if id < 0 {
            warn!(
                "LR2 IMAGE missing id / LR2画像のIDが見つかりません: {}",
                line
            );
            return;
        }

        self.skin.sources.insert(
            id as u32,
            SkinSource {
                id: id as u32,
                path,
            },
        );
    }

    fn parse_src_line(&mut self, line: &str) {
        let tokens = parse_tokens(line);
        if tokens.len() < 7 {
            return;
        }

        let command = tokens[0].trim_start_matches('#');
        let base = base_command(command);
        let index = parse_i32(tokens.get(1));
        if index < 0 {
            return;
        }

        let src = parse_i32(tokens.get(2)).max(0) as u32;
        let x = parse_i32(tokens.get(3));
        let y = parse_i32(tokens.get(4));
        let w = parse_i32(tokens.get(5));
        let h = parse_i32(tokens.get(6));
        let divx = parse_i32(tokens.get(7)).max(1);
        let divy = parse_i32(tokens.get(8)).max(1);
        let cycle = parse_i32(tokens.get(9));
        let timer = parse_i32(tokens.get(10));

        let id = object_id(base, index);
        let image = ImageDef {
            id: id.clone(),
            src,
            x,
            y,
            w,
            h,
            divx,
            divy,
            timer,
            cycle,
        };

        self.skin.images.insert(id, image);
    }

    fn parse_dst_line(&mut self, line: &str) {
        let tokens = parse_tokens(line);
        if tokens.len() < 7 {
            return;
        }

        let command = tokens[0].trim_start_matches('#');
        let base = base_command(command);
        let index = parse_i32(tokens.get(1));
        if index < 0 {
            return;
        }

        let time = parse_i32(tokens.get(2));
        let x = parse_f32(tokens.get(3));
        let y = parse_f32(tokens.get(4));
        let w = parse_f32(tokens.get(5));
        let h = parse_f32(tokens.get(6));
        let acc = parse_i32(tokens.get(7));
        let a = parse_f32(tokens.get(8)).max(0.0);
        let r = parse_f32(tokens.get(9)).max(0.0);
        let g = parse_f32(tokens.get(10)).max(0.0);
        let b = parse_f32(tokens.get(11)).max(0.0);
        let blend = parse_i32(tokens.get(12));
        let filter = parse_i32(tokens.get(13));
        let angle = parse_f32(tokens.get(14));
        let center = parse_i32(tokens.get(15));
        let loop_count = parse_i32(tokens.get(16));
        let timer = parse_i32(tokens.get(17));

        let ops = [tokens.get(18), tokens.get(19), tokens.get(20)]
            .into_iter()
            .flatten()
            .filter_map(|v| v.parse::<i32>().ok())
            .filter(|v| *v != 0)
            .collect::<Vec<_>>();

        let id = object_id(base, index);
        let entry = self
            .objects
            .entry(id.clone())
            .or_insert_with(|| SkinObjectData {
                object_type: SkinObjectType::Image,
                id: id.clone(),
                op: Vec::new(),
                timer,
                loop_count,
                offset: 0,
                blend,
                filter,
                stretch: 0,
                dst: Vec::new(),
            });

        if entry.op.is_empty() {
            entry.op = ops;
        }
        if entry.timer == 0 {
            entry.timer = timer;
        }
        if entry.loop_count == 0 {
            entry.loop_count = loop_count;
        }
        if entry.blend == 0 {
            entry.blend = blend;
        }
        if entry.filter == 0 {
            entry.filter = filter;
        }

        entry.dst.push(Destination {
            time,
            x,
            y,
            w,
            h,
            acc,
            a,
            r,
            g,
            b,
            angle,
            center,
        });
    }
}

fn parse_header_only(content: &str) -> SkinHeader {
    let mut header = SkinHeader::default();
    let mut in_header = false;
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("#INFORMATION") {
            in_header = true;
            continue;
        }
        if trimmed.starts_with("#ENDOFHEADER") {
            break;
        }
        if in_header {
            let tokens: Vec<&str> = trimmed.split_whitespace().collect();
            if tokens.is_empty() {
                continue;
            }
            match tokens[0].to_uppercase().as_str() {
                "TYPE" => {
                    if let Some(value) = tokens.get(1).and_then(|v| v.parse::<i32>().ok()) {
                        if let Some(skin_type) = SkinType::from_i32(value) {
                            header.skin_type = skin_type;
                        }
                    }
                }
                "RESOLUTION" => {
                    if let (Some(w), Some(h)) = (
                        tokens.get(1).and_then(|v| v.parse::<u32>().ok()),
                        tokens.get(2).and_then(|v| v.parse::<u32>().ok()),
                    ) {
                        header.width = w;
                        header.height = h;
                    }
                }
                "TITLE" | "NAME" => {
                    if tokens.len() > 1 {
                        header.name = tokens[1..].join(" ");
                    }
                }
                "AUTHOR" => {
                    if tokens.len() > 1 {
                        header.author = tokens[1..].join(" ");
                    }
                }
                _ => {}
            }
        }
    }
    header
}

fn parse_include_path(line: &str) -> Option<String> {
    let tokens = parse_tokens(line);
    tokens.get(1).map(|s| normalize_path(s))
}

fn split_comment(line: &str) -> (&str, Option<&str>) {
    if let Some(idx) = line.find("//") {
        (&line[..idx], Some(&line[idx + 2..]))
    } else {
        (line, None)
    }
}

fn parse_comment_id(comment: Option<&str>) -> Option<i32> {
    comment.and_then(|c| {
        c.split_whitespace()
            .find_map(|token| token.parse::<i32>().ok())
    })
}

fn parse_tokens(line: &str) -> Vec<String> {
    let mut tokens: Vec<String> = line
        .split([',', '\t'])
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    if tokens.len() <= 1 {
        tokens = line
            .split_whitespace()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string())
            .collect();
    }

    tokens
}

fn parse_i32(token: Option<&String>) -> i32 {
    token.and_then(|v| v.parse::<i32>().ok()).unwrap_or(0)
}

fn parse_f32(token: Option<&String>) -> f32 {
    token.and_then(|v| v.parse::<f32>().ok()).unwrap_or(0.0)
}

fn object_id(command: &str, index: i32) -> String {
    format!("{}-{}", command, index)
}

fn base_command(command: &str) -> &str {
    command
        .strip_prefix("SRC_")
        .or_else(|| command.strip_prefix("DST_"))
        .unwrap_or(command)
}

fn normalize_path(path: &str) -> String {
    path.replace('\\', "/")
}

fn eval_condition(line: &str, options: &HashMap<i32, i32>) -> bool {
    let tokens = parse_tokens(line);
    if tokens.len() < 2 {
        return true;
    }

    let op = tokens[1].parse::<i32>().ok();
    match op.and_then(|id| options.get(&id).copied()) {
        Some(value) => value != 0,
        None => true,
    }
}
