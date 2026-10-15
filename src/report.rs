//! One self-contained page: the world's vital signs and its firsts.

use std::fs;
use std::path::Path;

struct Series<'a> {
    label: &'a str,
    var: &'a str,
}

fn chart(title: &str, xs: &[f64], rows: &[Vec<f64>], series: &[Series], idx: &[usize]) -> String {
    let (w, h, pad) = (640.0, 180.0, 28.0);
    let xmax = xs.last().copied().unwrap_or(1.0).max(1.0);
    let mut lo = f64::MAX;
    let mut hi = f64::MIN;
    for r in rows {
        for &i in idx {
            lo = lo.min(r[i]);
            hi = hi.max(r[i]);
        }
    }
    if lo > 0.0 {
        lo = 0.0;
    }
    if hi <= lo {
        hi = lo + 1.0;
    }
    let sx = |x: f64| pad + x / xmax * (w - pad * 2.0);
    let sy = |y: f64| h - pad - (y - lo) / (hi - lo) * (h - pad * 2.0);
    let mut svg = format!(r#"<svg viewBox="0 0 {w} {h}" role="img" aria-label="{title}">"#);
    svg += &format!(r#"<line x1="{pad}" y1="{0}" x2="{1}" y2="{0}" class="axis"/>"#, sy(0.0f64.max(lo)), w - pad);
    svg += &format!(r#"<text x="{pad}" y="14" class="tick">{hi:.0}</text><text x="{pad}" y="{}" class="tick">{lo:.0}</text>"#, h - 8.0);
    svg += &format!(r#"<text x="{}" y="{}" class="tick" text-anchor="end">epoch {xmax:.0}</text>"#, w - pad, h - 8.0);
    for (s, &i) in series.iter().zip(idx) {
        // Long runs: plot at most ~1500 points per line, each the mean of its stretch.
        let stride = xs.len().div_ceil(1500).max(1);
        let pts: Vec<String> = xs
            .chunks(stride)
            .zip(rows.chunks(stride))
            .map(|(xc, rc)| {
                let y = rc.iter().map(|r| r[i]).sum::<f64>() / rc.len() as f64;
                format!("{:.1},{:.1}", sx(xc[xc.len() / 2]), sy(y))
            })
            .collect();
        svg += &format!(r#"<polyline points="{}" fill="none" stroke="var({})" stroke-width="1.6"/>"#, pts.join(" "), s.var);
    }
    svg += "</svg>";
    let legend: String = series
        .iter()
        .map(|s| format!(r#"<span><i style="background:var({})"></i>{}</span>"#, s.var, s.label))
        .collect();
    format!(r#"<section><h2>{title}</h2><div class="legend">{legend}</div>{svg}</section>"#)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn write(dir: &Path) -> Result<(), String> {
    let csv = fs::read_to_string(dir.join("census.csv")).map_err(|e| e.to_string())?;
    let mut lines = csv.lines();
    let header: Vec<&str> = lines.next().unwrap_or("").split(',').collect();
    let col = |name: &str| header.iter().position(|h| *h == name).unwrap_or(0);
    let mut rows: Vec<Vec<f64>> = Vec::new();
    let mut last_dominant = String::new();
    for l in lines {
        let f: Vec<&str> = l.split(',').collect();
        last_dominant = f.last().unwrap_or(&"").to_string();
        rows.push(f.iter().map(|v| v.parse::<f64>().unwrap_or(0.0)).collect());
    }
    if rows.is_empty() {
        return Err("census is empty".into());
    }
    let xs: Vec<f64> = rows.iter().map(|r| r[col("epoch")]).collect();
    let last = rows.last().unwrap();
    let events = fs::read_to_string(dir.join("events.log")).unwrap_or_default();
    let events_html: String = events
        .lines()
        .filter_map(|l| l.split_once(' '))
        .map(|(e, t)| format!("<li><b>epoch {e}</b> {}</li>", esc(t)))
        .collect();

    let s = |label, var| Series { label, var };
    let charts = [
        chart("Population", &xs, &rows, &[s("creatures alive", "--c1")], &[col("pop")]),
        chart(
            "Genotypes alive, real vs. neutral shadow",
            &xs,
            &rows,
            &[s("real", "--c1"), s("shadow (selection by chance)", "--c3")],
            &[col("diversity"), col("shadow_diversity")],
        ),
        chart(
            "Novelty: cumulative adaptive activity beyond chance",
            &xs,
            &rows,
            &[s("excess new activity", "--c2")],
            &[col("excess")],
        ),
        chart(
            "Body size",
            &xs,
            &rows,
            &[s("mean", "--c1"), s("smallest", "--c2"), s("largest", "--c3")],
            &[col("mean_size"), col("min_size"), col("max_size")],
        ),
        chart(
            "Creatures computing on numbers from the outside",
            &xs,
            &rows,
            &[s("any logic task", "--c1"), s("NAND", "--c3"), s("XOR", "--c4"), s("EQU, the hardest", "--c2")],
            &[col("computing"), col("task_nand"), col("task_xor"), col("task_equ")],
        ),
        chart("Births per epoch", &xs, &rows, &[s("births", "--c1")], &[col("births")]),
    ]
    .to_vec();
    let mut charts = charts;
    if header.contains(&"ocean_acc") {
        let pct: Vec<Vec<f64>> = rows
            .iter()
            .map(|r| vec![r[col("ocean_acc")] * 100.0, r[col("ocean_best")] * 100.0])
            .collect();
        charts.push(chart(
            "The ocean: % of bytes predicted before diving",
            &xs,
            &pct,
            &[s("population mean", "--c1"), s("best creature", "--c2")],
            &[0, 1],
        ));
    }
    if header.contains(&"modes_novelty") {
        charts.push(chart(
            "Open-endedness: new lasting phenotypes, all-time",
            &xs,
            &rows,
            &[s("novel persistent phenotypes", "--c2")],
            &[col("modes_novelty")],
        ));
        charts.push(chart(
            "Open-endedness: ecology and complexity of lasting phenotypes",
            &xs,
            &rows,
            &[s("ecology (effective phenotypes)", "--c1"), s("complexity (NAND steps)", "--c4"), s("observer surprise (bits/op)", "--c3")],
            &[col("modes_ecology"), col("modes_complexity"), col("observer_bits")],
        ));
    }
    let charts = charts.join("\n");

    let html = format!(
        r#"<!doctype html>
<html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1">
<title>Abiogenesis Run</title>
<style>
:root {{ --bg:#f6f4ef; --fg:#1d1b17; --mute:#6d685e; --card:#fffdf8; --line:#d9d3c7; --c1:#2a6f97; --c2:#c0392b; --c3:#8a8170; --c4:#3f8f4f; }}
@media (prefers-color-scheme: dark) {{ :root:not([data-theme="light"]) {{ --bg:#0f1113; --fg:#e8e4dc; --mute:#9a958b; --card:#171a1d; --line:#2c3035; --c1:#6cb4e4; --c2:#ff7a66; --c3:#8d877b; --c4:#7fcf8a; }} }}
:root[data-theme="dark"] {{ --bg:#0f1113; --fg:#e8e4dc; --mute:#9a958b; --card:#171a1d; --line:#2c3035; --c1:#6cb4e4; --c2:#ff7a66; --c3:#8d877b; --c4:#7fcf8a; }}
body {{ margin:0; background:var(--bg); color:var(--fg); font:15px/1.5 ui-sans-serif,system-ui,sans-serif; }}
main {{ max-width:720px; margin:0 auto; padding:32px 16px 64px; }}
h1 {{ font-size:28px; margin:0 0 4px; letter-spacing:-0.01em; }}
.sub {{ color:var(--mute); margin:0 0 24px; }}
.stats {{ display:grid; grid-template-columns:repeat(auto-fit,minmax(130px,1fr)); gap:10px; margin-bottom:24px; }}
.stat {{ background:var(--card); border:1px solid var(--line); border-radius:10px; padding:12px; }}
.stat b {{ display:block; font-size:22px; font-variant-numeric:tabular-nums; }}
.stat span {{ color:var(--mute); font-size:13px; }}
section {{ background:var(--card); border:1px solid var(--line); border-radius:10px; padding:14px; margin-bottom:14px; }}
h2 {{ font-size:15px; margin:0 0 6px; }}
svg {{ width:100%; height:auto; display:block; }}
.axis {{ stroke:var(--line); }}
.tick {{ fill:var(--mute); font-size:11px; }}
.legend {{ display:flex; gap:14px; flex-wrap:wrap; color:var(--mute); font-size:13px; margin-bottom:4px; }}
.legend i {{ display:inline-block; width:10px; height:3px; margin-right:6px; vertical-align:middle; }}
ol {{ padding-left:18px; margin:0; }} li {{ margin:4px 0; }} li b {{ color:var(--mute); font-weight:500; margin-right:6px; }}
</style></head><body><main>
<h1>Abiogenesis</h1>
<p class="sub">One run, {epochs} epochs. Dominant genotype at the end: {dom}.</p>
<div class="stats">
<div class="stat"><b>{pop}</b><span>alive</span></div>
<div class="stat"><b>{div}</b><span>genotypes</span></div>
<div class="stat"><b>{sizes}</b><span>body sizes</span></div>
<div class="stat"><b>{computing}</b><span>computing</span></div>
<div class="stat"><b>{excess:+}</b><span>novelty beyond chance</span></div>
</div>
{charts}
<section><h2>Firsts</h2><ol>{events_html}</ol></section>
</main></body></html>"#,
        epochs = rows.len(),
        dom = esc(&last_dominant),
        pop = last[col("pop")],
        div = last[col("diversity")],
        sizes = last[col("size_classes")],
        excess = last[col("excess")] as i64,
        computing = last[col("computing")],
    );
    fs::write(dir.join("report.html"), html).map_err(|e| e.to_string())
}
