use std::time::Duration;

use crate::term::Frame;

#[derive(Clone, Debug)]
pub struct Options {
    pub trim_idle: Option<Duration>,
    pub quantize: Duration,
    pub min_frame: Duration,
    pub tail: Duration,
    pub speed: f64,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            trim_idle: Some(Duration::from_millis(1000)),
            quantize: Duration::from_millis(30),
            min_frame: Duration::from_millis(30),
            tail: Duration::from_millis(2000),
            speed: 1.0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Timeline {
    pub frames: Vec<Frame>,
    pub starts: Vec<Duration>,
    pub total: Duration,
}

impl Timeline {
    pub fn len(&self) -> usize {
        self.frames.len()
    }

    pub fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }
}

fn quantize_to(d: Duration, q: Duration) -> Duration {
    if q.is_zero() {
        return d;
    }
    let n = d.as_nanos();
    let step = q.as_nanos();
    let rounded = ((n + step / 2) / step) * step;
    Duration::from_nanos(rounded as u64)
}

pub fn optimize(raw: &[(Duration, Frame)], opts: &Options) -> Timeline {
    if raw.is_empty() {
        return Timeline {
            frames: Vec::new(),
            starts: Vec::new(),
            total: Duration::ZERO,
        };
    }

    let mut deduped: Vec<(Duration, &Frame)> = Vec::with_capacity(raw.len());
    for (at, frame) in raw {
        if deduped.last().map(|(_, f)| *f != frame).unwrap_or(true) {
            deduped.push((*at, frame));
        }
    }

    let speed = if opts.speed > 0.0 { opts.speed } else { 1.0 };
    let mut starts: Vec<Duration> = Vec::with_capacity(deduped.len());
    let mut clock = Duration::ZERO;
    let mut prev_raw = deduped[0].0;

    for (i, (at, _)) in deduped.iter().enumerate() {
        if i > 0 {
            let mut gap = at.saturating_sub(prev_raw);
            if let Some(max) = opts.trim_idle {
                gap = gap.min(max);
            }
            clock += gap.div_f64(speed);
        }
        prev_raw = *at;
        starts.push(clock);
    }

    let mut frames: Vec<Frame> = Vec::with_capacity(deduped.len());
    let mut out_starts: Vec<Duration> = Vec::with_capacity(deduped.len());

    for (i, (_, frame)) in deduped.iter().enumerate() {
        let t = quantize_to(starts[i], opts.quantize);
        match out_starts.last() {
            Some(prev) if t.saturating_sub(*prev) < opts.min_frame && !frames.is_empty() => {
                let last = frames.len() - 1;
                frames[last] = (*frame).clone();
            }
            _ => {
                frames.push((*frame).clone());
                out_starts.push(t);
            }
        }
    }

    let total = out_starts
        .last()
        .copied()
        .unwrap_or(Duration::ZERO)
        .saturating_add(opts.tail.max(opts.min_frame));

    Timeline {
        frames,
        starts: out_starts,
        total,
    }
}
