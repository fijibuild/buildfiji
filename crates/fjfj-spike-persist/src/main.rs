//! Persistence spike (bead buildfiji-23d.9): encode a real Bazel graph in
//! each candidate format and report size and cold-load time.

mod columnar;
mod graph;
mod kv;
mod load;
mod nodebytes;
mod rkyv_enc;
mod varint;

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use anyhow::{Result, ensure};
use clap::Parser;

#[derive(Parser)]
#[command(about = "fjfj persistence format spike")]
struct Args {
    /// `bazel aquery 'deps(//...)' --output=jsonproto --include_artifacts`
    #[arg(long)]
    aquery: PathBuf,
    /// `bazel cquery 'deps(//...)' --output=jsonproto`
    #[arg(long)]
    cquery: PathBuf,
    /// Scratch directory for store files.
    #[arg(long)]
    out: PathBuf,
    /// zstd level for the compressed candidates.
    #[arg(long, default_value_t = 3)]
    zstd_level: i32,
    /// Timing repetitions (best of N reported).
    #[arg(long, default_value_t = 5)]
    reps: u32,
}

struct Row {
    name: &'static str,
    bytes: u64,
    write: Duration,
    load: Duration,
}

fn best<T>(reps: u32, mut f: impl FnMut() -> Result<T>) -> Result<(T, Duration)> {
    let mut best = Duration::MAX;
    let mut out = None;
    for _ in 0..reps.max(1) {
        let t = Instant::now();
        let v = f()?;
        best = best.min(t.elapsed());
        out = Some(v);
    }
    Ok((out.expect("reps >= 1"), best))
}

fn dir_size(p: &Path) -> Result<u64> {
    let md = std::fs::metadata(p)?;
    if md.is_file() {
        return Ok(md.len());
    }
    let mut total = 0;
    for e in std::fs::read_dir(p)? {
        total += dir_size(&e?.path())?;
    }
    Ok(total)
}

fn reset(p: &Path) -> Result<()> {
    if p.exists() {
        if p.is_dir() {
            std::fs::remove_dir_all(p)?;
        } else {
            std::fs::remove_file(p)?;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    std::fs::create_dir_all(&args.out)?;
    let (g, stats) = load::load(&args.aquery, &args.cquery)?;
    let nodes = g.nodes.len();
    println!(
        "graph: {} nodes ({} configured targets, {} actions, {} depsets, {} files), {} edges, {} strings / {} string bytes",
        nodes,
        stats.configured_targets,
        stats.actions,
        stats.depsets,
        stats.files,
        g.edge_count(),
        g.strings.table.len(),
        g.string_bytes()
    );
    let checksum = g.scan_checksum();
    let mut rows = Vec::new();

    // A: columnar + zstd.
    let (enc, write) = best(args.reps, || columnar::encode(&g, args.zstd_level))?;
    let snap = args.out.join("snapshot.fjfj");
    std::fs::write(&snap, &enc.bytes)?;
    let (decoded, load) = best(args.reps, || columnar::decode(&std::fs::read(&snap)?))?;
    ensure!(decoded.same_as(&g), "columnar roundtrip mismatch");
    println!(
        "columnar sections raw/zstd: strings {}/{}, edges {}/{} ({} unique lists), nodes {}/{}",
        enc.section_raw[0],
        enc.section_zstd[0],
        enc.section_raw[1],
        enc.section_zstd[1],
        enc.unique_edge_lists,
        enc.section_raw[2],
        enc.section_zstd[2]
    );
    rows.push(Row {
        name: "columnar raw (uncompressed)",
        bytes: enc.raw_len as u64,
        write,
        load,
    });
    rows.push(Row {
        name: "columnar + zstd, eager decode",
        bytes: enc.bytes.len() as u64,
        write,
        load,
    });

    // B: rkyv.
    let (rk, write) = best(args.reps, || rkyv_enc::encode(&g))?;
    let rk_path = args.out.join("snapshot.rkyv");
    std::fs::write(&rk_path, &rk)?;
    let (_, load) = best(args.reps, || {
        let bytes = rkyv_enc::zstd_unwrap(&rkyv_enc::zstd_wrap(&std::fs::read(&rk_path)?, 1)?)?;
        let a = rkyv_enc::access_checked(&bytes)?;
        ensure!(rkyv_enc::scan(a) == checksum, "rkyv scan mismatch");
        Ok(())
    })?;
    let _ = load; // roundtrip through zstd only to force an aligned copy; timed separately below
    let (_, load_checked) = best(args.reps, || {
        let mut v = rkyv::util::AlignedVec::<16>::new();
        v.extend_from_slice(&std::fs::read(&rk_path)?);
        let a = rkyv_enc::access_checked(&v)?;
        ensure!(rkyv_enc::scan(a) == checksum, "rkyv scan mismatch");
        Ok(())
    })?;
    rows.push(Row {
        name: "rkyv raw, validated access + scan",
        bytes: rk.len() as u64,
        write,
        load: load_checked,
    });
    let (_, load_mat) = best(args.reps, || {
        let mut v = rkyv::util::AlignedVec::<16>::new();
        v.extend_from_slice(&std::fs::read(&rk_path)?);
        let a = rkyv_enc::access_checked(&v)?;
        let m = rkyv_enc::materialise(a)?;
        ensure!(m.same_as(&g), "rkyv materialise mismatch");
        Ok(())
    })?;
    rows.push(Row {
        name: "rkyv raw, materialised",
        bytes: rk.len() as u64,
        write,
        load: load_mat,
    });
    let (rkz, write_z) = best(args.reps, || rkyv_enc::zstd_wrap(&rk, args.zstd_level))?;
    let rkz_path = args.out.join("snapshot.rkyv.zst");
    std::fs::write(&rkz_path, &rkz)?;
    let (_, load_z) = best(args.reps, || {
        let v = rkyv_enc::zstd_unwrap(&std::fs::read(&rkz_path)?)?;
        let a = rkyv_enc::access_checked(&v)?;
        ensure!(rkyv_enc::scan(a) == checksum, "rkyv scan mismatch");
        Ok(())
    })?;
    rows.push(Row {
        name: "rkyv + zstd, validated access + scan",
        bytes: rkz.len() as u64,
        write: write + write_z,
        load: load_z,
    });

    // C: redb.
    let redb_path = args.out.join("graph.redb");
    let (_, write) = best(args.reps, || {
        reset(&redb_path)?;
        kv::redb_write(&g, &redb_path)
    })?;
    let (decoded, load) = best(args.reps, || kv::redb_read(&redb_path))?;
    ensure!(decoded.same_as(&g), "redb roundtrip mismatch");
    rows.push(Row {
        name: "redb (B-tree, no compression)",
        bytes: dir_size(&redb_path)?,
        write,
        load,
    });

    // D: fjall.
    let fjall_path = args.out.join("graph.fjall");
    let (_, write) = best(args.reps, || {
        reset(&fjall_path)?;
        kv::fjall_write(&g, &fjall_path)
    })?;
    let (decoded, load) = best(args.reps, || kv::fjall_read(&fjall_path))?;
    ensure!(decoded.same_as(&g), "fjall roundtrip mismatch");
    rows.push(Row {
        name: "fjall (LSM, lz4)",
        bytes: dir_size(&fjall_path)?,
        write,
        load,
    });

    println!();
    println!("| candidate | bytes | bytes/node | write | cold load |");
    println!("|---|---:|---:|---:|---:|");
    for r in &rows {
        println!(
            "| {} | {} | {:.1} | {:.1?} | {:.1?} |",
            r.name,
            r.bytes,
            r.bytes as f64 / nodes as f64,
            r.write,
            r.load
        );
    }
    Ok(())
}
