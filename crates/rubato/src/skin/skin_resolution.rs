/// Skin resolution data carrier (width x height in pixels).
///
/// This is the skin-local resolution type used for skin coordinate calculations,
/// distinct from `crate::skin::resolution::Resolution` which is a display resolution enum.
#[derive(Clone, Debug, Default)]
pub struct Resolution {
    pub width: f32,
    pub height: f32,
}
