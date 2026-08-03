pub mod import;
pub mod text;
pub mod theme;

use std::fmt::Write as _;

use crate::optimize::Timeline;
use crate::svg::text::{escape, num, pct};
use crate::svg::theme::{paint, Palette, Theme};
use crate::term::{Color, Frame};

#[derive(Clone, Debug)]
pub struct RenderOpts {
    pub literal: Option<Palette>,
    pub theme: Theme,
    pub font_family: String,
    pub font_size: f64,
    pub advance: f64,
    pub line_height: f64,
    pub padding: f64,
    pub window: bool,
    pub title: String,
    pub cols: u16,
    pub rows: u16,
    pub loop_forever: bool,
}

impl RenderOpts {
    pub fn with_metrics(mut self) -> Self {
        if self.advance <= 0.0 {
            self.advance = self.font_size * 0.6;
        }
        if self.line_height <= 0.0 {
            self.line_height = self.font_size * 1.32;
        }
        self
    }
}

const CHROME_H: f64 = 34.0;

pub fn render(tl: &Timeline, o: &RenderOpts) -> String {
    let content_w = o.cols as f64 * o.advance;
    let content_h = o.rows as f64 * o.line_height;
    let chrome = if o.window { CHROME_H } else { 0.0 };
    let width = content_w + o.padding * 2.0;
    let height = content_h + o.padding * 2.0 + chrome;
    let top = o.padding + chrome;
    let baseline = (o.line_height - o.font_size) * 0.5 + o.font_size * 0.78;

    let mut s = String::with_capacity(64 * 1024);

    let _ = write!(
        s,
        r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="{ff}" font-size="{fs}px">"#,
        w = num(width),
        h = num(height),
        ff = escape(&o.font_family),
        fs = num(o.font_size),
    );

    s.push_str("<style>");
    if o.literal.is_none() {
        vars(&mut s, ":root", &o.theme.dark, &o.theme);
        if o.theme.has_light() {
            s.push_str("@media(prefers-color-scheme:light){");
            vars(&mut s, ":root", o.theme.light(), &o.theme);
            s.push('}');
            s.push_str(":root[data-theme=\"light\"]{");
            inner_vars(&mut s, o.theme.light(), &o.theme);
            s.push('}');
            s.push_str(":root[data-theme=\"dark\"]{");
            inner_vars(&mut s, &o.theme.dark, &o.theme);
            s.push('}');
        }
    }
    s.push_str("text{white-space:pre;font-variant-ligatures:none}");
    s.push_str(".b{font-weight:700}.i{font-style:italic}.u{text-decoration:underline}");
    let _ = write!(s, ".cur{{fill:{};opacity:.7}}", cursor_paint(o));

    if tl.len() > 1 {
        keyframes(&mut s, tl, content_h);
        let _ = write!(
            s,
            ".s{{animation:tsroll {dur}s step-end {rep}}}",
            dur = num(tl.total.as_secs_f64()),
            rep = if o.loop_forever {
                "infinite"
            } else {
                "1 forwards"
            },
        );
    }
    s.push_str("</style>");

    let _ = write!(
        s,
        r#"<rect width="{w}" height="{h}" rx="{r}" fill="{f}"/>"#,
        w = num(width),
        h = num(height),
        r = if o.window { 10 } else { 6 },
        f = paint(Color::Default, "bg", o.literal.as_ref()),
    );

    if o.window {
        chrome_bar(&mut s, o, width);
    }

    let _ = write!(
        s,
        r#"<defs><clipPath id="tsclip"><rect x="{x}" y="{y}" width="{w}" height="{h}"/></clipPath></defs>"#,
        x = num(o.padding),
        y = num(top),
        w = num(content_w),
        h = num(content_h),
    );

    let _ = write!(
        s,
        r#"<g clip-path="url(#tsclip)"><g transform="translate({x},{y})"><g class="s">"#,
        x = num(o.padding),
        y = num(top),
    );

    for (i, frame) in tl.frames.iter().enumerate() {
        let _ = write!(
            s,
            r#"<g transform="translate(0,{y})">"#,
            y = num(i as f64 * content_h)
        );
        emit_frame(&mut s, frame, o, baseline);
        s.push_str("</g>");
    }

    s.push_str("</g></g></g>");

    if o.theme.has_border() {
        let _ = write!(
            s,
            r#"<rect x="0.5" y="0.5" width="{w}" height="{h}" rx="{r}" fill="none" stroke="{c}"/>"#,
            w = num(width - 1.0),
            h = num(height - 1.0),
            r = if o.window { 10 } else { 6 },
            c = slot(o, "border", |p| p.border()),
        );
    }

    s.push_str("</svg>");
    s
}

fn slot(o: &RenderOpts, var: &str, pick: fn(&Palette) -> &str) -> String {
    match &o.literal {
        Some(p) => pick(p).to_string(),
        None => format!("var(--{var})"),
    }
}

fn cursor_paint(o: &RenderOpts) -> String {
    if o.theme.has_cursor() {
        slot(o, "cur", |p| p.cursor())
    } else {
        paint(Color::Default, "fg", o.literal.as_ref())
    }
}

fn vars(s: &mut String, sel: &str, p: &Palette, t: &Theme) {
    let _ = write!(s, "{sel}{{");
    inner_vars(s, p, t);
    s.push('}');
}

fn inner_vars(s: &mut String, p: &Palette, t: &Theme) {
    let _ = write!(s, "--bg:{};--fg:{};", p.bg, p.fg);
    if t.has_cursor() {
        let _ = write!(s, "--cur:{};", p.cursor());
    }
    if t.has_chrome() {
        let _ = write!(s, "--chrome:{};", p.chrome());
    }
    if t.has_border() {
        let _ = write!(s, "--border:{};", p.border());
    }
    if t.has_buttons() {
        for (i, b) in p.buttons().iter().enumerate() {
            let _ = write!(s, "--btn{i}:{b};");
        }
    }
    for (i, c) in p.ansi.iter().enumerate() {
        let _ = write!(s, "--c{i}:{c};");
    }
}

fn keyframes(s: &mut String, tl: &Timeline, frame_h: f64) {
    let total = tl.total.as_secs_f64().max(f64::EPSILON);
    s.push_str("@keyframes tsroll{");
    for (i, start) in tl.starts.iter().enumerate() {
        let p = (start.as_secs_f64() / total) * 100.0;
        let _ = write!(
            s,
            "{}%{{transform:translateY({}px)}}",
            pct(p),
            num(-(i as f64) * frame_h)
        );
    }
    let _ = write!(
        s,
        "100%{{transform:translateY({}px)}}",
        num(-((tl.len().saturating_sub(1)) as f64) * frame_h)
    );
    s.push('}');
}

fn chrome_bar(s: &mut String, o: &RenderOpts, width: f64) {
    if o.theme.has_chrome() {
        let r = 10.0;
        let _ = write!(
            s,
            r#"<path d="M0,{r} A{r},{r} 0 0 1 {r},0 H{a} A{r},{r} 0 0 1 {w},{r} V{h} H0 Z" fill="{f}"/>"#,
            r = num(r),
            a = num(width - r),
            w = num(width),
            h = num(CHROME_H),
            f = slot(o, "chrome", |p| p.chrome()),
        );
    }
    if o.theme.has_border() {
        let _ = write!(
            s,
            r#"<rect x="0" y="{y}" width="{w}" height="1" fill="{f}"/>"#,
            y = num(CHROME_H),
            w = num(width),
            f = slot(o, "border", |p| p.border()),
        );
    }

    let buttons: Vec<String> = o
        .literal
        .as_ref()
        .map(|p| p.buttons().iter().map(|b| b.to_string()).collect())
        .unwrap_or_else(|| {
            if o.theme.has_buttons() {
                (0..3).map(|i| format!("var(--btn{i})")).collect()
            } else {
                theme::BUTTONS.iter().map(|b| b.to_string()).collect()
            }
        });

    for (fill, cx) in buttons.iter().zip([20.0, 38.0, 56.0]) {
        let _ = write!(
            s,
            r#"<circle cx="{cx}" cy="17" r="5.5" fill="{fill}"/>"#,
            cx = num(cx),
        );
    }
    if !o.title.is_empty() {
        let _ = write!(
            s,
            r#"<text x="{x}" y="21" text-anchor="middle" fill="{f}" opacity=".55" font-size="{fs}px">{t}</text>"#,
            x = num(width / 2.0),
            f = paint(Color::Default, "fg", o.literal.as_ref()),
            fs = num(o.font_size * 0.85),
            t = escape(&o.title),
        );
    }
}

fn emit_frame(s: &mut String, frame: &Frame, o: &RenderOpts, baseline: f64) {
    for (r, row) in frame.rows.iter().enumerate() {
        for run in &row.runs {
            if run.style.bg.is_default() {
                continue;
            }
            let _ = write!(
                s,
                r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{f}" shape-rendering="crispEdges"/>"#,
                x = num(run.col as f64 * o.advance),
                y = num(r as f64 * o.line_height),
                w = num(run.width as f64 * o.advance),
                h = num(o.line_height),
                f = paint(run.style.bg, "bg", o.literal.as_ref()),
            );
        }
    }

    if let Some((cr, cc)) = frame.cursor {
        let _ = write!(
            s,
            r#"<rect class="cur" x="{x}" y="{y}" width="{w}" height="{h}"/>"#,
            x = num(cc as f64 * o.advance),
            y = num(cr as f64 * o.line_height),
            w = num(o.advance),
            h = num(o.line_height),
        );
    }

    for (r, row) in frame.rows.iter().enumerate() {
        if row.runs.is_empty() {
            continue;
        }
        let _ = write!(
            s,
            r#"<text y="{y}" xml:space="preserve">"#,
            y = num(r as f64 * o.line_height + baseline)
        );
        for run in &row.runs {
            let mut class = String::new();
            if run.style.bold {
                class.push('b');
            }
            if run.style.italic {
                class.push_str(" i");
            }
            if run.style.underline {
                class.push_str(" u");
            }
            let _ = write!(
                s,
                r#"<tspan x="{x}" fill="{f}""#,
                x = num(run.col as f64 * o.advance),
                f = paint(run.style.fg, "fg", o.literal.as_ref()),
            );
            if !class.is_empty() {
                let _ = write!(s, r#" class="{}""#, class.trim());
            }
            let _ = write!(s, ">{}</tspan>", escape(&run.text));
        }
        s.push_str("</text>");
    }
}
