mod feed;
mod film;
mod isa;
mod meter;
mod modes;
mod analyze;
mod channon;
mod genes;
mod prof;
mod report;
mod rng;
mod world;

use feed::{hex, Feed};
use meter::Meter;
use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::time::Instant;
use world::{Params, World};

const USAGE: &str = "abiogenesis

  run     [--out DIR] [--epochs N] [--epoch-instr N] [--seed N] [--soup-bits N] [--feed SPEC] [--sun-bonus N] [--indel N] [--no-tasks] [--ancestor FILE.tasm] [--ocean FILE|genomes] [--ocean-div N] [--max-age N] [--no-memory] [--no-infect] [--reef] [--max-size N] [--economy energy] [--regime feudal|capitalist|evolved] [--social] [--all-ops] [--learned-mut PCT] [--v4] [--mem2] [--energy-total N] [--streams K]
  serve   DIR [run options] [--snapshot-every N]   run forever, resumable
  islands N [run options] [--migrate-every E] [--migrate-count K]
  verify  DIR       replay a run from its seed and feed, check every epoch hash
  report  DIR       rebuild report.html from a run's census
  compare A B       judge runs A_1..A_10 against controls B_1..B_10 (Channon class 3, 8/10)
  ancestor          print the seed replicator

feeds: chain[:text]  file:PATH  cmd:COMMAND  replay:FEED_LOG";

#[derive(Clone)]
struct Config {
    seed: u64,
    epochs: u64,
    epoch_instr: u64,
    warmup: u64,
    feed: String,
    /// Source of the seed creature. Saved next to the run as ancestor.tasm.
    ancestor: String,
    /// Path of the ocean file, empty for none. Its hash is checked on replay.
    ocean: String,
    /// Islands: every `migrate_every` epochs, send `migrate_count` migrants on. 0 = alone.
    migrate_every: u64,
    migrate_count: u64,
    p: Params,
}

impl Config {
    fn to_text(&self) -> String {
        let p = &self.p;
        format!(
            "seed={}\nepochs={}\nepoch_instr={}\nwarmup={}\nfeed={}\nsoup_bits={}\ncopy_error={}\ncosmic_base={}\nfill={}\nmin_size={}\nmax_size={}\nsearch_limit={}\nsun_bonus={}\ntasks={}\nindel={}\nmax_merit={}\nmax_age={}\nocean={}\nocean_hash={}\nocean_div={}\nocean_cap={}\nmemory={}\ninfect={}\ninfect_radius={}\nreef={}\nmigrate_every={}\nmigrate_count={}\nenergy={}\nenergy_total={}\nregime={}\nlean={}\nsocial={}\nlearned_mut={}\nv4={}\nmem2={}\nstreams={}\nops={}\n",
            self.seed, self.epochs, self.epoch_instr, self.warmup, self.feed, p.soup_bits, p.copy_error,
            p.cosmic_base, p.fill, p.min_size, p.max_size, p.search_limit, p.sun_bonus, p.tasks as u8, p.indel, p.max_merit, p.max_age, self.ocean, ocean_hash(&self.ocean), p.ocean_div, p.ocean_cap, p.memory as u8, p.infect as u8, p.infect_radius, p.reef as u8, self.migrate_every, self.migrate_count, p.energy as u8, p.energy_total, match p.regime { world::Regime::Commons => 0, world::Regime::Feudal => 1, world::Regime::Capitalist => 2, world::Regime::Evolved => 3 },
            p.lean as u8, p.social as u8, p.learned_mut, p.v4 as u8, p.mem2 as u8, p.streams,
            world::alphabet(p, !self.ocean.is_empty()).iter().map(|&o| isa::NAMES[o as usize]).collect::<Vec<_>>().join(" ")
        )
    }

    fn from_text(t: &str) -> Result<Config, String> {
        let kv: BTreeMap<&str, &str> = t.lines().filter_map(|l| l.split_once('=')).collect();
        let get = |k: &str| kv.get(k).copied().ok_or(format!("config is missing {k}"));
        let num = |k: &str| get(k).and_then(|v| v.parse::<u64>().map_err(|e| format!("{k}: {e}")));
        Ok(Config {
            seed: num("seed")?,
            epochs: num("epochs")?,
            epoch_instr: num("epoch_instr")?,
            warmup: num("warmup")?,
            feed: get("feed")?.to_string(),
            ancestor: String::new(),
            ocean: get("ocean")?.to_string(),
            migrate_every: num("migrate_every").unwrap_or(0),
            migrate_count: num("migrate_count").unwrap_or(0),
            p: Params {
                soup_bits: num("soup_bits")? as u32,
                copy_error: num("copy_error")?,
                cosmic_base: num("cosmic_base")?,
                fill: get("fill")?.parse().map_err(|e| format!("fill: {e}"))?,
                min_size: num("min_size")? as u32,
                max_size: num("max_size")? as u32,
                search_limit: num("search_limit")? as u32,
                sun_bonus: num("sun_bonus")?,
                tasks: num("tasks")? != 0,
                indel: num("indel")?,
                max_merit: num("max_merit")? as u32,
                max_age: num("max_age")?,
                ocean_div: num("ocean_div")? as u32,
                ocean_cap: num("ocean_cap")? as u32,
                memory: num("memory")? != 0,
                infect: num("infect").map(|v| v != 0).unwrap_or(false),
                infect_radius: num("infect_radius").unwrap_or(2048) as u32,
                reef: num("reef").map(|v| v != 0).unwrap_or(false),
                energy: num("energy").map(|v| v != 0).unwrap_or(false),
                energy_total: num("energy_total").unwrap_or(64_000_000),
                regime: match num("regime").unwrap_or(0) { 1 => world::Regime::Feudal, 2 => world::Regime::Capitalist, 3 => world::Regime::Evolved, _ => world::Regime::Commons },
                // Runs from before the lean core have no `lean` key and used every op.
                lean: num("lean").map(|v| v != 0).unwrap_or(false),
                social: num("social").map(|v| v != 0).unwrap_or(false),
                genome_ocean: get("ocean")? == "genomes",
                learned_mut: num("learned_mut").unwrap_or(0) as u32,
                v4: num("v4").map(|v| v != 0).unwrap_or(false),
                mem2: num("mem2").map(|v| v != 0).unwrap_or(false),
                streams: num("streams").unwrap_or(0) as u32,
            },
        })
    }
}

struct Sinks {
    census: File,
    events: File,
    feed: File,
    chain: File,
    migrants: Option<File>,
}

/// How an island exchanges migrants: live channels to its neighbours, or a log replayed.
enum Links {
    Live { tx: Sender<Vec<Vec<u8>>>, rx: Receiver<Vec<Vec<u8>>> },
    Replay(BTreeMap<u64, Vec<Vec<u8>>>),
}

/// Everything needed to carry on exactly where a run stopped. The jump-table cache and
/// the ocean bytes are not saved: one is rebuilt, the other re-read and hash-checked.
#[derive(serde::Serialize)]
struct SnapshotRef<'a> {
    next_epoch: u64,
    prev: [u8; 32],
    world: &'a World,
    meter: &'a Meter,
    feed: &'a Feed,
}

#[derive(serde::Deserialize)]
struct Snapshot {
    next_epoch: u64,
    prev: [u8; 32],
    world: World,
    meter: Meter,
    feed: Feed,
}

fn save_snapshot(dir: &Path, s: &SnapshotRef) -> Result<(), String> {
    let bytes = bincode::serialize(s).map_err(|e| e.to_string())?;
    let tmp = dir.join("snapshot.tmp");
    fs::write(&tmp, bytes).map_err(|e| e.to_string())?;
    fs::rename(&tmp, dir.join("snapshot.bin")).map_err(|e| e.to_string())
}

fn load_snapshot(dir: &Path) -> Option<Snapshot> {
    bincode::deserialize(&fs::read(dir.join("snapshot.bin")).ok()?).ok()
}

/// Where and how often to snapshot, and the snapshot to resume from, if any.
struct Checkpoint<'a> {
    dir: &'a Path,
    every: u64,
    resume: Option<Snapshot>,
}

/// Run a world. With sinks it records everything; without, it just returns the epoch hashes.
fn simulate(
    cfg: &Config,
    feed: Feed,
    mut sinks: Option<Sinks>,
    loud: bool,
    links: Option<Links>,
    mut checkpoint: Option<Checkpoint>,
) -> Result<(Vec<[u8; 32]>, World, Meter), String> {
    let ocean = if cfg.ocean.is_empty() || cfg.p.genome_ocean { Vec::new() } else { fs::read(&cfg.ocean).map_err(|e| format!("{}: {e}", cfg.ocean))? };
    let resume = checkpoint.as_mut().and_then(|c| c.resume.take());
    let resumed = resume.is_some();
    let (mut w, mut m, mut feed, mut prev, e0) = match resume {
        Some(s) => {
            let mut w = s.world;
            w.after_load(ocean);
            (w, s.meter, s.feed, s.prev, s.next_epoch)
        }
        None => {
            let mut w = World::new(cfg.p.clone(), cfg.seed, ocean);
            let ancestor = isa::assemble(&cfg.ancestor)?;
            let g = w.inject(&ancestor, w.n / 2);
            let m = Meter::new(g, ancestor.len() as u32, cfg.seed, cfg.warmup);
            (w, m, feed, [0u8; 32], 0)
        }
    };
    let mut chain = Vec::new();
    if let Some(s) = sinks.as_mut().filter(|_| !resumed) {
        writeln!(s.census, "epoch,instr,pop,diversity,size_classes,mean_size,min_size,max_size,mean_activity,new_activity,shadow_diversity,shadow_new_activity,excess,births,deaths,sun_center,sun_half,cosmic_interval,goal,computing,mean_merit,task_not,task_nand,task_and,task_orn,task_or,task_andn,task_nor,task_xor,task_equ,task_echo,scratch,mother_scratch,lost_resize,lost_indel,lost_point,gained,resized_births,divers,ocean_acc,ocean_best,ocean_bits,modes_change,modes_novelty,modes_complexity,modes_ecology,observer_bits,infections,reef_emits,reef_reads,kin_reads,norm_cum,norm_median,norm_new,pool,ledger_gap,gini,top_share,rent,evictions,enclosures,revolts_won,revolts_lost,gifts,gifted,titled,mean_levy,matings,lessons,joins,groups,largest_group,died_old,died_starved,died_reaped,gene_alive,gene_cum,gene_median,gene_new,gene_null_new,streams_used,dominant").ok();
    }
    let mut film = film::Film::from_env();
    let t0 = Instant::now();
    for e in e0..cfg.epochs {
        let fb = feed.next()?;
        w.set_weather(&fb);
        if w.p.genome_ocean {
            w.refresh_genome_ocean();
        }
        prof::time(prof::RUN, || w.run(cfg.epoch_instr));
        let (c, notes) = prof::time(prof::CENSUS, || m.census(&mut w));
        if let Some(f) = film.as_mut() {
            f.frame(e, &w, &c);
        }
        if std::env::var_os("ABIO_AUDIT").is_some() {
            let (u, o) = w.audit();
            if u != o {
                eprintln!("AUDIT epoch {e}: used {u} but {o} cells owned, alive {}", w.alive);
            }
        }

        if cfg.migrate_every > 0 && (e + 1) % cfg.migrate_every == 0 {
            let out = w.emigrants(cfg.migrate_count as usize);
            let incoming = match &links {
                Some(Links::Live { tx, rx }) => {
                    tx.send(out).ok();
                    rx.recv().unwrap_or_default()
                }
                Some(Links::Replay(log)) => log.get(&e).cloned().unwrap_or_default(),
                None => Vec::new(),
            };
            for g in &incoming {
                w.immigrate(g);
                if let Some(f) = sinks.as_mut().and_then(|s| s.migrants.as_mut()) {
                    writeln!(f, "{e} {}", hex(g)).ok();
                }
            }
        }

        // The money-safety audit: in the energy economy, balances plus the commons must
        // always equal the total. Any gap means energy was minted or destroyed.
        let (gini, top) = w.wealth();
        let titled = w.titled();
        let groups = w.alliances();
        let gap = if cfg.p.energy { w.ledger() as i64 - cfg.p.energy_total as i64 } else { 0 };
        if gap != 0 {
            if let Some(s) = sinks.as_mut() {
                writeln!(s.events, "{e} LEDGER BROKEN: off by {gap}").ok();
            }
        }
        let mut h = blake3::Hasher::new();
        h.update(&prev);
        h.update(&e.to_le_bytes());
        h.update(&fb);
        h.update(&prof::time(prof::STATE_HASH, || w.state_hash()));
        prev = *h.finalize().as_bytes();
        chain.push(prev);

        if let Some(s) = sinks.as_mut() {
            let wx = w.weather;
            writeln!(
                s.census,
                "{e},{},{},{},{},{:.2},{},{},{:.2},{},{},{},{},{},{},{},{},{},{},{},{:.2},{},{:.2},{:.2},{},{},{},{},{},{},{},{:.4},{:.4},{:.4},{},{},{},{:.2},{:.3},{},{},{},{},{},{},{},{},{},{:.3},{:.3},{},{},{},{},{},{},{},{},{:.1},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
                w.instr, c.pop, c.diversity, c.size_classes, c.mean_size, c.min_size, c.max_size, c.mean_activity,
                c.new_activity, c.shadow_diversity, c.shadow_new_activity, c.excess, c.births, c.deaths,
                wx.sun_center, wx.sun_half, wx.cosmic_interval, wx.goal, c.computing, c.mean_merit,
                c.tasks.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(","),
                c.scratch, c.stats.mother_nopx as f64 / c.stats.births.max(1) as f64, c.stats.lost_resize, c.stats.lost_indel,
                c.stats.lost_point, c.stats.gained, c.stats.resized, c.divers, c.ocean_acc, c.ocean_best, c.ocean_bits, c.modes.change, c.modes.novelty, c.modes.complexity, c.modes.ecology, c.modes.observer_bits, c.stats.infections, c.stats.emits, c.stats.reef_reads, c.stats.kin_reads, c.norm.cum, c.norm.median, c.norm.new, w.pool, gap, gini, top, c.stats.rent, c.stats.evictions, c.stats.enclosures, c.stats.revolts_won, c.stats.revolts_lost, c.stats.gifts, c.stats.gifted, titled.0, titled.1, c.stats.matings, c.stats.lessons, c.stats.joins, groups.0, groups.1, c.stats.died_old, c.stats.died_starved, c.stats.died_reaped, c.genes.alive, c.genes.cum, c.genes.median, c.genes.new, c.genes.null_new, w.streams_used(), c.dominant
            )
            .ok();
            writeln!(s.feed, "{e} {}", hex(&fb)).ok();
            writeln!(s.chain, "{e} {}", hex(&prev)).ok();
            for n in &notes {
                writeln!(s.events, "{e} {n}").ok();
            }
        }
        if let Some(cp) = checkpoint.as_ref().filter(|c| c.every > 0 && (e + 1) % c.every == 0) {
            if let Some(s) = sinks.as_mut() {
                for f in [&mut s.census, &mut s.events, &mut s.feed, &mut s.chain] {
                    f.flush().ok();
                }
            }
            save_snapshot(cp.dir, &SnapshotRef { next_epoch: e + 1, prev, world: &w, meter: &m, feed: &feed })?;
            report::write(cp.dir).ok();
        }
        if loud {
            for n in &notes {
                println!("  [epoch {e}] {n}");
            }
            if e % 25 == 0 || e + 1 == cfg.epochs {
                let rate = w.instr as f64 / t0.elapsed().as_secs_f64() / 1e6;
                println!(
                    "epoch {e:>5}  pop {:>5}  genotypes {:>5}  sizes {}-{}  computing {:>5}  novelty {:>+5}  dominant {}  [{rate:.0}M instr/s]",
                    c.pop, c.diversity, c.min_size, c.max_size, c.computing, c.excess, c.dominant
                );
            }
        }
        if w.alive == 0 && cfg.migrate_every == 0 {
            if loud {
                println!("  [epoch {e}] extinction. the world is empty.");
            }
            if let Some(s) = sinks.as_mut() {
                writeln!(s.events, "{e} extinction").ok();
            }
            break;
        }
    }
    prof::report(t0.elapsed().as_secs_f64());
    Ok((chain, w, m))
}

/// blake3 of the ocean file, so a replay can prove it swam the same sea.
fn ocean_hash(path: &str) -> String {
    if path.is_empty() || path == "genomes" {
        return path.to_string();
    }
    fs::read(path).map(|b| hex(blake3::hash(&b).as_bytes())).unwrap_or_default()
}

fn arg_val<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.iter().position(|a| a == key).and_then(|i| args.get(i + 1)).map(|s| s.as_str())
}

fn cmd_run(args: &[String]) -> Result<(), String> {
    let cfg = config_from_args(args)?;
    let out = PathBuf::from(arg_val(args, "--out").map(String::from).unwrap_or(format!("runs/seed{}", cfg.seed)));
    println!("abiogenesis: soup {} cells, {} epochs of {} instructions, feed {}", 1u64 << cfg.p.soup_bits, cfg.epochs, cfg.epoch_instr, cfg.feed);
    run_one(&cfg, &out, true, None)
}

/// Run one world (or one island) into `out`: config, logs, genebank, report.
fn run_one(cfg: &Config, out: &Path, loud: bool, links: Option<Links>) -> Result<(), String> {
    fs::create_dir_all(out).map_err(|e| e.to_string())?;
    fs::write(out.join("config.txt"), cfg.to_text()).map_err(|e| e.to_string())?;
    fs::write(out.join("ancestor.tasm"), &cfg.ancestor).map_err(|e| e.to_string())?;
    let mk = |n: &str| File::create(out.join(n)).map_err(|e| e.to_string());
    let sinks = Sinks {
        census: mk("census.csv")?,
        events: mk("events.log")?,
        feed: mk("feed.log")?,
        chain: mk("chain.log")?,
        migrants: if links.is_some() { Some(mk("migrants.log")?) } else { None },
    };
    let (_, w, m) = simulate(cfg, Feed::parse(&cfg.feed, cfg.seed)?, Some(sinks), loud, links, None)?;
    write_genebank(out, &w, &m)?;
    report::write(out)?;
    if loud {
        if w.p.learned_mut > 0 {
            println!("mutation model trained on {} triples", w.model_total);
        }
        println!("done. {}", out.join("report.html").display());
    }
    Ok(())
}

/// Run a world that never stops: start it, or pick up its snapshot, and keep going.
/// Snapshots every `--snapshot-every` epochs (default 100), with the report refreshed.
fn cmd_serve(args: &[String]) -> Result<(), String> {
    let dir = PathBuf::from(args.first().ok_or("serve DIR [run options]")?);
    let args = &args[1..];
    let every = arg_val(args, "--snapshot-every").and_then(|v| v.parse().ok()).unwrap_or(100);
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let resume = load_snapshot(&dir);
    let cfg = match &resume {
        Some(s) => {
            let mut cfg = Config::from_text(&fs::read_to_string(dir.join("config.txt")).map_err(|e| e.to_string())?)?;
            cfg.ancestor = fs::read_to_string(dir.join("ancestor.tasm")).map_err(|e| e.to_string())?;
            let text = fs::read_to_string(dir.join("config.txt")).map_err(|e| e.to_string())?;
            if ocean_hash(&cfg.ocean) != text.lines().find_map(|l| l.strip_prefix("ocean_hash=")).unwrap_or("") {
                return Err(format!("ocean {} does not match the recorded hash", cfg.ocean));
            }
            // Logs may run past the snapshot if we were killed in between: cut them back.
            for f in ["census.csv", "events.log", "feed.log", "chain.log"] {
                trim_log(&dir.join(f), s.next_epoch, f == "census.csv")?;
            }
            println!("resuming {} at epoch {}", dir.display(), s.next_epoch);
            cfg
        }
        None => {
            let mut cfg = config_from_args(args)?;
            if arg_val(args, "--epochs").is_none() {
                cfg.epochs = u64::MAX / 4;
            }
            fs::write(dir.join("config.txt"), cfg.to_text()).map_err(|e| e.to_string())?;
            fs::write(dir.join("ancestor.tasm"), &cfg.ancestor).map_err(|e| e.to_string())?;
            println!("starting {}", dir.display());
            cfg
        }
    };
    let open = |n: &str| {
        fs::OpenOptions::new().create(true).append(true).open(dir.join(n)).map_err(|e| e.to_string())
    };
    let sinks = Sinks { census: open("census.csv")?, events: open("events.log")?, feed: open("feed.log")?, chain: open("chain.log")?, migrants: None };
    let feed = Feed::parse(&cfg.feed, cfg.seed)?;
    simulate(&cfg, feed, Some(sinks), true, None, Some(Checkpoint { dir: &dir, every, resume }))?;
    Ok(())
}

/// Keep only the lines for epochs before `upto` (and a CSV header).
fn trim_log(path: &Path, upto: u64, header: bool) -> Result<(), String> {
    let Ok(text) = fs::read_to_string(path) else { return Ok(()) };
    let kept: Vec<&str> = text
        .lines()
        .enumerate()
        .filter(|(i, l)| {
            (header && *i == 0) || l.split([',', ' ']).next().and_then(|e| e.parse::<u64>().ok()).is_some_and(|e| e < upto)
        })
        .map(|(_, l)| l)
        .collect();
    fs::write(path, kept.join("\n") + "\n").map_err(|e| e.to_string())
}

/// N worlds on N threads in a ring. Every few epochs each sends a few migrants to the next.
/// Each island logs what it received, so any one of them can be verified alone.
fn cmd_islands(args: &[String]) -> Result<(), String> {
    let n: usize = args.first().and_then(|a| a.parse().ok()).ok_or("islands N [run options]")?;
    let args = &args[1..];
    let mut base = config_from_args(args)?;
    let num = |k: &str, d: u64| arg_val(args, k).map(|v| v.parse::<u64>().unwrap_or(d)).unwrap_or(d);
    base.migrate_every = num("--migrate-every", 10).max(1);
    base.migrate_count = num("--migrate-count", 4);
    let out = PathBuf::from(arg_val(args, "--out").unwrap_or("runs/islands"));
    let (txs, rxs): (Vec<_>, Vec<_>) = (0..n).map(|_| channel::<Vec<Vec<u8>>>()).unzip();
    let mut rxs: Vec<Option<Receiver<_>>> = rxs.into_iter().map(Some).collect();
    println!("abiogenesis: {n} islands, {} epochs, {} migrants every {} epochs", base.epochs, base.migrate_count, base.migrate_every);
    std::thread::scope(|s| {
        let mut handles = Vec::new();
        for i in 0..n {
            let mut cfg = base.clone();
            cfg.seed = base.seed + i as u64;
            let links = Links::Live { tx: txs[(i + 1) % n].clone(), rx: rxs[i].take().unwrap() };
            let dir = out.join(format!("island_{i}"));
            handles.push(s.spawn(move || run_one(&cfg, &dir, i == 0, Some(links))));
        }
        handles.into_iter().map(|h| h.join().unwrap_or(Err("island panicked".into()))).collect::<Result<Vec<_>, _>>()
    })?;
    println!("done. {}", out.display());
    Ok(())
}

fn config_from_args(args: &[String]) -> Result<Config, String> {
    let num = |k: &str, d: u64| arg_val(args, k).map(|v| v.parse::<u64>().map_err(|e| format!("{k}: {e}"))).unwrap_or(Ok(d));
    let mut p = Params::default();
    p.soup_bits = num("--soup-bits", p.soup_bits as u64)? as u32;
    p.sun_bonus = num("--sun-bonus", p.sun_bonus)?;
    p.indel = num("--indel", p.indel)?;
    p.tasks = !args.iter().any(|a| a == "--no-tasks");
    p.ocean_div = num("--ocean-div", p.ocean_div as u64)? as u32;
    p.max_age = num("--max-age", p.max_age)?;
    p.memory = !args.iter().any(|a| a == "--no-memory");
    p.infect = !args.iter().any(|a| a == "--no-infect");
    // The reef lost its 10v10 test (2026-10-16), so it is opt-in now.
    p.reef = args.iter().any(|a| a == "--reef");
    p.max_size = num("--max-size", p.max_size as u64)? as u32;
    p.lean = !args.iter().any(|a| a == "--all-ops");
    p.social = args.iter().any(|a| a == "--social");
    p.genome_ocean = arg_val(args, "--ocean") == Some("genomes");
    p.learned_mut = num("--learned-mut", 0)?.min(100) as u32;
    p.v4 = args.iter().any(|a| a == "--v4");
    p.mem2 = args.iter().any(|a| a == "--mem2");
    p.energy_total = num("--energy-total", p.energy_total)?;
    p.streams = num("--streams", 0)?.min(256) as u32;
    p.energy = arg_val(args, "--economy") == Some("energy");
    p.regime = match arg_val(args, "--regime") {
        Some("feudal") => world::Regime::Feudal,
        Some("capitalist") => world::Regime::Capitalist,
        Some("evolved") => world::Regime::Evolved,
        _ => world::Regime::Commons,
    };
    if p.regime != world::Regime::Commons {
        p.energy = true;
    }
    let seed = num("--seed", 1)?;
    let cfg = Config {
        seed,
        epochs: num("--epochs", 500)?,
        epoch_instr: num("--epoch-instr", 2_000_000)?,
        warmup: num("--warmup", 20)?,
        feed: arg_val(args, "--feed").unwrap_or("chain").to_string(),
        ancestor: match arg_val(args, "--ancestor") {
            Some(path) => fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?,
            None => isa::ANCESTOR.to_string(),
        },
        ocean: arg_val(args, "--ocean").unwrap_or("").to_string(),
        migrate_every: 0,
        migrate_count: 0,
        p,
    };
    Ok(cfg)
}

/// Save the genomes of the most common survivors, readable.
fn write_genebank(out: &Path, w: &World, m: &Meter) -> Result<(), String> {
    let dir = out.join("genebank");
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    for (id, name, pop, act) in m.top(12) {
        if let Some(c) = w.cs.iter().find(|c| c.alive && c.genotype == id) {
            let body = format!("; {name}  population {pop}  activity {act}\n{}", isa::disassemble(w.genome(c)));
            fs::write(dir.join(format!("{name}.tasm")), body).map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn cmd_verify(dir: &Path) -> Result<(), String> {
    let mut cfg = Config::from_text(&fs::read_to_string(dir.join("config.txt")).map_err(|e| e.to_string())?)?;
    cfg.ancestor = fs::read_to_string(dir.join("ancestor.tasm")).unwrap_or_else(|_| isa::ANCESTOR.to_string());
    let text = fs::read_to_string(dir.join("config.txt")).map_err(|e| e.to_string())?;
    let recorded = text.lines().find_map(|l| l.strip_prefix("ocean_hash=")).unwrap_or("");
    if ocean_hash(&cfg.ocean) != recorded {
        return Err(format!("ocean {} does not match the recorded hash", cfg.ocean));
    }
    let recorded: Vec<(u64, [u8; 32])> = fs::read_to_string(dir.join("chain.log"))
        .map_err(|e| e.to_string())?
        .lines()
        .filter_map(|l| {
            let (e, h) = l.split_once(' ')?;
            Some((e.parse().ok()?, feed::unhex(h)?))
        })
        .collect();
    cfg.epochs = recorded.len() as u64;
    let feed = Feed::parse(&format!("replay:{}", dir.join("feed.log").display()), cfg.seed)?;
    let t0 = Instant::now();
    let links = if cfg.migrate_every > 0 {
        let mut log: BTreeMap<u64, Vec<Vec<u8>>> = BTreeMap::new();
        for l in fs::read_to_string(dir.join("migrants.log")).unwrap_or_default().lines() {
            if let Some((e, g)) = l.split_once(' ') {
                let bytes: Vec<u8> = (0..g.len() / 2).filter_map(|i| u8::from_str_radix(&g[i * 2..i * 2 + 2], 16).ok()).collect();
                log.entry(e.parse().map_err(|_| "bad migrants.log")?).or_default().push(bytes);
            }
        }
        Some(Links::Replay(log))
    } else {
        None
    };
    let (chain, _, _) = simulate(&cfg, feed, None, false, links, None)?;
    for (i, (e, h)) in recorded.iter().enumerate() {
        match chain.get(i) {
            Some(mine) if mine == h => {}
            Some(mine) => return Err(format!("epoch {e}: recorded {} but replay gives {}", hex(h), hex(mine))),
            None => return Err(format!("replay stopped before epoch {e}")),
        }
    }
    println!(
        "verified: {} epochs replayed in {:.1}s, final hash {}",
        recorded.len(),
        t0.elapsed().as_secs_f64(),
        recorded.last().map(|r| hex(&r.1)).unwrap_or_default()
    );
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let res = match args.first().map(|s| s.as_str()) {
        Some("run") => cmd_run(&args[1..]),
        Some("islands") => cmd_islands(&args[1..]),
        Some("serve") if args.len() > 1 => cmd_serve(&args[1..]),
        Some("verify") if args.len() > 1 => cmd_verify(Path::new(&args[1])),
        Some("report") if args.len() > 1 => report::write(Path::new(&args[1])),
        Some("compare") if args.len() > 2 => analyze::compare(&args[1], &args[2]),
        Some("ancestor") => {
            let a = isa::assemble(isa::ANCESTOR).unwrap();
            print!("; ancestor, {} cells, genotype {}\n{}", a.len(), world::genotype_of(&a), isa::disassemble(&a));
            Ok(())
        }
        _ => {
            println!("{USAGE}");
            Ok(())
        }
    };
    if let Err(e) = res {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
