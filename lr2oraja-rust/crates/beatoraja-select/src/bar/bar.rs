use crate::stubs::ScoreData;

/// Shared data for all bar types
/// Translates: bms.player.beatoraja.select.bar.Bar
#[derive(Clone, Debug, Default)]
pub struct BarData {
    /// Player score
    pub score: Option<ScoreData>,
    /// Rival score
    pub rscore: Option<ScoreData>,
}

impl BarData {
    pub fn get_score(&self) -> Option<&ScoreData> {
        self.score.as_ref()
    }

    pub fn set_score(&mut self, score: Option<ScoreData>) {
        self.score = score;
    }

    pub fn get_rival_score(&self) -> Option<&ScoreData> {
        self.rscore.as_ref()
    }

    pub fn set_rival_score(&mut self, score: Option<ScoreData>) {
        self.rscore = score;
    }
}

/// Bar enum representing all bar types in the select screen
pub enum Bar {
    Song(Box<super::song_bar::SongBar>),
    Folder(Box<super::folder_bar::FolderBar>),
    Command(Box<super::command_bar::CommandBar>),
    Container(Box<super::container_bar::ContainerBar>),
    Hash(Box<super::hash_bar::HashBar>),
    Table(Box<super::table_bar::TableBar>),
    Grade(Box<super::grade_bar::GradeBar>),
    RandomCourse(Box<super::random_course_bar::RandomCourseBar>),
    SearchWord(Box<super::search_word_bar::SearchWordBar>),
    SameFolder(Box<super::same_folder_bar::SameFolderBar>),
    Executable(Box<super::executable_bar::ExecutableBar>),
    Function(Box<super::function_bar::FunctionBar>),
    ContextMenu(Box<super::context_menu_bar::ContextMenuBar>),
    LeaderBoard(Box<super::leader_board_bar::LeaderBoardBar>),
}

impl Bar {
    pub fn get_title(&self) -> String {
        match self {
            Bar::Song(b) => b.get_title(),
            Bar::Folder(b) => b.get_title(),
            Bar::Command(b) => b.get_title(),
            Bar::Container(b) => b.get_title(),
            Bar::Hash(b) => b.get_title(),
            Bar::Table(b) => b.get_title(),
            Bar::Grade(b) => b.get_title(),
            Bar::RandomCourse(b) => b.get_title(),
            Bar::SearchWord(b) => b.get_title(),
            Bar::SameFolder(b) => b.get_title(),
            Bar::Executable(b) => b.get_title(),
            Bar::Function(b) => b.get_title(),
            Bar::ContextMenu(b) => b.get_title(),
            Bar::LeaderBoard(b) => b.get_title(),
        }
    }

    pub fn get_score(&self) -> Option<&ScoreData> {
        self.bar_data().get_score()
    }

    pub fn set_score(&mut self, score: Option<ScoreData>) {
        self.bar_data_mut().set_score(score);
    }

    pub fn get_rival_score(&self) -> Option<&ScoreData> {
        self.bar_data().get_rival_score()
    }

    pub fn set_rival_score(&mut self, score: Option<ScoreData>) {
        self.bar_data_mut().set_rival_score(score);
    }

    pub fn get_lamp(&self, is_player: bool) -> i32 {
        match self {
            Bar::Song(b) => b.get_lamp(is_player),
            Bar::Folder(b) => b.directory.get_lamp(is_player),
            Bar::Command(b) => b.directory.get_lamp(is_player),
            Bar::Container(b) => b.directory.get_lamp(is_player),
            Bar::Hash(b) => b.directory.get_lamp(is_player),
            Bar::Table(b) => b.directory.get_lamp(is_player),
            Bar::Grade(b) => b.get_lamp(is_player),
            Bar::RandomCourse(b) => b.get_lamp(is_player),
            Bar::SearchWord(b) => b.directory.get_lamp(is_player),
            Bar::SameFolder(b) => b.directory.get_lamp(is_player),
            Bar::Executable(b) => b.get_lamp(is_player),
            Bar::Function(b) => b.get_lamp(is_player),
            Bar::ContextMenu(b) => b.get_lamp(is_player),
            Bar::LeaderBoard(b) => b.directory.get_lamp(is_player),
        }
    }

    pub fn bar_data(&self) -> &BarData {
        match self {
            Bar::Song(b) => &b.selectable.bar_data,
            Bar::Folder(b) => &b.directory.bar_data,
            Bar::Command(b) => &b.directory.bar_data,
            Bar::Container(b) => &b.directory.bar_data,
            Bar::Hash(b) => &b.directory.bar_data,
            Bar::Table(b) => &b.directory.bar_data,
            Bar::Grade(b) => &b.selectable.bar_data,
            Bar::RandomCourse(b) => &b.selectable.bar_data,
            Bar::SearchWord(b) => &b.directory.bar_data,
            Bar::SameFolder(b) => &b.directory.bar_data,
            Bar::Executable(b) => &b.selectable.bar_data,
            Bar::Function(b) => &b.selectable.bar_data,
            Bar::ContextMenu(b) => &b.directory.bar_data,
            Bar::LeaderBoard(b) => &b.directory.bar_data,
        }
    }

    pub fn bar_data_mut(&mut self) -> &mut BarData {
        match self {
            Bar::Song(b) => &mut b.selectable.bar_data,
            Bar::Folder(b) => &mut b.directory.bar_data,
            Bar::Command(b) => &mut b.directory.bar_data,
            Bar::Container(b) => &mut b.directory.bar_data,
            Bar::Hash(b) => &mut b.directory.bar_data,
            Bar::Table(b) => &mut b.directory.bar_data,
            Bar::Grade(b) => &mut b.selectable.bar_data,
            Bar::RandomCourse(b) => &mut b.selectable.bar_data,
            Bar::SearchWord(b) => &mut b.directory.bar_data,
            Bar::SameFolder(b) => &mut b.directory.bar_data,
            Bar::Executable(b) => &mut b.selectable.bar_data,
            Bar::Function(b) => &mut b.selectable.bar_data,
            Bar::ContextMenu(b) => &mut b.directory.bar_data,
            Bar::LeaderBoard(b) => &mut b.directory.bar_data,
        }
    }

    /// Check if this bar is a SongBar
    pub fn as_song_bar(&self) -> Option<&super::song_bar::SongBar> {
        if let Bar::Song(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_song_bar_mut(&mut self) -> Option<&mut super::song_bar::SongBar> {
        if let Bar::Song(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_directory_bar(&self) -> Option<&super::directory_bar::DirectoryBarData> {
        match self {
            Bar::Folder(b) => Some(&b.directory),
            Bar::Command(b) => Some(&b.directory),
            Bar::Container(b) => Some(&b.directory),
            Bar::Hash(b) => Some(&b.directory),
            Bar::Table(b) => Some(&b.directory),
            Bar::SearchWord(b) => Some(&b.directory),
            Bar::SameFolder(b) => Some(&b.directory),
            Bar::ContextMenu(b) => Some(&b.directory),
            Bar::LeaderBoard(b) => Some(&b.directory),
            _ => None,
        }
    }

    pub fn as_grade_bar(&self) -> Option<&super::grade_bar::GradeBar> {
        if let Bar::Grade(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_grade_bar_mut(&mut self) -> Option<&mut super::grade_bar::GradeBar> {
        if let Bar::Grade(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_random_course_bar(&self) -> Option<&super::random_course_bar::RandomCourseBar> {
        if let Bar::RandomCourse(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_executable_bar(&self) -> Option<&super::executable_bar::ExecutableBar> {
        if let Bar::Executable(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_function_bar(&self) -> Option<&super::function_bar::FunctionBar> {
        if let Bar::Function(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_function_bar_mut(&mut self) -> Option<&mut super::function_bar::FunctionBar> {
        if let Bar::Function(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_folder_bar(&self) -> Option<&super::folder_bar::FolderBar> {
        if let Bar::Folder(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_table_bar(&self) -> Option<&super::table_bar::TableBar> {
        if let Bar::Table(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_hash_bar(&self) -> Option<&super::hash_bar::HashBar> {
        if let Bar::Hash(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_context_menu_bar(&self) -> Option<&super::context_menu_bar::ContextMenuBar> {
        if let Bar::ContextMenu(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_search_word_bar(&self) -> Option<&super::search_word_bar::SearchWordBar> {
        if let Bar::SearchWord(b) = self {
            Some(b)
        } else {
            None
        }
    }

    pub fn as_selectable_bar(&self) -> Option<&super::selectable_bar::SelectableBarData> {
        match self {
            Bar::Song(b) => Some(&b.selectable),
            Bar::Grade(b) => Some(&b.selectable),
            Bar::RandomCourse(b) => Some(&b.selectable),
            Bar::Executable(b) => Some(&b.selectable),
            Bar::Function(b) => Some(&b.selectable),
            _ => None,
        }
    }

    pub fn as_selectable_bar_mut(
        &mut self,
    ) -> Option<&mut super::selectable_bar::SelectableBarData> {
        match self {
            Bar::Song(b) => Some(&mut b.selectable),
            Bar::Grade(b) => Some(&mut b.selectable),
            Bar::RandomCourse(b) => Some(&mut b.selectable),
            Bar::Executable(b) => Some(&mut b.selectable),
            Bar::Function(b) => Some(&mut b.selectable),
            _ => None,
        }
    }

    /// Check if this is a DirectoryBar variant
    pub fn is_directory_bar(&self) -> bool {
        matches!(
            self,
            Bar::Folder(_)
                | Bar::Command(_)
                | Bar::Container(_)
                | Bar::Hash(_)
                | Bar::Table(_)
                | Bar::SearchWord(_)
                | Bar::SameFolder(_)
                | Bar::ContextMenu(_)
                | Bar::LeaderBoard(_)
        )
    }
}
