use std::error::Error;
use std::path::{Path, PathBuf};

const ICON_SVG: &str = "ui/icons/coffre.svg";
/// Sizes Windows picks from for the title bar, taskbar and Explorer views.
const ICON_SIZES: [u32; 5] = [16, 24, 32, 48, 256];

fn main() -> Result<(), Box<dyn Error>> {
    let configuration = slint_build::CompilerConfiguration::new().with_style("fluent-dark".into());
    slint_build::compile_with_config("ui/app-window.slint", configuration)?;

    println!("cargo:rerun-if-changed={ICON_SVG}");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // The .ico is derived from the SVG at build time, so the SVG stays the only source.
        let out_dir = PathBuf::from(std::env::var("OUT_DIR")?);
        let ico = out_dir.join("coffre.ico");
        std::fs::write(&ico, ico_from_svg(Path::new(ICON_SVG))?)?;
        let ico = ico.to_str().ok_or("chemin de l'icône non UTF-8")?;
        winresource::WindowsResource::new()
            .set_icon(ico)
            .compile()?;
    }
    Ok(())
}

/// Windows icon holding one PNG image per size (format accepted since Windows Vista).
fn ico_from_svg(svg: &Path) -> Result<Vec<u8>, Box<dyn Error>> {
    let tree = resvg::usvg::Tree::from_data(&std::fs::read(svg)?, &Default::default())?;
    let images = ICON_SIZES
        .iter()
        .map(|&size| render_png(&tree, size).map(|png| (size, png)))
        .collect::<Result<Vec<_>, _>>()?;

    let count = u16::try_from(images.len())?;
    let header_len = 6 + 16 * images.len();
    let mut ico = Vec::new();
    ico.extend_from_slice(&0u16.to_le_bytes());
    ico.extend_from_slice(&1u16.to_le_bytes());
    ico.extend_from_slice(&count.to_le_bytes());
    let mut offset = u32::try_from(header_len)?;
    for (size, png) in &images {
        // 0 stands for 256 in the one-byte width and height fields.
        let side = u8::try_from(*size).unwrap_or(0);
        let length = u32::try_from(png.len())?;
        ico.extend_from_slice(&[side, side, 0, 0]);
        ico.extend_from_slice(&1u16.to_le_bytes());
        ico.extend_from_slice(&32u16.to_le_bytes());
        ico.extend_from_slice(&length.to_le_bytes());
        ico.extend_from_slice(&offset.to_le_bytes());
        offset += length;
    }
    for (_, png) in images {
        ico.extend_from_slice(&png);
    }
    Ok(ico)
}

fn render_png(tree: &resvg::usvg::Tree, size: u32) -> Result<Vec<u8>, Box<dyn Error>> {
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size).ok_or("taille d'icône nulle")?;
    let scale = f32::from(u16::try_from(size)?) / tree.size().width();
    resvg::render(
        tree,
        resvg::tiny_skia::Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap.encode_png()?)
}
