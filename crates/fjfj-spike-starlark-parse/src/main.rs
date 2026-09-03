//! Spike for buildfiji-mum.1: how fast is the `starlark` crate's front end on
//! real BUILD trees, how much memory does the AST cost, and what share of a
//! load is parsing?
//!
//! For each corpus directory it reports, per phase and in MB/s of Starlark
//! source: lex only, parse (lex + parse + AST build), parse held in memory
//! (RSS delta), stub evaluation (see `stubs`), and parallel parse scaling.
//!
//! Usage:
//!   fixtures/fetch.sh <corpus-dir>
//!   fjfj-spike-starlark-parse <corpus-dir>/envoy <corpus-dir>/tensorflow ...

mod stubs;

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;
use std::time::Duration;
use std::time::Instant;

use clap::Parser;
use starlark::environment::Module;
use starlark::eval::Evaluator;
use starlark::syntax::AstModule;
use starlark::syntax::Dialect;
use starlark_syntax::codemap::CodeMap;
use starlark_syntax::lexer::Lexer;
use starlark_syntax::lexer::Token;

use crate::stubs::Names;
use crate::stubs::StubEnv;

#[derive(Parser)]
#[command(about = "Starlark front-end throughput and memory spike (buildfiji-mum.1)")]
struct Args {
    /// Corpus directories, measured one at a time.
    #[arg(required = true)]
    corpora: Vec<PathBuf>,
    /// Timed passes per phase; the fastest is reported.
    #[arg(long, default_value_t = 5)]
    repeat: usize,
    /// Replicate the corpus N times to reach a larger tree (chromium scale).
    #[arg(long, default_value_t = 1)]
    replicate: usize,
    /// Threads for the parallel parse phase.
    #[arg(long)]
    threads: Option<usize>,
    /// Skip the stub evaluation phase.
    #[arg(long)]
    skip_eval: bool,
    /// Print this many parse and evaluate failures per corpus.
    #[arg(long, default_value_t = 5)]
    show_errors: usize,
}

/// One corpus file, already read and classified by dialect.
struct Source {
    name: String,
    text: String,
    is_build: bool,
}

impl Source {
    fn dialect(&self) -> Dialect {
        if self.is_build {
            fjfj_starlark::build_dialect()
        } else {
            fjfj_starlark::bzl_dialect()
        }
    }
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let threads = match args.threads {
        Some(n) => n,
        None => std::thread::available_parallelism()?.get(),
    };
    println!(
        "starlark {} · repeat {} · replicate {} · threads {}",
        starlark_version(),
        args.repeat,
        args.replicate,
        threads
    );

    let mut summary = Vec::new();
    for (index, dir) in args.corpora.iter().enumerate() {
        // RSS only tells the truth about the first corpus in a process: the
        // allocator keeps the pages freed by earlier corpora, so a later hold
        // phase measures reuse, not cost. Memory numbers come from one-corpus
        // runs.
        let row = measure(dir, &args, threads, index == 0)?;
        summary.push(row);
    }

    println!("\n== summary ==");
    println!(
        "{:<14} {:>7} {:>10} {:>9} {:>9} {:>10} {:>9} {:>9}",
        "corpus", "files", "MB", "lex MB/s", "parse", "par parse", "AST/src", "parse %"
    );
    for row in &summary {
        println!(
            "{:<14} {:>7} {:>10.1} {:>9.1} {:>9.1} {:>10.1} {:>9.2} {:>8.0}%",
            row.name,
            row.files,
            row.bytes as f64 / 1e6,
            row.lex_mbs,
            row.parse_mbs,
            row.par_parse_mbs,
            row.ast_ratio,
            row.parse_share * 100.0
        );
    }
    Ok(())
}

struct SummaryRow {
    name: String,
    files: usize,
    bytes: usize,
    lex_mbs: f64,
    parse_mbs: f64,
    par_parse_mbs: f64,
    ast_ratio: f64,
    parse_share: f64,
}

fn measure(
    dir: &Path,
    args: &Args,
    threads: usize,
    memory_is_reliable: bool,
) -> anyhow::Result<SummaryRow> {
    let name = dir
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| dir.display().to_string());
    println!("\n== {} ==", dir.display());

    let read_start = Instant::now();
    let mut sources = read_corpus(dir)?;
    let read = read_start.elapsed();
    // Read throughput is over the tree as it is on disk, before replication.
    let read_mb = sources.iter().map(|s| s.text.len()).sum::<usize>() as f64 / 1e6;
    if args.replicate > 1 {
        sources = replicate(sources, args.replicate);
    }
    anyhow::ensure!(
        !sources.is_empty(),
        "no Starlark files under {}",
        dir.display()
    );

    let bytes: usize = sources.iter().map(|s| s.text.len()).sum();
    let lines: usize = sources.iter().map(|s| s.text.lines().count()).sum();
    let build_files = sources.iter().filter(|s| s.is_build).count();
    let mb = bytes as f64 / 1e6;
    println!(
        "{} files ({} BUILD, {} bzl), {:.2} MB, {} lines, mean {:.0} B/file",
        sources.len(),
        build_files,
        sources.len() - build_files,
        mb,
        lines,
        bytes as f64 / sources.len() as f64,
    );
    println!(
        "  read       {:>8.1} MB/s  {:.3} s to read the tree (warm page cache)",
        read_mb / read.as_secs_f64().max(f64::MIN_POSITIVE),
        read.as_secs_f64(),
    );
    let rss_sources = rss_bytes();

    // Baseline: `AstModule::parse` takes the source by value, so every timed
    // parse pass includes one copy of the corpus. Measured here so it can be
    // reported (and shown to be noise) rather than silently folded into parse.
    let copy = best_of(args.repeat, || {
        let mut acc = 0usize;
        for s in &sources {
            acc += std::hint::black_box(s.text.clone()).len();
        }
        std::hint::black_box(acc);
    });

    // Phase 1: lex only.
    let mut tokens = 0usize;
    let lex = best_of(args.repeat, || {
        tokens = 0;
        for s in &sources {
            let dialect = s.dialect();
            let codemap = CodeMap::new(s.name.clone(), s.text.clone());
            let mut lexer = Lexer::new(&s.text, &dialect, codemap);
            while let Some(token) = lexer.next() {
                match token {
                    Ok(_) => tokens += 1,
                    Err(_) => break,
                }
            }
        }
    });

    // Phase 2: parse, dropping each AST immediately.
    let mut parse_ok = 0usize;
    let mut errors: Vec<String> = Vec::new();
    let parse = best_of(args.repeat, || {
        parse_ok = 0;
        errors.clear();
        for s in &sources {
            match AstModule::parse(&s.name, s.text.clone(), &s.dialect()) {
                Ok(ast) => {
                    parse_ok += 1;
                    drop(std::hint::black_box(ast));
                }
                Err(e) => {
                    if errors.len() < args.show_errors {
                        errors.push(format!("{}: {}", s.name, one_line(&e.to_string())));
                    }
                }
            }
        }
    });

    // Per-file distribution, measured in one untimed extra pass.
    let mut per_file: Vec<(f64, usize)> = Vec::with_capacity(sources.len());
    for s in &sources {
        let start = Instant::now();
        let ast = AstModule::parse(&s.name, s.text.clone(), &s.dialect());
        let elapsed = start.elapsed().as_secs_f64();
        if ast.is_ok() {
            per_file.push((elapsed, s.text.len()));
        }
        drop(ast);
    }

    println!(
        "  lex        {:>8.1} MB/s  {:>10} tokens  {:.2} B/token",
        mb / lex.as_secs_f64(),
        tokens,
        bytes as f64 / tokens as f64,
    );
    println!(
        "  parse      {:>8.1} MB/s  {:>10} ok / {} files  ({} failed)",
        mb / parse.as_secs_f64(),
        parse_ok,
        sources.len(),
        sources.len() - parse_ok,
    );
    println!(
        "  parse wall {:>8.3} s    copy baseline {:.3} s ({:.1}% of parse)  lex share {:.0}%",
        parse.as_secs_f64(),
        copy.as_secs_f64(),
        100.0 * copy.as_secs_f64() / parse.as_secs_f64(),
        100.0 * lex.as_secs_f64() / parse.as_secs_f64(),
    );
    report_per_file(&per_file);
    for e in &errors {
        println!("    parse error: {e}");
    }

    // Phase 3: parse and hold every AST, to price the retained syntax tree.
    let before = rss_bytes();
    let start = Instant::now();
    let asts: Vec<AstModule> = sources
        .iter()
        .filter_map(|s| AstModule::parse(&s.name, s.text.clone(), &s.dialect()).ok())
        .collect();
    let hold = start.elapsed();
    let after = rss_bytes();
    let ast_bytes = after.saturating_sub(before);
    let ast_ratio = ast_bytes as f64 / bytes as f64;
    println!(
        "  hold       {:>8.1} MB/s  RSS +{:.1} MB for {} ASTs ({:.2}x source, {:.0} B/file){}",
        mb / hold.as_secs_f64(),
        ast_bytes as f64 / 1e6,
        asts.len(),
        ast_ratio,
        ast_bytes as f64 / asts.len() as f64,
        if memory_is_reliable {
            ""
        } else {
            "  [unreliable: not the first corpus in this process]"
        },
    );
    println!(
        "  rss        sources {:.1} MB, with ASTs {:.1} MB",
        rss_sources as f64 / 1e6,
        after as f64 / 1e6,
    );

    // Names for the stub environment come from the held ASTs plus a token scan.
    let mut names = Names::default();
    for ast in &asts {
        for load in ast.loads() {
            for (_local, source) in load.symbols.iter() {
                names.loaded.insert((*source).to_owned());
            }
        }
    }
    for s in &sources {
        scan_names(s, &mut names);
    }
    drop(asts);

    // Phase 4: stub evaluation, for the parse share of a load.
    let mut parse_share = 0.0;
    if !args.skip_eval {
        let env = StubEnv::new(&names)?;
        let mut eval_ok = 0usize;
        let mut parse_secs = 0.0;
        let mut eval_secs = 0.0;
        let mut eval_errors: Vec<String> = Vec::new();
        for s in &sources {
            let start = Instant::now();
            let ast = match AstModule::parse(&s.name, s.text.clone(), &s.dialect()) {
                Ok(ast) => ast,
                Err(_) => continue,
            };
            let parsed = start.elapsed();
            let start = Instant::now();
            let result = Module::with_temp_heap(|module| {
                let mut evaluator = Evaluator::new(&module);
                evaluator.set_loader(&env);
                evaluator.eval_module(ast, &env.globals).map(|_| ())
            });
            let evaluated = start.elapsed();
            match result {
                Ok(_) => {
                    eval_ok += 1;
                    parse_secs += parsed.as_secs_f64();
                    eval_secs += evaluated.as_secs_f64();
                }
                Err(e) => {
                    if eval_errors.len() < args.show_errors {
                        eval_errors.push(format!("{}: {}", s.name, one_line(&e.to_string())));
                    }
                }
            }
        }
        parse_share = parse_secs / (parse_secs + eval_secs);
        println!(
            "  eval       {:>8} of {} files evaluate under stubs ({:.0}%)",
            eval_ok,
            parse_ok,
            100.0 * eval_ok as f64 / parse_ok as f64,
        );
        println!(
            "  load split parse {:.3} s + stub eval {:.3} s -> parse is {:.0}% of load (upper bound)",
            parse_secs,
            eval_secs,
            100.0 * parse_share,
        );
        for e in &eval_errors {
            println!("    eval error: {e}");
        }
    }

    // Phase 5: parallel parse.
    let par = best_of(args.repeat.min(3), || {
        let cursor = AtomicUsize::new(0);
        std::thread::scope(|scope| {
            for _ in 0..threads {
                scope.spawn(|| {
                    loop {
                        let i = cursor.fetch_add(1, Ordering::Relaxed);
                        let Some(s) = sources.get(i) else { break };
                        let ast = AstModule::parse(&s.name, s.text.clone(), &s.dialect());
                        drop(std::hint::black_box(ast));
                    }
                });
            }
        });
    });
    let par_mbs = mb / par.as_secs_f64();
    println!(
        "  par parse  {:>8.1} MB/s  {} threads, {:.2} s, {:.1}x speedup",
        par_mbs,
        threads,
        par.as_secs_f64(),
        parse.as_secs_f64() / par.as_secs_f64(),
    );

    Ok(SummaryRow {
        name,
        files: sources.len(),
        bytes,
        lex_mbs: mb / lex.as_secs_f64(),
        parse_mbs: mb / parse.as_secs_f64(),
        par_parse_mbs: par_mbs,
        ast_ratio,
        parse_share,
    })
}

/// Collect callable and `root.member` names with a token scan; the evaluator
/// needs a binding for every one of them.
fn scan_names(source: &Source, names: &mut Names) {
    let dialect = source.dialect();
    let codemap = CodeMap::new(source.name.clone(), source.text.clone());
    let mut lexer = Lexer::new(&source.text, &dialect, codemap);
    let mut prev: Option<String> = None;
    let mut dotted: Option<String> = None;
    while let Some(token) = lexer.next() {
        let Ok((_, token, _)) = token else { break };
        match token {
            Token::Identifier(id) => {
                if let Some(root) = dotted.take() {
                    names.namespaces.entry(root).or_default().insert(id.clone());
                    prev = None;
                } else {
                    prev = Some(id);
                }
            }
            Token::Dot => {
                dotted = prev.take();
            }
            Token::OpeningRound => {
                if let Some(id) = prev.take() {
                    names.callables.insert(id);
                }
                dotted = None;
            }
            _ => {
                prev = None;
                dotted = None;
            }
        }
    }
}

fn report_per_file(per_file: &[(f64, usize)]) {
    if per_file.is_empty() {
        return;
    }
    let mut times: Vec<f64> = per_file.iter().map(|(t, _)| *t).collect();
    times.sort_by(f64::total_cmp);
    let pct = |p: f64| times[((times.len() - 1) as f64 * p) as usize] * 1e6;
    let slowest = per_file
        .iter()
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .expect("non-empty");
    println!(
        "  per file   p50 {:.0} us, p95 {:.0} us, p99 {:.0} us, max {:.1} ms ({:.0} KB file)",
        pct(0.5),
        pct(0.95),
        pct(0.99),
        slowest.0 * 1e3,
        slowest.1 as f64 / 1e3,
    );
}

fn best_of(repeat: usize, mut f: impl FnMut()) -> Duration {
    let mut best = Duration::MAX;
    for _ in 0..repeat.max(1) {
        let start = Instant::now();
        f();
        best = best.min(start.elapsed());
    }
    best
}

fn read_corpus(dir: &Path) -> anyhow::Result<Vec<Source>> {
    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    out.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(out)
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<Source>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();
        if entry.file_type()?.is_dir() {
            if file_name != ".git" {
                walk(root, &path, out)?;
            }
            continue;
        }
        let is_build = file_name == "BUILD" || file_name == "BUILD.bazel";
        let is_bzl = file_name.ends_with(".bzl")
            || file_name.ends_with(".star")
            || file_name == "MODULE.bazel";
        if !is_build && !is_bzl {
            continue;
        }
        // Skip the handful of files that are not UTF-8 or are templates with
        // placeholder syntax; they are not Starlark the parser could accept.
        let Ok(text) = std::fs::read_to_string(&path) else {
            continue;
        };
        out.push(Source {
            name: path
                .strip_prefix(root)
                .unwrap_or(&path)
                .display()
                .to_string(),
            text,
            is_build,
        });
    }
    Ok(())
}

/// Grow the corpus by replicating it, to reach a tree size no single open
/// source Bazel repo has (the chromium-scale point in buildfiji-mum.1).
fn replicate(sources: Vec<Source>, times: usize) -> Vec<Source> {
    let mut out = Vec::with_capacity(sources.len() * times);
    for copy in 0..times {
        for s in &sources {
            out.push(Source {
                name: format!("copy{copy}/{}", s.name),
                text: s.text.clone(),
                is_build: s.is_build,
            });
        }
    }
    out
}

/// Resident set size of this process, via `ps` (no allocator instrumentation,
/// so the timed phases run on an untouched allocator).
fn rss_bytes() -> usize {
    let pid = std::process::id().to_string();
    let Ok(out) = Command::new("ps").args(["-o", "rss=", "-p", &pid]).output() else {
        return 0;
    };
    String::from_utf8_lossy(&out.stdout)
        .trim()
        .parse::<usize>()
        .unwrap_or(0)
        * 1024
}

fn one_line(s: &str) -> String {
    s.lines().next().unwrap_or("").chars().take(120).collect()
}

fn starlark_version() -> &'static str {
    "0.14"
}
