use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "assets/pieces"]
pub struct SvgSprites;

#[derive(RustEmbed)]
#[folder = "assets/fonts"]
pub struct EmbeddedFonts;
