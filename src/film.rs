//! Frames for the viewer (`viewer/index.html`). With `ABIO_FRAMES=DIR` set, every
//! `ABIO_FRAME_EVERY` epochs (default 1) a run, serve or verify writes `DIR/NNNNNNNN.abf`
//! and then `DIR/latest`. It only reads the world, so hashes are unchanged.
//!
//! Frame layout, little-endian: "ABF1", epoch u32, soup_bits u32, creature count u32,
//! 12 stats as f32 (pop, diversity, mean_size, births, deaths, gene_alive, gene_cum,
//! new_total, null_total, divers, ocean_bits, streams_used), then per creature
//! start u32, size u16, genotype low bits u16, flags u8.
//!
//! The same stats are appended to `DIR/stats.csv` so the viewer can draw the history
//! without fetching every frame. new_total and null_total count new significant genes
//! (real, and the null shadow's) over every epoch since the start, not only frame epochs. A replay appends again; readers keep the last row per epoch.

use crate::meter::Census;
use crate::world::World;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct Film {
    dir: PathBuf,
    every: u64,
    new_total: u64,
    null_total: u64,
}

impl Film {
    pub fn from_env() -> Option<Film> {
        let dir = PathBuf::from(std::env::var_os("ABIO_FRAMES")?);
        fs::create_dir_all(&dir).ok()?;
        let every = std::env::var("ABIO_FRAME_EVERY").ok().and_then(|v| v.parse().ok()).unwrap_or(1u64).max(1);
        Some(Film { dir, every, new_total: 0, null_total: 0 })
    }

    pub fn frame(&mut self, epoch: u64, w: &World, c: &Census) {
        self.new_total += c.genes.new as u64;
        self.null_total += c.genes.null_new as u64;
        if epoch % self.every != 0 {
            return;
        }
        let spans = w.frame_spans();
        let mut b = Vec::with_capacity(64 + spans.len() * 9);
        b.extend_from_slice(b"ABF1");
        for v in [epoch as u32, w.n.trailing_zeros(), spans.len() as u32] {
            b.extend_from_slice(&v.to_le_bytes());
        }
        let g = &c.genes;
        let stats = [
            c.pop as f32, c.diversity as f32, c.mean_size as f32, c.births as f32, c.deaths as f32,
            g.alive as f32, g.cum as f32, self.new_total as f32, self.null_total as f32, c.divers as f32, c.ocean_bits as f32,
            w.streams_used() as f32,
        ];
        for v in stats {
            b.extend_from_slice(&v.to_le_bytes());
        }
        for (start, size, geno, flags) in spans {
            b.extend_from_slice(&start.to_le_bytes());
            b.extend_from_slice(&size.to_le_bytes());
            b.extend_from_slice(&geno.to_le_bytes());
            b.push(flags);
        }
        let line = format!("{epoch},{}\n", stats.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
        let csv = self.dir.join("stats.csv");
        let fresh = !csv.exists();
        if let Ok(mut f) = fs::OpenOptions::new().create(true).append(true).open(&csv) {
            if fresh {
                f.write_all(b"epoch,pop,diversity,mean_size,births,deaths,gene_alive,gene_cum,new_total,null_total,divers,ocean_bits,streams_used\n").ok();
            }
            f.write_all(line.as_bytes()).ok();
        }
        let path = self.dir.join(format!("{epoch:08}.abf"));
        let tmp = self.dir.join("frame.tmp");
        if fs::write(&tmp, &b).is_ok() && fs::rename(&tmp, &path).is_ok() {
            let lt = self.dir.join("latest.tmp");
            if fs::write(&lt, epoch.to_string()).is_ok() {
                fs::rename(&lt, self.dir.join("latest")).ok();
            }
        }
    }
}
