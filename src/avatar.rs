use std::{
    collections::BTreeMap,
    fs,
    io::Cursor,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use fontdue::{
    Font, FontSettings,
    layout::{CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle, VerticalAlign},
};
use image::{
    DynamicImage, GenericImageView, ImageFormat, Pixel, Rgba, RgbaImage,
    imageops::{FilterType, crop_imm, overlay, resize},
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use unicode_segmentation::UnicodeSegmentation;
use url::Url;

use crate::config::{AvatarConfig, AvatarProfile, EmojiConfig, resolve_path};

#[derive(Clone, Debug)]
pub enum ResolvedAvatar {
    RemoteUrl(String),
    Png(Vec<u8>),
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum AvatarSelection {
    Profile {
        name: String,
    },
    Inline {
        profile: AvatarConfig,
        #[serde(skip)]
        base_directory: PathBuf,
    },
}

pub struct AvatarRenderer {
    http: reqwest::Client,
}

impl AvatarRenderer {
    pub fn new() -> Result<Self> {
        Ok(Self {
            http: reqwest::Client::builder()
                .user_agent(concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION")
                ))
                .build()
                .context("could not initialize the avatar HTTP client")?,
        })
    }

    pub async fn resolve(
        &self,
        profile: &AvatarConfig,
        config_directory: &Path,
        data_directory: &Path,
        emoji_config: &EmojiConfig,
    ) -> Result<ResolvedAvatar> {
        match profile {
            AvatarConfig::Image { source, size: _ } if is_remote(source) => {
                validate_https(source, "avatar image URL")?;
                Ok(ResolvedAvatar::RemoteUrl(source.clone()))
            }
            AvatarConfig::Image { source, size } => {
                let path = resolve_path(config_directory, Path::new(source));
                let bytes = fs::read(&path)
                    .with_context(|| format!("could not read avatar image {}", path.display()))?;
                Ok(ResolvedAvatar::Png(square_png(&bytes, *size)?))
            }
            AvatarConfig::Emoji {
                emoji,
                background,
                foreground,
                size,
                scale,
            } => {
                let artwork = self
                    .emoji_artwork(emoji, data_directory, &emoji_config.asset_base_url)
                    .await?;
                Ok(ResolvedAvatar::Png(render_emoji(
                    &artwork,
                    foreground.as_deref().map(parse_color).transpose()?,
                    parse_color(background)?,
                    *size,
                    *scale,
                )?))
            }
            AvatarConfig::Text {
                text,
                foreground,
                background,
                font,
                size,
                font_size,
            } => {
                let font = load_font(config_directory, font.as_deref(), text)?;
                Ok(ResolvedAvatar::Png(render_text(
                    text,
                    &font,
                    parse_color(foreground)?,
                    parse_color(background)?,
                    *size,
                    *font_size,
                )?))
            }
            AvatarConfig::FontIcon {
                glyph,
                font,
                foreground,
                background,
                size,
                font_size,
            } => {
                let glyph = parse_glyph(glyph)?;
                let font_path = resolve_path(config_directory, font);
                let font = load_font_file(&font_path, &glyph)?;
                Ok(ResolvedAvatar::Png(render_text(
                    &glyph,
                    &font,
                    parse_color(foreground)?,
                    parse_color(background)?,
                    *size,
                    *font_size,
                )?))
            }
        }
    }

    pub async fn preview(
        &self,
        profile: &AvatarConfig,
        config_directory: &Path,
        data_directory: &Path,
        emoji_config: &EmojiConfig,
    ) -> Result<Vec<u8>> {
        match self
            .resolve(profile, config_directory, data_directory, emoji_config)
            .await?
        {
            ResolvedAvatar::Png(png) => Ok(png),
            ResolvedAvatar::RemoteUrl(url) => {
                let response = self
                    .http
                    .get(&url)
                    .send()
                    .await
                    .context("could not download remote avatar preview")?;
                ensure!(
                    response.status().is_success(),
                    "remote avatar preview returned HTTP {}",
                    response.status().as_u16()
                );
                let bytes = response
                    .bytes()
                    .await
                    .context("could not read remote avatar preview")?;
                let size = match profile {
                    AvatarConfig::Image { size, .. } => *size,
                    _ => unreachable!("only image profiles resolve to a remote URL"),
                };
                square_png(&bytes, size)
            }
        }
    }

    async fn emoji_artwork(
        &self,
        emoji: &str,
        data_directory: &Path,
        asset_base_url: &str,
    ) -> Result<Vec<u8>> {
        let codepoints = emoji_codepoints(emoji)?;
        let cache_directory = data_directory.join("emoji");
        fs::create_dir_all(&cache_directory).with_context(|| {
            format!("could not create emoji cache {}", cache_directory.display())
        })?;
        let cache_path = cache_directory.join(format!("{codepoints}.png"));
        if cache_path.is_file() {
            return fs::read(&cache_path)
                .with_context(|| format!("could not read cached emoji {}", cache_path.display()));
        }

        let mut base =
            Url::parse(asset_base_url).context("emoji.asset_base_url must be a valid URL")?;
        ensure!(
            base.scheme() == "https",
            "emoji.asset_base_url must use HTTPS"
        );
        if !base.path().ends_with('/') {
            base.set_path(&format!("{}/", base.path()));
        }
        let url = base
            .join(&format!("{codepoints}.png"))
            .context("could not construct the emoji artwork URL")?;
        let response = self
            .http
            .get(url)
            .send()
            .await
            .context("could not download emoji artwork")?;
        ensure!(
            response.status().is_success(),
            "emoji artwork provider returned HTTP {} for `{emoji}`",
            response.status().as_u16()
        );
        let bytes = response
            .bytes()
            .await
            .context("could not read emoji artwork")?
            .to_vec();
        image::load_from_memory(&bytes).context("emoji provider returned an invalid image")?;
        write_cache_file(&cache_path, &bytes)?;
        Ok(bytes)
    }
}

pub fn digest(png: &[u8]) -> String {
    hex::encode(Sha256::digest(png))
}

pub fn select_profile<'a>(
    profiles: &'a BTreeMap<String, AvatarProfile>,
    name: &str,
) -> Result<&'a AvatarConfig> {
    profiles
        .get(name)
        .map(|profile| &profile.avatar)
        .with_context(|| format!("unknown avatar profile `{name}`"))
}

pub fn parse_color(value: &str) -> Result<Rgba<u8>> {
    let digits = value.strip_prefix('#').unwrap_or(value);
    ensure!(
        matches!(digits.len(), 6 | 8)
            && digits
                .chars()
                .all(|character| character.is_ascii_hexdigit()),
        "color `{value}` must be #RRGGBB or #RRGGBBAA"
    );
    let red = u8::from_str_radix(&digits[0..2], 16)?;
    let green = u8::from_str_radix(&digits[2..4], 16)?;
    let blue = u8::from_str_radix(&digits[4..6], 16)?;
    let alpha = if digits.len() == 8 {
        u8::from_str_radix(&digits[6..8], 16)?
    } else {
        255
    };
    Ok(Rgba([red, green, blue, alpha]))
}

pub fn emoji_codepoints(emoji: &str) -> Result<String> {
    ensure!(
        emoji.graphemes(true).count() == 1,
        "emoji avatar must contain exactly one grapheme"
    );
    let codepoints: Vec<_> = emoji
        .chars()
        .map(u32::from)
        .filter(|codepoint| *codepoint != 0xFE0F)
        .map(|codepoint| format!("{codepoint:x}"))
        .collect();
    ensure!(!codepoints.is_empty(), "emoji avatar cannot be empty");
    Ok(codepoints.join("-"))
}

fn square_png(bytes: &[u8], size: u32) -> Result<Vec<u8>> {
    let image = image::load_from_memory(bytes).context("could not decode avatar image")?;
    let (width, height) = image.dimensions();
    ensure!(width > 0 && height > 0, "avatar image is empty");
    let side = width.min(height);
    let x = (width - side) / 2;
    let y = (height - side) / 2;
    let rgba = image.to_rgba8();
    let cropped = crop_imm(&rgba, x, y, side, side).to_image();
    let resized = resize(&cropped, size, size, FilterType::Lanczos3);
    encode_png(resized)
}

fn render_emoji(
    artwork: &[u8],
    foreground: Option<Rgba<u8>>,
    background: Rgba<u8>,
    size: u32,
    scale: f32,
) -> Result<Vec<u8>> {
    let mut image = image::load_from_memory(artwork)
        .context("could not decode emoji artwork")?
        .to_rgba8();
    if let Some(foreground) = foreground {
        recolor_emoji_artwork(&mut image, foreground);
    }
    let target = ((size as f32 * scale).round() as u32).clamp(1, size);
    let artwork = resize(&image, target, target, FilterType::Lanczos3);
    let mut canvas = RgbaImage::from_pixel(size, size, background);
    let offset = i64::from((size - target) / 2);
    overlay(&mut canvas, &artwork, offset, offset);
    encode_png(canvas)
}

fn recolor_emoji_artwork(image: &mut RgbaImage, foreground: Rgba<u8>) {
    for pixel in image.pixels_mut() {
        let alpha = (u16::from(pixel.0[3]) * u16::from(foreground.0[3]) + 127) / u16::from(u8::MAX);
        pixel.0 = [
            foreground.0[0],
            foreground.0[1],
            foreground.0[2],
            alpha as u8,
        ];
    }
}

fn render_text(
    text: &str,
    font: &Font,
    foreground: Rgba<u8>,
    background: Rgba<u8>,
    size: u32,
    font_size: f32,
) -> Result<Vec<u8>> {
    ensure_font_coverage(font, text)?;
    let mut canvas = RgbaImage::from_pixel(size, size, background);
    let fonts = [font];
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.reset(&LayoutSettings {
        max_width: Some(size as f32),
        max_height: Some(size as f32),
        horizontal_align: HorizontalAlign::Center,
        vertical_align: VerticalAlign::Middle,
        wrap_hard_breaks: false,
        ..LayoutSettings::default()
    });
    layout.append(&fonts, &TextStyle::new(text, font_size, 0));

    for glyph in layout.glyphs() {
        let (_, bitmap) = font.rasterize_config(glyph.key);
        for row in 0..glyph.height {
            for column in 0..glyph.width {
                let x = glyph.x.round() as i64 + column as i64;
                let y = glyph.y.round() as i64 + row as i64;
                if x < 0 || y < 0 || x >= i64::from(size) || y >= i64::from(size) {
                    continue;
                }
                let coverage = bitmap[row * glyph.width + column];
                if coverage == 0 {
                    continue;
                }
                let mut source = foreground;
                source.0[3] = ((u16::from(source.0[3]) * u16::from(coverage)) / 255) as u8;
                canvas.get_pixel_mut(x as u32, y as u32).blend(&source);
            }
        }
    }
    encode_png(canvas)
}

fn load_font(config_directory: &Path, configured: Option<&Path>, text: &str) -> Result<Font> {
    if let Some(path) = configured {
        return load_font_file(&resolve_path(config_directory, path), text);
    }

    let mut database = fontdb::Database::new();
    database.load_system_fonts();
    let face_ids: Vec<_> = database.faces().map(|face| face.id).collect();
    for id in face_ids {
        let candidate = database.with_face_data(id, |data, index| {
            Font::from_bytes(
                data,
                FontSettings {
                    collection_index: index,
                    ..FontSettings::default()
                },
            )
        });
        if let Some(Ok(font)) = candidate {
            if has_font_coverage(&font, text) {
                return Ok(font);
            }
        }
    }

    bail!(
        "no system font contains every glyph in `{text}`; configure a compatible `font` file in the avatar profile"
    )
}

fn load_font_file(path: &Path, text: &str) -> Result<Font> {
    let bytes =
        fs::read(path).with_context(|| format!("could not read font file {}", path.display()))?;
    let font = Font::from_bytes(bytes, FontSettings::default())
        .map_err(|error| anyhow::anyhow!("could not parse font {}: {error}", path.display()))?;
    ensure_font_coverage(&font, text).with_context(|| {
        format!(
            "font {} does not support the requested glyphs",
            path.display()
        )
    })?;
    Ok(font)
}

fn ensure_font_coverage(font: &Font, text: &str) -> Result<()> {
    ensure!(
        has_font_coverage(font, text),
        "selected font does not contain every requested glyph"
    );
    Ok(())
}

fn has_font_coverage(font: &Font, text: &str) -> bool {
    text.chars()
        .filter(|character| !character.is_whitespace())
        .all(|character| font.has_glyph(character))
}

fn parse_glyph(value: &str) -> Result<String> {
    let trimmed = value.trim();
    let codepoint = trimmed
        .strip_prefix("U+")
        .or_else(|| trimmed.strip_prefix("u+"))
        .or_else(|| trimmed.strip_prefix("0x"))
        .or_else(|| trimmed.strip_prefix("0X"));
    let glyph = if let Some(codepoint) = codepoint {
        let value = u32::from_str_radix(codepoint, 16)
            .with_context(|| format!("font-icon glyph `{trimmed}` is not hexadecimal"))?;
        char::from_u32(value)
            .with_context(|| format!("font-icon glyph `{trimmed}` is not valid Unicode"))?
            .to_string()
    } else if let Some(codepoint) = trimmed
        .strip_prefix("\\u{")
        .and_then(|value| value.strip_suffix('}'))
    {
        let value = u32::from_str_radix(codepoint, 16)
            .with_context(|| format!("font-icon glyph `{trimmed}` is not hexadecimal"))?;
        char::from_u32(value)
            .with_context(|| format!("font-icon glyph `{trimmed}` is not valid Unicode"))?
            .to_string()
    } else {
        trimmed.to_owned()
    };
    ensure!(
        glyph.chars().count() == 1,
        "font-icon glyph must resolve to exactly one Unicode character"
    );
    Ok(glyph)
}

fn encode_png(image: RgbaImage) -> Result<Vec<u8>> {
    let mut output = Cursor::new(Vec::new());
    DynamicImage::ImageRgba8(image)
        .write_to(&mut output, ImageFormat::Png)
        .context("could not encode avatar PNG")?;
    Ok(output.into_inner())
}

fn write_cache_file(path: &Path, bytes: &[u8]) -> Result<()> {
    let temporary = temporary_path(path);
    fs::write(&temporary, bytes)
        .with_context(|| format!("could not write emoji cache {}", temporary.display()))?;
    fs::rename(&temporary, path)
        .with_context(|| format!("could not update emoji cache {}", path.display()))
}

fn temporary_path(path: &Path) -> PathBuf {
    path.with_extension(format!("png.tmp-{}", std::process::id()))
}

fn validate_https(value: &str, field: &str) -> Result<()> {
    let url = Url::parse(value).with_context(|| format!("{field} must be a valid URL"))?;
    ensure!(url.scheme() == "https", "{field} must use HTTPS");
    Ok(())
}

fn is_remote(source: &str) -> bool {
    source.starts_with("https://") || source.starts_with("http://")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rgb_and_rgba_colors() {
        assert_eq!(parse_color("#5865F2").unwrap(), Rgba([88, 101, 242, 255]));
        assert_eq!(parse_color("FFFFFF80").unwrap(), Rgba([255, 255, 255, 128]));
        assert!(parse_color("#xyz").is_err());
    }

    #[test]
    fn converts_emoji_sequences_to_twemoji_names() {
        assert_eq!(emoji_codepoints("🚀").unwrap(), "1f680");
        assert_eq!(emoji_codepoints("❤️").unwrap(), "2764");
        assert!(emoji_codepoints("ab").is_err());
    }

    #[test]
    fn crops_images_to_square_pngs() {
        let source = RgbaImage::from_pixel(40, 20, Rgba([1, 2, 3, 255]));
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(source)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        let output = square_png(&bytes.into_inner(), 64).unwrap();
        let image = image::load_from_memory(&output).unwrap();
        assert_eq!(image.dimensions(), (64, 64));
    }

    #[test]
    fn composites_emoji_over_background() {
        let source = RgbaImage::from_pixel(10, 10, Rgba([255, 0, 0, 255]));
        let mut bytes = Cursor::new(Vec::new());
        DynamicImage::ImageRgba8(source)
            .write_to(&mut bytes, ImageFormat::Png)
            .unwrap();
        let output =
            render_emoji(&bytes.into_inner(), None, Rgba([0, 0, 255, 255]), 100, 0.5).unwrap();
        let image = image::load_from_memory(&output).unwrap().to_rgba8();
        assert_eq!(*image.get_pixel(0, 0), Rgba([0, 0, 255, 255]));
        assert_eq!(*image.get_pixel(50, 50), Rgba([255, 0, 0, 255]));
    }

    #[test]
    fn recolors_emoji_rgb_while_preserving_its_alpha_shape() {
        let mut image = RgbaImage::new(3, 1);
        image.put_pixel(0, 0, Rgba([221, 46, 68, 255]));
        image.put_pixel(1, 0, Rgba([221, 46, 68, 128]));
        image.put_pixel(2, 0, Rgba([0, 0, 0, 0]));

        recolor_emoji_artwork(&mut image, Rgba([255, 255, 255, 255]));

        assert_eq!(*image.get_pixel(0, 0), Rgba([255, 255, 255, 255]));
        assert_eq!(*image.get_pixel(1, 0), Rgba([255, 255, 255, 128]));
        assert_eq!(*image.get_pixel(2, 0), Rgba([255, 255, 255, 0]));
    }

    #[test]
    fn parses_font_icon_codepoints() {
        assert_eq!(parse_glyph("U+004E").unwrap(), "N");
        assert_eq!(parse_glyph("\\u{004e}").unwrap(), "N");
        assert_eq!(parse_glyph("告").unwrap(), "告");
        assert!(parse_glyph("AB").is_err());
    }

    #[test]
    fn renders_unicode_with_a_discovered_font() {
        let font = load_font(Path::new("."), None, "告").unwrap();
        let png = render_text(
            "告",
            &font,
            Rgba([255, 255, 255, 255]),
            Rgba([88, 101, 242, 255]),
            128,
            80.0,
        )
        .unwrap();
        assert_eq!(
            image::load_from_memory(&png).unwrap().dimensions(),
            (128, 128)
        );
        assert_eq!(digest(&png).len(), 64);
    }

    #[test]
    fn selects_named_profiles() {
        let mut profiles = BTreeMap::new();
        profiles.insert(
            "remote".to_owned(),
            AvatarConfig::Image {
                source: "https://example.com/avatar.png".to_owned(),
                size: 256,
            }
            .into(),
        );
        assert!(select_profile(&profiles, "remote").is_ok());
        assert!(select_profile(&profiles, "missing").is_err());
    }
}
