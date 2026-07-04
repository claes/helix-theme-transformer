use palette16::{compact_colors, Base16Palette};

pub fn export_base16_yaml(palette: &Base16Palette) -> Result<String, serde_yaml::Error> {
    serde_yaml::to_string(&compact_colors(palette))
}
