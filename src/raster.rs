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
        opts.theme.light().clone()
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

pub fn delay_ms(timeline: &Timeline, index: usize) -> u64 {
    let start = timeline.starts[index];
    let end = timeline
        .starts
        .get(index + 1)
        .copied()
        .unwrap_or(timeline.total);
    (end.saturating_sub(start).as_millis() as u64).max(MIN_DELAY_CS as u64 * 10)
}

pub fn frame_at(timeline: &Timeline, at: Duration) -> usize {
    timeline
        .starts
        .iter()
        .rposition(|start| *start <= at)
        .unwrap_or(0)
        .min(timeline.frames.len().saturating_sub(1))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rect {
    pub x: u32,
    pub y: u32,
    pub w: u32,
    pub h: u32,
}

pub fn dirty(old: &[u8], new: &[u8], width: u32, height: u32) -> Rect {
    let stride = width as usize * 4;
    let (mut top, mut bottom) = (None, 0u32);
    let (mut left, mut right) = (width, 0u32);

    for y in 0..height {
        let row = y as usize * stride;
        let a = &old[row..row + stride];
        let b = &new[row..row + stride];
        if a == b {
            continue;
        }
        if top.is_none() {
            top = Some(y);
        }
        bottom = y;

        let first = a
            .chunks_exact(4)
            .zip(b.chunks_exact(4))
            .position(|(p, q)| p != q)
            .unwrap_or(0) as u32;
        let last = a
            .chunks_exact(4)
            .zip(b.chunks_exact(4))
            .rposition(|(p, q)| p != q)
            .unwrap_or(0) as u32;
        left = left.min(first);
        right = right.max(last);
    }

    match top {
        None => Rect {
            x: 0,
            y: 0,
            w: 1,
            h: 1,
        },
        Some(top) => Rect {
            x: left,
            y: top,
            w: right - left + 1,
            h: bottom - top + 1,
        },
    }
}

fn crop(rgba: &[u8], width: u32, patch: Rect) -> Vec<u8> {
    if patch.x == 0 && patch.w == width {
        let start = patch.y as usize * width as usize * 4;
        let len = patch.h as usize * width as usize * 4;
        return rgba[start..start + len].to_vec();
    }

    let stride = width as usize * 4;
    let mut out = Vec::with_capacity(patch.w as usize * patch.h as usize * 4);
    for y in patch.y..patch.y + patch.h {
        let row = y as usize * stride + patch.x as usize * 4;
        out.extend_from_slice(&rgba[row..row + patch.w as usize * 4]);
    }
    out
}

fn probe(timeline: &Timeline, opts: &RenderOpts, scale: f32) -> Result<(u32, u32)> {
    let first = timeline
        .frames
        .first()
        .ok_or_else(|| anyhow!("nothing to render"))?;
    let pixmap = rasterize(
        &render(&still(first), opts),
        scale,
        &options(&opts.font_family),
    )?;
    Ok((pixmap.width(), pixmap.height()))
}

fn each_frame(
    timeline: &Timeline,
    opts: &RenderOpts,
    scale: f32,
    mut sink: impl FnMut(usize, &Pixmap) -> Result<()>,
) -> Result<(u32, u32)> {
    if timeline.frames.is_empty() {
        return Err(anyhow!("nothing to render"));
    }

    let opt = options(&opts.font_family);
    let mut size = None;

    for (i, frame) in timeline.frames.iter().enumerate() {
        let pixmap = rasterize(&render(&still(frame), opts), scale, &opt)?;
        let dims = (pixmap.width(), pixmap.height());
        match size {
            None => size = Some(dims),
            Some(first) if first != dims => {
                return Err(anyhow!("frame {} rasterized to a different size", i + 1))
            }
            _ => {}
        }
        sink(i, &pixmap)?;
    }

    size.ok_or_else(|| anyhow!("nothing to render"))
}

pub fn png(timeline: &Timeline, opts: &RenderOpts, scale: f32, light: bool) -> Result<Vec<u8>> {
    png_at(timeline, opts, scale, light, None)
}

pub fn png_at(
    timeline: &Timeline,
    opts: &RenderOpts,
    scale: f32,
    light: bool,
    at: Option<Duration>,
) -> Result<Vec<u8>> {
    if timeline.frames.is_empty() {
        return Err(anyhow!("nothing to render"));
    }
    let index = match at {
        Some(at) => frame_at(timeline, at),
        None => timeline.frames.len() - 1,
    };
    let opts = flattened(opts, light);
    let svg = render(&still(&timeline.frames[index]), &opts);
    let pixmap = rasterize(&svg, scale, &options(&opts.font_family))?;
    pixmap.encode_png().context("encoding the png")
}

pub fn gif(timeline: &Timeline, opts: &RenderOpts, scale: f32, light: bool) -> Result<Vec<u8>> {
    let opts = flattened(opts, light);
    let (w, h) = probe(timeline, &opts, scale)?;
    let width = u16::try_from(w).context("image is too wide for a gif")?;
    let height = u16::try_from(h).context("image is too tall for a gif")?;

    let mut out = Vec::new();
    {
        let mut encoder =
            gif::Encoder::new(&mut out, width, height, &[]).context("starting the gif")?;
        if opts.loop_forever {
            encoder
                .set_repeat(gif::Repeat::Infinite)
                .context("setting the gif to loop")?;
        }

        each_frame(timeline, &opts, scale, |i, pixmap| {
            let mut rgba = straight_rgba(pixmap);
            let mut encoded = gif::Frame::from_rgba_speed(width, height, &mut rgba, QUANTIZE_SPEED);
            encoded.delay = delay_cs(timeline, i);
            encoder
                .write_frame(&encoded)
                .with_context(|| format!("writing gif frame {}", i + 1))
        })?;
    }

    Ok(out)
}

pub fn webp(timeline: &Timeline, opts: &RenderOpts, scale: f32, light: bool) -> Result<Vec<u8>> {
    let opts = flattened(opts, light);
    let size = probe(timeline, &opts, scale)?;

    let mut config = webp_animation::EncoderOptions::default();
    config.anim_params.loop_count = if opts.loop_forever { 0 } else { 1 };
    let mut encoder = webp_animation::Encoder::new_with_options(size, config)
        .map_err(|e| anyhow!("starting the webp: {e}"))?;

    let mut clock: i32 = 0;
    each_frame(timeline, &opts, scale, |i, pixmap| {
        encoder
            .add_frame(&straight_rgba(pixmap), clock)
            .map_err(|e| anyhow!("writing webp frame {}: {e}", i + 1))?;
        clock += delay_ms(timeline, i) as i32;
        Ok(())
    })?;

    let data = encoder
        .finalize(clock)
        .map_err(|e| anyhow!("finishing the webp: {e}"))?;
    Ok(data.to_vec())
}

pub fn apng(timeline: &Timeline, opts: &RenderOpts, scale: f32, light: bool) -> Result<Vec<u8>> {
    let opts = flattened(opts, light);
    let count = u32::try_from(timeline.frames.len()).context("too many frames for an apng")?;
    if count == 0 {
        return Err(anyhow!("nothing to render"));
    }

    let (w, h) = probe(timeline, &opts, scale)?;

    let mut out = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut out, w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .set_animated(count, if opts.loop_forever { 0 } else { 1 })
            .context("starting the apng")?;
        let mut writer = encoder.write_header().context("writing the apng header")?;
        let mut previous: Option<Vec<u8>> = None;

        each_frame(timeline, &opts, scale, |i, pixmap| {
            let rgba = straight_rgba(pixmap);
            let ms = u16::try_from(delay_ms(timeline, i).min(u16::MAX as u64)).unwrap_or(u16::MAX);
            writer
                .set_frame_delay(ms, 1000)
                .with_context(|| format!("timing apng frame {}", i + 1))?;

            let patch = previous
                .as_ref()
                .map(|old| dirty(old, &rgba, w, h))
                .unwrap_or(Rect {
                    x: 0,
                    y: 0,
                    w,
                    h: h.max(1),
                });

            writer
                .set_frame_position(0, 0)
                .and_then(|_| writer.set_frame_dimension(patch.w, patch.h))
                .and_then(|_| writer.set_frame_position(patch.x, patch.y))
                .and_then(|_| writer.set_blend_op(png::BlendOp::Source))
                .and_then(|_| writer.set_dispose_op(png::DisposeOp::None))
                .with_context(|| format!("placing apng frame {}", i + 1))?;

            writer
                .write_image_data(&crop(&rgba, w, patch))
                .with_context(|| format!("writing apng frame {}", i + 1))?;
            previous = Some(rgba);
            Ok(())
        })?;

        writer.finish().context("finishing the apng")?;
    }

    Ok(out)
}
