/// Folder data
#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct FolderData {
    pub title: String,
    pub subtitle: String,
    pub command: String,
    pub path: String,
    pub banner: String,
    pub parent: String,
    pub date: i32,
    pub max: i32,
    pub adddate: i32,
    #[serde(rename = "type")]
    pub folder_type: i32,
}
