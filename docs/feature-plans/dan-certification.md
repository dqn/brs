# Dan Certification System (段位認定)

段位認定モードの実装。複数譜面を連続プレイし、ゲージ継続で段位を取得するシステム。

## Overview

| 項目 | 内容 |
|------|------|
| 優先度 | 低 |
| 難易度 | 高 |
| 推定工数 | 4-5日 |
| 依存関係 | GAS (実装済み) |

## Background

段位認定は IIDX の段位認定モードを模したシステム。4 曲を連続でプレイし、ゲージを維持してクリアすることで段位を取得できる。

### Dan Grade Hierarchy

```
7級 → 6級 → ... → 1級 → 初段 → 二段 → ... → 十段 → 皆伝 → (Overjoy)
```

## Dependencies

- GAS (Gauge Auto Shift) - 実装済み
- スコア保存システム - 実装済み

## Files to Modify/Create

| ファイル | 変更内容 |
|----------|----------|
| `src/dan/mod.rs` (新規) | Dan モジュールルート |
| `src/dan/course.rs` (新規) | コース定義・読み込み |
| `src/dan/course_state.rs` (新規) | コース進行状態管理 |
| `src/scene/dan_select.rs` (新規) | 段位選択画面 |
| `src/scene/dan_gameplay.rs` (新規) | 段位プレイ画面 |
| `src/scene/dan_result.rs` (新規) | 段位結果画面 |
| `src/database/dan.rs` (新規) | 段位記録保存 |

## Implementation Phases

### Phase 1: Course Definition

```rust
// src/dan/course.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DanGrade {
    Kyu(u8),    // 7級-1級 (7-1)
    Dan(u8),    // 初段-十段 (1-10)
    Kaiden,     // 皆伝
    Overjoy,    // Overjoy (community grade)
}

impl DanGrade {
    pub fn display_name(&self) -> String {
        match self {
            Self::Kyu(n) => format!("{}級", n),
            Self::Dan(1) => "初段".to_string(),
            Self::Dan(2) => "二段".to_string(),
            Self::Dan(3) => "三段".to_string(),
            Self::Dan(4) => "四段".to_string(),
            Self::Dan(5) => "五段".to_string(),
            Self::Dan(6) => "六段".to_string(),
            Self::Dan(7) => "七段".to_string(),
            Self::Dan(8) => "八段".to_string(),
            Self::Dan(9) => "九段".to_string(),
            Self::Dan(10) => "十段".to_string(),
            Self::Dan(n) => format!("{}段", n),
            Self::Kaiden => "皆伝".to_string(),
            Self::Overjoy => "Overjoy".to_string(),
        }
    }

    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Kyu(1) => Some(Self::Dan(1)),
            Self::Kyu(n) => Some(Self::Kyu(n - 1)),
            Self::Dan(10) => Some(Self::Kaiden),
            Self::Dan(n) => Some(Self::Dan(n + 1)),
            Self::Kaiden => Some(Self::Overjoy),
            Self::Overjoy => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanCourse {
    /// Course name (e.g., "SP 初段")
    pub name: String,
    /// Dan grade
    pub grade: DanGrade,
    /// Chart file paths (typically 4 charts)
    pub charts: Vec<String>,
    /// Required gauge type (usually Hard or ExHard)
    pub gauge_type: GaugeType,
    /// Clear requirements
    pub requirements: DanRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanRequirements {
    /// Minimum gauge percentage at course end
    pub min_gauge: f32,
    /// Maximum BAD+POOR count (None = unlimited)
    pub max_bad_poor: Option<u32>,
    /// Require full combo
    pub full_combo: bool,
}

impl Default for DanRequirements {
    fn default() -> Self {
        Self {
            min_gauge: 0.0,  // Just survive
            max_bad_poor: None,
            full_combo: false,
        }
    }
}
```

### Phase 2: Course File Format

```json
// dan_courses/sp_shodan.json
{
  "name": "SP 初段",
  "grade": { "Dan": 1 },
  "charts": [
    "songs/dan/shodan/stage1.bms",
    "songs/dan/shodan/stage2.bms",
    "songs/dan/shodan/stage3.bms",
    "songs/dan/shodan/stage4.bms"
  ],
  "gauge_type": "Hard",
  "requirements": {
    "min_gauge": 0.0,
    "max_bad_poor": null,
    "full_combo": false
  }
}
```

### Phase 3: Course State Management

```rust
// src/dan/course_state.rs

pub struct CourseState {
    /// Course definition
    course: DanCourse,
    /// Current stage index (0-3 for 4-stage course)
    current_stage: usize,
    /// Gauge manager (carries over between stages)
    gauge: GaugeManager,
    /// Accumulated statistics
    total_stats: CourseStats,
    /// Per-stage results
    stage_results: Vec<StageResult>,
    /// Course start timestamp
    start_time: Instant,
}

#[derive(Debug, Clone, Default)]
pub struct CourseStats {
    pub total_ex_score: u32,
    pub total_max_combo: u32,
    pub total_pgreat: u32,
    pub total_great: u32,
    pub total_good: u32,
    pub total_bad: u32,
    pub total_poor: u32,
}

#[derive(Debug, Clone)]
pub struct StageResult {
    pub chart_path: String,
    pub ex_score: u32,
    pub max_combo: u32,
    pub gauge_at_end: f32,
}

impl CourseState {
    pub fn new(course: DanCourse) -> Self {
        let gauge = GaugeManager::new(
            course.gauge_type,
            GaugeSystem::Beatoraja,
            0,  // Note count set per stage
            300.0,
        );

        Self {
            course,
            current_stage: 0,
            gauge,
            total_stats: CourseStats::default(),
            stage_results: Vec::new(),
            start_time: Instant::now(),
        }
    }

    pub fn current_chart_path(&self) -> &str {
        &self.course.charts[self.current_stage]
    }

    pub fn stage_number(&self) -> usize {
        self.current_stage + 1
    }

    pub fn total_stages(&self) -> usize {
        self.course.charts.len()
    }

    /// Add result from completed stage and advance
    pub fn complete_stage(&mut self, result: PlayResult, final_gauge: f32) {
        self.stage_results.push(StageResult {
            chart_path: result.chart_path.clone(),
            ex_score: result.ex_score,
            max_combo: result.max_combo,
            gauge_at_end: final_gauge,
        });

        // Accumulate stats
        self.total_stats.total_ex_score += result.ex_score;
        self.total_stats.total_pgreat += result.pgreat_count;
        self.total_stats.total_great += result.great_count;
        self.total_stats.total_good += result.good_count;
        self.total_stats.total_bad += result.bad_count;
        self.total_stats.total_poor += result.poor_count;

        if result.max_combo > self.total_stats.total_max_combo {
            self.total_stats.total_max_combo = result.max_combo;
        }

        self.current_stage += 1;
    }

    /// Check if there are more stages
    pub fn has_next_stage(&self) -> bool {
        self.current_stage < self.course.charts.len()
    }

    /// Get gauge HP to carry to next stage
    pub fn carry_gauge_hp(&self) -> f32 {
        self.gauge.hp()
    }

    /// Check if course is cleared based on requirements
    pub fn is_cleared(&self) -> bool {
        // Must have completed all stages
        if self.current_stage < self.course.charts.len() {
            return false;
        }

        // Check gauge requirement
        let final_gauge = self.stage_results.last()
            .map(|r| r.gauge_at_end)
            .unwrap_or(0.0);

        if final_gauge < self.course.requirements.min_gauge {
            return false;
        }

        // Check BAD+POOR requirement
        if let Some(max_bp) = self.course.requirements.max_bad_poor {
            let total_bp = self.total_stats.total_bad + self.total_stats.total_poor;
            if total_bp > max_bp {
                return false;
            }
        }

        // Check full combo requirement
        if self.course.requirements.full_combo {
            if self.total_stats.total_bad > 0 || self.total_stats.total_poor > 0 {
                return false;
            }
        }

        true
    }

    /// Check if course is failed (gauge depleted)
    pub fn is_failed(&self) -> bool {
        self.gauge.is_failed()
    }
}
```

### Phase 4: Dan Gameplay Scene

```rust
// src/scene/dan_gameplay.rs

pub struct DanGameplayScene {
    course_state: CourseState,
    current_game: GameState,
    stage_display: StageDisplay,
}

impl DanGameplayScene {
    pub fn new(course: DanCourse) -> Self {
        let mut course_state = CourseState::new(course);
        let mut game = GameState::new();

        // Load first chart
        game.load_chart(course_state.current_chart_path())
            .expect("Failed to load dan chart");

        // Set gauge type from course
        game.set_gauge_type(course_state.course.gauge_type);

        Self {
            course_state,
            current_game: game,
            stage_display: StageDisplay::new(),
        }
    }

    fn load_next_stage(&mut self) {
        let chart_path = self.course_state.current_chart_path();

        // Create new game state but preserve gauge
        let carry_hp = self.course_state.carry_gauge_hp();
        self.current_game = GameState::new();
        self.current_game.load_chart(chart_path)
            .expect("Failed to load dan chart");

        // Restore gauge HP
        self.current_game.set_gauge_hp(carry_hp);
    }
}

impl Scene for DanGameplayScene {
    fn update(&mut self) -> SceneTransition {
        self.current_game.update();

        // Check for stage completion
        if self.current_game.is_finished() {
            let result = self.current_game.get_result(
                self.course_state.current_chart_path()
            );
            let final_gauge = self.current_game.gauge_hp();

            self.course_state.complete_stage(result, final_gauge);

            if self.course_state.has_next_stage() {
                // Load next stage
                self.load_next_stage();
                self.stage_display.show_stage_transition(
                    self.course_state.stage_number()
                );
            } else {
                // Course complete
                return SceneTransition::Replace(Box::new(
                    DanResultScene::new(self.course_state.clone())
                ));
            }
        }

        // Check for failure
        if self.current_game.is_failed() {
            return SceneTransition::Replace(Box::new(
                DanResultScene::new_failed(self.course_state.clone())
            ));
        }

        SceneTransition::None
    }

    fn draw(&self) {
        self.current_game.draw();

        // Draw stage indicator
        let stage_text = format!(
            "STAGE {}/{}",
            self.course_state.stage_number(),
            self.course_state.total_stages()
        );
        draw_text(&stage_text, 10.0, 60.0, 24.0, WHITE);

        // Draw course name
        draw_text(
            &self.course_state.course.name,
            10.0, 90.0, 20.0, YELLOW
        );

        self.stage_display.draw();
    }
}
```

### Phase 5: Dan Records

```rust
// src/database/dan.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DanRecord {
    /// Course identifier (hash of course definition)
    pub course_id: String,
    /// Dan grade
    pub grade: DanGrade,
    /// Whether cleared
    pub cleared: bool,
    /// Best EX score
    pub best_ex_score: u32,
    /// Clear count
    pub clear_count: u32,
    /// Play count
    pub play_count: u32,
    /// First clear timestamp
    pub first_clear: Option<u64>,
    /// Best clear timestamp
    pub best_clear: Option<u64>,
}

pub struct DanRepository {
    records: HashMap<String, DanRecord>,
    path: PathBuf,
}

impl DanRepository {
    pub fn load() -> Self {
        // Load from ~/.local/share/brs/brs/dan_records.json
        // ...
    }

    pub fn update(&mut self, course_id: &str, result: &CourseState) -> bool {
        let is_new_clear = result.is_cleared() && !self.is_cleared(course_id);
        // ... update logic
        is_new_clear
    }

    pub fn highest_cleared_grade(&self) -> Option<DanGrade> {
        self.records.values()
            .filter(|r| r.cleared)
            .map(|r| r.grade)
            .max_by_key(|g| g.sort_key())
    }
}
```

## Course Directory Structure

```
dan_courses/
├── sp/
│   ├── 7kyu.json
│   ├── 6kyu.json
│   ├── ...
│   ├── shodan.json
│   ├── nidan.json
│   ├── ...
│   └── kaiden.json
└── dp/
    └── ...
```

## Verification

1. テスト用の短いコース（2曲）を作成
2. ゲージ継続が正しく動作することを確認
3. ステージ間遷移が正しく動作することを確認
4. クリア判定が要件に基づいて正しく動作することを確認
5. 段位記録の保存・読み込みを確認

## Notes

- IIDX の段位認定は通常 4 曲構成
- ゲージタイプは通常 HARD（一部 EX-HARD）
- コース定義ファイルは JSON で外部化し、ユーザーがカスタム可能に
- 将来的には IR と連携して公式段位認定も可能
