//! `compare A B`: judges an experiment, paired seed by seed, on the criterion fixed before
//! any run. A pair is won when A's normalised cumulative activity is still growing (last
//! quarter's mean above the second quarter's) and above B's, and A's new adaptively
//! significant components keep coming (second half at least half the first) and outnumber
//! B's. Eight wins in ten is a WIN.
//!
//! It judges twice when both arms have the columns: on whole genotypes (`norm_*`, the
//! fixed rule) and on genes (`gene_*`, Channon's own choice of component in Geb).

use std::fs;
use std::path::{Path, PathBuf};

struct Series {
    epochs: usize,
    final_pop: u64,
    final_div: u64,
    q2_cum: f64,
    q4_cum: f64,
    new_h1: u64,
    new_h2: u64,
    ledger_bad: bool,
}

impl Series {
    fn growing(&self) -> bool {
        self.q4_cum > self.q2_cum
    }
    fn sustained(&self) -> bool {
        self.new_h1 > 0 && self.new_h2 * 2 >= self.new_h1
    }
    fn extinct(&self) -> bool {
        self.final_pop == 0
    }
}

fn load(dir: &Path, cum_col: &str, new_col: &str) -> Result<Series, String> {
    let raw = fs::read(dir.join("census.csv")).map_err(|e| format!("{}: {e}", dir.display()))?;
    let text: String = String::from_utf8_lossy(&raw).chars().filter(|&c| c != '\0').collect();
    let mut lines = text.lines();
    let head: Vec<&str> = lines.next().ok_or("empty census")?.split(',').collect();
    let col = |n: &str| head.iter().position(|h| *h == n).ok_or(format!("no column {n}"));
    let (pop, div, cum, new, gap) = (col("pop")?, col("diversity")?, col(cum_col)?, col(new_col)?, col("ledger_gap"));
    let rows: Vec<Vec<f64>> = lines
        .map(|l| l.split(',').map(|x| x.parse().unwrap_or(0.0)).collect::<Vec<f64>>())
        .filter(|r| r.len() == head.len())
        .collect();
    let n = rows.len();
    if n < 8 {
        return Err(format!("{}: only {n} rows", dir.display()));
    }
    let mean = |a: usize, b: usize| rows[a..b].iter().map(|r| r[cum]).sum::<f64>() / (b - a) as f64;
    let sum = |a: usize, b: usize| rows[a..b].iter().map(|r| r[new] as u64).sum::<u64>();
    let last = &rows[n - 1];
    Ok(Series {
        epochs: n,
        final_pop: last[pop] as u64,
        final_div: last[div] as u64,
        q2_cum: mean(n / 4, n / 2),
        q4_cum: mean(3 * n / 4, n),
        new_h1: sum(0, n / 2),
        new_h2: sum(n / 2, n),
        ledger_bad: gap.map(|g| rows.iter().any(|r| r[g] != 0.0)).unwrap_or(false),
    })
}

/// Seeds present under both prefixes: `runs/X` matches `runs/X_1`, `runs/X_2`, ...
fn seeds(prefix: &str) -> Vec<(u64, PathBuf)> {
    let p = Path::new(prefix);
    let parent = p.parent().filter(|d| !d.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let stem = format!("{}_", p.file_name().and_then(|s| s.to_str()).unwrap_or(""));
    let mut out: Vec<(u64, PathBuf)> = fs::read_dir(parent)
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|e| {
            let name = e.file_name().into_string().ok()?;
            let seed = name.strip_prefix(&stem)?.parse().ok()?;
            e.path().join("census.csv").exists().then_some((seed, e.path()))
        })
        .collect();
    out.sort();
    out
}

/// One-sided sign test: chance of at least `k` wins in `n` fair coin flips.
fn sign_p(k: usize, n: usize) -> f64 {
    let choose = |n: usize, k: usize| (0..k).fold(1.0, |a, i| a * (n - i) as f64 / (i + 1) as f64);
    (k..=n).map(|i| choose(n, i)).sum::<f64>() / 2f64.powi(n as i32)
}

pub fn compare(a: &str, b: &str) -> Result<(), String> {
    judge(a, b, "genotypes", "norm_cum", "norm_new")?;
    let has_genes = |p: &str| {
        seeds(p).first().and_then(|(_, d)| fs::read_to_string(d.join("census.csv")).ok()).is_some_and(|t| t.lines().next().unwrap_or("").contains("gene_cum"))
    };
    if has_genes(a) && has_genes(b) {
        println!();
        judge(a, b, "genes", "gene_cum", "gene_new")?;
    }
    Ok(())
}

fn judge(a: &str, b: &str, unit: &str, cum_col: &str, new_col: &str) -> Result<(), String> {
    let (sa, sb) = (seeds(a), seeds(b));
    let pairs: Vec<(u64, &PathBuf, &PathBuf)> =
        sa.iter().filter_map(|(s, pa)| sb.iter().find(|(t, _)| t == s).map(|(_, pb)| (*s, pa, pb))).collect();
    if pairs.is_empty() {
        return Err(format!("no paired seeds for {a} and {b}"));
    }
    println!("A = {a}, B = {b}, components = {unit}");
    println!("seed | A: epochs pop div  q2_cum  q4_cum  new h1/h2 | B: epochs pop div  q2_cum  q4_cum  new h1/h2 | grow sust | win");
    let mut wins = 0;
    let mut warn = Vec::new();
    for (seed, pa, pb) in &pairs {
        let (x, y) = (load(pa, cum_col, new_col)?, load(pb, cum_col, new_col)?);
        let grow = x.growing() && x.q4_cum > y.q4_cum;
        let sust = x.sustained() && x.new_h2 > y.new_h2;
        let win = grow && sust && !x.extinct();
        wins += win as usize;
        let f = |s: &Series| {
            format!(
                "{:>6} {:>4} {:>4} {:>7.1} {:>7.1} {:>4}/{:<4}",
                s.epochs, s.final_pop, s.final_div, s.q2_cum, s.q4_cum, s.new_h1, s.new_h2
            )
        };
        println!("{seed:>4} | {} | {} | {:>4} {:>4} | {}", f(&x), f(&y), yn(grow), yn(sust), if win { "WIN" } else { "-" });
        for (s, tag) in [(&x, "A"), (&y, "B")] {
            if s.extinct() {
                warn.push(format!("seed {seed} {tag}: extinct"));
            }
            if s.ledger_bad {
                warn.push(format!("seed {seed} {tag}: ledger gap nonzero"));
            }
            if s.final_pop > 50 && s.final_div * 10 >= s.final_pop * 9 {
                warn.push(format!("seed {seed} {tag}: {} genotypes in {} creatures (meltdown?)", s.final_div, s.final_pop));
            }
        }
        if x.epochs != y.epochs {
            warn.push(format!("seed {seed}: unequal lengths {} vs {}", x.epochs, y.epochs));
        }
    }
    for w in &warn {
        println!("  ! {w}");
    }
    let n = pairs.len();
    let p = sign_p(wins, n);
    let verdict = if n >= 10 && wins * 10 >= n * 8 { "WIN" } else { "FAIL" };
    println!("verdict ({unit}): {wins}/{n} pairs, sign-test p = {p:.3}: {verdict}");
    Ok(())
}

fn yn(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}
