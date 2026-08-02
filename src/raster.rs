use std::sync::Arc;
use std::time::Duration;

use anyhow::{anyhow, Context, Result};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg;

use crate::optimize::Timeline;
use crate::svg::{render, RenderOpts};
use crate::term::Frame;

pub const MIN_DELAY_CS: u16 = 2;
pub const QUANTIZE_SPEED: i32 = 10;

fn still(frame: &Frame) -> Timeline {
    Timeline {
        frames: vec![frame.clone()],
        starts: vec![Duration::ZERO],
        total: Duration::from_millis(1),
    }
}

fn flattened(opts: &RenderOpts, light: bool) -> RenderOpts {
    let mut out = opts.clone();
    out.literal = Some(if light {
        opts.theme.light.clone()
    } else {
        opts.theme.dark.clone()
    });
    out
}

fn options(family: &str) -> usvg::Options<'static> {
    let mut db = usvg::fontdb::Database::new();
    db.load_system_fonts();

    let mut opt = usvg::Options::default();
    for name in family.split(',') {
        let name = name.trim().trim_matches('\'').trim_matches('"');
        if name.is_empty() || name.eq_ignore_ascii_case("monospace") {
            continue;
        }
        let found = db
            .query(&usvg::fontdb::Query {
                families: &[usvg::fontdb::Family::Name(name)],
                ..Default::default()
            })
            .is_some();
        if found {
            opt.font_family = name.to_string();
            break;
        }
    }
    db.set_monospace_family(opt.font_family.clone());
    opt.fontdb = Arc::new(db);
    opt
}

fn rasterize(svg: &str, scale: f32, opt: &usvg::Options) -> Result<Pixmap> {
    let tree = usvg::Tree::from_str(svg, opt).context("parsing the generated svg")?;
    let size = tree
        .size()
        .to_int_size()
        .scale_by(scale)
        .ok_or_else(|| anyhow!("scale {scale} produces an empty image"))?;

    let mut pixmap = Pixmap::new(size.width(), size.height()).ok_or_else(|| {
        anyhow!(
            "could not allocate a {}x{} bitmap",
            size.width(),
            size.height()
        )
    })?;

    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

fn straight_rgba(pixmap: &Pixmap) -> Vec<u8> {
    let mut out = Vec::with_capacity(pixmap.data().len());
    for pixel in pixmap.pixels() {
        let c = pixel.demultiply();
        out.extend_from_slice(&[c.red(), c.green(), c.blue(), c.alpha()]);
    }
    out
}

pub fn delay_cs(timeline: &Timeline, index: usize) -> u16 {
    let start = timeline.starts[index];
    let end = timeline
        .starts
        .get(index + 1)
        .copied()
        .unwrap_or(timeline.total);
    let ms = end.saturating_sub(start).as_millis() as u64;
    let cs = ms.div_ceil(10) as u16;
    cs.max(MIN_DELAY_CS)
}

pub fn png(timeline: &Timeline, opts: &RenderOpts, scale: f32, light: bool) -> Result<Vec<u8>> {
    let frame = timeline
        .frames
        .last()
        .ok_or_else(|| anyhow!("nothing to render"))?;
    let opts = flattened(opts, light);
    let svg = render(&still(frame), &opts);
    let pixmap = rasterize(&svg, scale, &options(&opts.font_family))?;
    pixmap.encode_png().context("encoding the png")
}

pub fn gif(timeline: &Timeline, opts: &RenderOpts, scale: f32, light: bool) -> Result<Vec<u8>> {
    if timeline.frames.is_empty() {
        return Err(anyhow!("nothing to render"));
    }

    let opts = flattened(opts, light);
    let opt = options(&opts.font_family);
    let first = rasterize(&render(&still(&timeline.frames[0]), &opts), scale, &opt)?;
    let width = u16::try_from(first.width()).context("image is too wide for a gif")?;
    let height = u16::try_from(first.height()).context("image is too tall for a gif")?;

    let mut out = Vec::new();
    {
        let mut encoder =
            gif::Encoder::new(&mut out, width, height, &[]).context("starting the gif")?;
        if opts.loop_forever {
            encoder
                .set_repeat(gif::Repeat::Infinite)
                .context("setting the gif to loop")?;
        }

        for (i, frame) in timeline.frames.iter().enumerate() {
            let pixmap = if i == 0 {
                first.clone()
            } else {
                rasterize(&render(&still(frame), &opts), scale, &opt)?
            };
            let mut rgba = straight_rgba(&pixmap);
            let mut encoded = gif::Frame::from_rgba_speed(width, height, &mut rgba, QUANTIZE_SPEED);
            encoded.delay = delay_cs(timeline, i);
            encoder
                .write_frame(&encoded)
                .with_context(|| format!("writing gif frame {}", i + 1))?;
        }
    }

    Ok(out)
}
