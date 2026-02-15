use bms_database::{CourseData, CourseDataConstraint, SongDatabase, TrophyData};

use crate::widgets::item_list::show_item_list;
use crate::widgets::song_list::show_song_list;
use crate::widgets::song_search::SongSearchState;

// Constraint options per type
const GRADE_OPTIONS: &[(&str, Option<CourseDataConstraint>)] = &[
    ("None", None),
    ("Grade", Some(CourseDataConstraint::Class)),
    ("Grade (Mirror)", Some(CourseDataConstraint::GradeMirror)),
    ("Grade (Random)", Some(CourseDataConstraint::GradeRandom)),
];

const SPEED_OPTIONS: &[(&str, Option<CourseDataConstraint>)] = &[
    ("None", None),
    ("No Speed", Some(CourseDataConstraint::NoSpeed)),
];

const JUDGE_OPTIONS: &[(&str, Option<CourseDataConstraint>)] = &[
    ("None", None),
    ("No Good", Some(CourseDataConstraint::NoGood)),
    ("No Great", Some(CourseDataConstraint::NoGreat)),
];

const GAUGE_OPTIONS: &[(&str, Option<CourseDataConstraint>)] = &[
    ("None", None),
    ("LR2", Some(CourseDataConstraint::GaugeLr2)),
    ("5Key", Some(CourseDataConstraint::Gauge5Keys)),
    ("7Key", Some(CourseDataConstraint::Gauge7Keys)),
    ("9Key", Some(CourseDataConstraint::Gauge9Keys)),
    ("24Key", Some(CourseDataConstraint::Gauge24Keys)),
];

const LN_OPTIONS: &[(&str, Option<CourseDataConstraint>)] = &[
    ("None", None),
    ("LN", Some(CourseDataConstraint::Ln)),
    ("CN", Some(CourseDataConstraint::Cn)),
    ("HCN", Some(CourseDataConstraint::Hcn)),
];

fn default_trophies() -> Vec<TrophyData> {
    vec![
        TrophyData {
            name: "Bronze".to_string(),
            missrate: 7.5,
            scorerate: 55.0,
        },
        TrophyData {
            name: "Silver".to_string(),
            missrate: 5.0,
            scorerate: 70.0,
        },
        TrophyData {
            name: "Gold".to_string(),
            missrate: 2.5,
            scorerate: 85.0,
        },
    ]
}

/// State for the Course sub-tab of the Table Editor.
pub struct CourseEditorState {
    pub courses: Vec<CourseData>,
    selected_idx: Option<usize>,
    course_name: String,
    release: bool,
    // Constraint slots (indexed by constraint type 0-4)
    constraints: [Option<CourseDataConstraint>; 5],
    // Trophy data (3 rows: bronze, silver, gold)
    trophies: [TrophyData; 3],
    song_search: SongSearchState,
    pub dirty: bool,
}

impl Default for CourseEditorState {
    fn default() -> Self {
        let defaults = default_trophies();
        Self {
            courses: Vec::new(),
            selected_idx: None,
            course_name: String::new(),
            release: true,
            constraints: [None; 5],
            trophies: [
                defaults[0].clone(),
                defaults[1].clone(),
                defaults[2].clone(),
            ],
            song_search: SongSearchState::default(),
            dirty: false,
        }
    }
}

impl CourseEditorState {
    pub fn set_courses(&mut self, courses: Vec<CourseData>) {
        self.courses = courses;
        self.selected_idx = if self.courses.is_empty() {
            None
        } else {
            Some(0)
        };
        self.sync_from_selected();
        self.dirty = false;
    }

    pub fn get_courses(&self) -> &[CourseData] {
        &self.courses
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, song_db: Option<&SongDatabase>) {
        ui.columns(2, |cols| {
            // Left: course list
            cols[0].label("Courses");
            let list_changed = show_item_list(
                &mut cols[0],
                "course_list",
                &mut self.courses,
                &mut self.selected_idx,
                |c| {
                    if c.name.is_empty() {
                        "(unnamed)".to_string()
                    } else {
                        c.name.clone()
                    }
                },
                || CourseData {
                    name: "New Course".to_string(),
                    trophy: default_trophies(),
                    release: true,
                    ..Default::default()
                },
                "Add Course",
            );
            if list_changed {
                self.dirty = true;
                self.sync_from_selected();
            }

            // Right: course details
            if let Some(idx) = self.selected_idx {
                self.show_course_details(&mut cols[1], idx, song_db);
            } else {
                cols[1].label("Select or add a course to edit.");
            }
        });
    }

    fn show_course_details(
        &mut self,
        ui: &mut egui::Ui,
        idx: usize,
        song_db: Option<&SongDatabase>,
    ) {
        // Course name
        ui.horizontal(|ui| {
            ui.label("NAME");
            let prev = self.course_name.clone();
            ui.text_edit_singleline(&mut self.course_name);
            if self.course_name != prev {
                if let Some(course) = self.courses.get_mut(idx) {
                    course.name = self.course_name.clone();
                }
                self.dirty = true;
            }
        });

        // Release checkbox
        let prev_release = self.release;
        ui.checkbox(&mut self.release, "Release");
        if self.release != prev_release {
            if let Some(course) = self.courses.get_mut(idx) {
                course.release = self.release;
            }
            self.dirty = true;
        }

        ui.separator();

        // Song list
        ui.label("Songs");
        if let Some(course) = self.courses.get_mut(idx)
            && show_song_list(ui, "course_songs", &mut course.hash)
        {
            self.dirty = true;
        }

        // Song search
        let added = self.song_search.show(ui, song_db);
        if !added.is_empty()
            && let Some(course) = self.courses.get_mut(idx)
        {
            course.hash.extend(added);
            self.dirty = true;
        }

        ui.separator();

        // Constraints
        ui.label("CONSTRAINT");
        let mut constraint_changed = false;

        constraint_changed |=
            show_constraint_combo(ui, "Grade", GRADE_OPTIONS, &mut self.constraints[0]);
        constraint_changed |=
            show_constraint_combo(ui, "HiSpeed", SPEED_OPTIONS, &mut self.constraints[1]);
        constraint_changed |=
            show_constraint_combo(ui, "Judge", JUDGE_OPTIONS, &mut self.constraints[2]);
        constraint_changed |=
            show_constraint_combo(ui, "Gauge", GAUGE_OPTIONS, &mut self.constraints[3]);
        constraint_changed |= show_constraint_combo(ui, "LN", LN_OPTIONS, &mut self.constraints[4]);

        if constraint_changed {
            self.apply_constraints_to_course(idx);
            self.dirty = true;
        }

        ui.separator();

        // Trophy
        ui.label("TROPHY");
        let mut trophy_changed = false;

        for trophy in &mut self.trophies {
            ui.horizontal(|ui| {
                ui.label(format!("{:8}", trophy.name));
                ui.label("Miss%");
                let prev_miss = trophy.missrate;
                ui.add(
                    egui::DragValue::new(&mut trophy.missrate)
                        .range(0.0..=100.0)
                        .speed(0.1),
                );
                if trophy.missrate != prev_miss {
                    trophy_changed = true;
                }
                ui.label("Score%");
                let prev_score = trophy.scorerate;
                ui.add(
                    egui::DragValue::new(&mut trophy.scorerate)
                        .range(0.0..=100.0)
                        .speed(0.1),
                );
                if trophy.scorerate != prev_score {
                    trophy_changed = true;
                }
            });
        }

        if trophy_changed {
            self.apply_trophies_to_course(idx);
            self.dirty = true;
        }
    }

    fn apply_constraints_to_course(&mut self, idx: usize) {
        if let Some(course) = self.courses.get_mut(idx) {
            course.constraint = self.constraints.iter().filter_map(|c| *c).collect();
        }
    }

    fn apply_trophies_to_course(&mut self, idx: usize) {
        if let Some(course) = self.courses.get_mut(idx) {
            course.trophy = self
                .trophies
                .iter()
                .filter(|t| t.missrate > 0.0 && t.scorerate < 100.0)
                .cloned()
                .collect();
        }
    }

    fn sync_from_selected(&mut self) {
        if let Some(idx) = self.selected_idx {
            if let Some(course) = self.courses.get(idx) {
                self.course_name = course.name.clone();
                self.release = course.release;

                // Parse constraints into 5 slots
                self.constraints = [None; 5];
                for &c in &course.constraint {
                    let ct = c.constraint_type() as usize;
                    if ct < 5 {
                        self.constraints[ct] = Some(c);
                    }
                }

                // Parse trophies (ensure 3 rows)
                let defaults = default_trophies();
                self.trophies = [
                    course
                        .trophy
                        .first()
                        .cloned()
                        .unwrap_or_else(|| defaults[0].clone()),
                    course
                        .trophy
                        .get(1)
                        .cloned()
                        .unwrap_or_else(|| defaults[1].clone()),
                    course
                        .trophy
                        .get(2)
                        .cloned()
                        .unwrap_or_else(|| defaults[2].clone()),
                ];
            }
        } else {
            self.course_name.clear();
            self.release = true;
            self.constraints = [None; 5];
            self.trophies = [
                default_trophies().remove(0),
                default_trophies().remove(0),
                default_trophies().remove(0),
            ];
        }
        self.song_search = SongSearchState::default();
    }
}

/// Show a constraint combo box. Returns `true` if the selection changed.
fn show_constraint_combo(
    ui: &mut egui::Ui,
    label: &str,
    options: &[(&str, Option<CourseDataConstraint>)],
    current: &mut Option<CourseDataConstraint>,
) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.label(format!("{label}:"));

        let current_label = options
            .iter()
            .find(|(_, v)| *v == *current)
            .map(|(l, _)| *l)
            .unwrap_or("None");

        egui::ComboBox::from_id_salt(label)
            .selected_text(current_label)
            .show_ui(ui, |ui| {
                for &(name, value) in options {
                    if ui.selectable_label(*current == value, name).clicked() {
                        *current = value;
                        changed = true;
                    }
                }
            });
    });
    changed
}
