/// Skin configuration skin.
/// Translated from Java: SkinConfigurationSkin extends Skin
#[non_exhaustive]
pub struct SkinConfigurationSkin {
    pub sample_bms: Vec<String>,
    pub default_skin_type: i32,
    pub custom_property_count: i32,
    pub custom_offset_style: i32,
}

impl SkinConfigurationSkin {
    pub fn new() -> Self {
        Self {
            sample_bms: Vec::new(),
            default_skin_type: 0,
            custom_property_count: -1,
            custom_offset_style: 0,
        }
    }
    pub fn get_sample_bms(&self) -> &[String] {
        &self.sample_bms
    }
    pub fn get_default_skin_type(&self) -> i32 {
        self.default_skin_type
    }
    pub fn get_custom_offset_style(&self) -> i32 {
        self.custom_offset_style
    }
    pub fn get_custom_property_count(&self) -> i32 {
        self.custom_property_count
    }
}

impl Default for SkinConfigurationSkin {
    fn default() -> Self {
        Self::new()
    }
}
