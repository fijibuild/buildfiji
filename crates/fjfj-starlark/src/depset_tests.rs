//! `depset` against Bazel 9.2.0: the tables at the bottom are what it printed
//! and reported for the same BUILD files (buildfiji-mum.14), generated from
//! probe runs, so a disagreement is fjfj's to explain.

use crate::test_support::{Capture, run};
use crate::{FileKind, bzl_globals, parse};
use starlark::environment::Module;
use starlark::eval::Evaluator;
use std::cell::RefCell;

/// Probes that are known differences: `depset([len])` (a builtin function is
/// "mutable" to Bazel) belongs to buildfiji-ahp.
const RUNTIME_WORDING: &[&str] = &["depset([len"];

fn known_difference(src: &str) -> bool {
    RUNTIME_WORDING.iter().any(|marker| src.contains(marker))
}

/// Bazel's `str(list)` and ours agree, so compare printed text directly.
#[test]
fn what_bazel_prints() {
    let mut wrong = Vec::new();
    for (src, want) in PRINTS.iter().filter(|(src, _)| !known_difference(src)) {
        match run(src) {
            Ok(got) if got.join("\n") == *want => {}
            other => wrong.push(format!("{src}\n  want: {want}\n  got:  {other:?}")),
        }
    }
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn what_bazel_reports() {
    let mut wrong = Vec::new();
    for (src, want) in ERRORS.iter().filter(|(src, _)| !known_difference(src)) {
        match run(src) {
            Err(got) if got.contains(want) => {}
            other => wrong.push(format!("{src}\n  want: {want}\n  got:  {other:?}")),
        }
    }
    assert!(
        wrong.is_empty(),
        "{} differ:\n{}",
        wrong.len(),
        wrong.join("\n")
    );
}

#[test]
fn a_very_deep_chain_flattens_without_recursion() {
    let got = run("def chain(n):\n    d = depset([\"leaf\"])\n    for i in range(n):\n        d = depset([str(i)], transitive = [d])\n    return d\nl = chain(100000).to_list()\nprint(len(l), l[0], l[1], l[-1])")
    .unwrap();
    assert_eq!(got, ["100001 leaf 0 99999"]);
}

#[test]
fn a_dag_with_exponentially_many_paths_is_walked_once() {
    // Each level names the level below twice, so there are 2^60 paths.
    let got = run("def dag(n):\n    d = depset([\"leaf\"])\n    for i in range(n):\n        d = depset([str(i)], transitive = [d, d])\n    return d\nprint(len(dag(60).to_list()))")
    .unwrap();
    assert_eq!(got, ["61"]);
}

#[test]
fn building_shares_rather_than_copies() {
    // A one-set depset of the same order is that set, so wrapping is free.
    let got = run("a = depset([\"p\", \"q\"], order = \"preorder\")\nb = depset(transitive = [a], order = \"preorder\")\nprint(a == b)\nc = depset(transitive = [b, depset()], order = \"preorder\")\nprint(a == c)")
    .unwrap();
    assert_eq!(got, ["True", "True"]);
}

/// Freezing keeps the DAG, its orders and its identity.
#[test]
fn depsets_survive_freezing_and_combine_with_live_ones() {
    use starlark::environment::{FrozenModule, Globals};
    use starlark::eval::FileLoader;

    struct Loader;
    impl FileLoader for Loader {
        fn load(&self, path: &str) -> starlark::Result<FrozenModule> {
            assert_eq!(path, ":lib.bzl");
            let src = "shared = depset([\"s1\", \"s2\"], order = \"topological\")\ntop = depset([\"t\"], transitive = [shared], order = \"topological\")\nempty = depset()\n";
            let ast = parse(path, src, FileKind::Bzl).map_err(starlark::Error::new_other)?;
            Module::with_temp_heap(|module| {
                {
                    let mut eval = Evaluator::new(&module);
                    eval.eval_module(ast, &bzl_globals())?;
                }
                Ok(module.freeze()?)
            })
        }
    }
    let _: fn() -> Globals = bzl_globals;
    let src = "load(\":lib.bzl\", \"shared\", \"top\", \"empty\")\nprint(top.to_list(), top)\nlive = depset([\"l\"], transitive = [top], order = \"topological\")\nprint(live.to_list())\nprint(depset(transitive = [top], order = \"topological\") == top)\nprint(empty == depset(), empty)\nprint(depset([\"x\"], transitive = [shared]).to_list())\n";
    let ast = parse("t.bzl", src, FileKind::Bzl).unwrap();
    let capture = Capture(RefCell::new(Vec::new()));
    Module::with_temp_heap(|module| {
        let mut eval = Evaluator::new(&module);
        eval.set_loader(&Loader);
        eval.set_print_handler(&capture);
        eval.eval_module(ast, &bzl_globals()).unwrap();
    });
    assert_eq!(
        capture.0.into_inner(),
        [
            "[\"t\", \"s1\", \"s2\"] depset([\"t\", \"s1\", \"s2\"], order = \"topological\")",
            "[\"l\", \"t\", \"s1\", \"s2\"]",
            "True",
            "True depset([])",
            // A default-order set walks a topological child by its own layout.
            "[\"s2\", \"s1\", \"x\"]",
        ]
    );
}

/// Freezing recurses through nested sets inside the `starlark` crate, about
/// 3 KiB of stack a level: this test thread's 2 MiB gave out between 500 and
/// 1,000 levels before the freeze grew its own stack.
#[test]
fn a_deep_chain_freezes() {
    let ast = parse(
        "t.bzl",
        "def chain(n):\n    d = depset([\"leaf\"])\n    for i in range(n):\n        d = depset([str(i)], transitive = [d])\n    return d\ntop = chain(50000)\n",
        FileKind::Bzl,
    )
    .unwrap();
    Module::with_temp_heap(|module| {
        {
            let mut eval = Evaluator::new(&module);
            eval.eval_module(ast, &bzl_globals()).unwrap();
        }
        let frozen = module.freeze().unwrap();
        let top = frozen.get("top").unwrap();
        assert_eq!(
            crate::depset_to_list(top.value()).unwrap().unwrap().len(),
            50001
        );
    });
}

/// `(source, what Bazel 9.2.0 printed)`: every line `print` gave.
const PRINTS: &[(&str, &str)] = &[
    (
        r#"a=depset([1,2])
print(a, repr(a), str(a), type(a), a.to_list())"#,
        r#"depset([1, 2]) depset([1, 2]) depset([1, 2]) depset [1, 2]"#,
    ),
    (
        r#"print(depset(), depset([]), depset(None), depset(direct=[1]), depset([1], "postorder"), depset([1], order="topological"), depset([1], order="preorder"), depset([1], order="default"))"#,
        r#"depset([]) depset([]) depset([]) depset([1]) depset([1], order = "postorder") depset([1], order = "topological") depset([1], order = "preorder") depset([1])"#,
    ),
    (r#"print(depset([1,1,2,1,3,2]).to_list())"#, r#"[1, 2, 3]"#),
    (r#"print(depset((1,2)).to_list())"#, r#"[1, 2]"#),
    (
        r#"print(depset(["b","a","c"]).to_list())"#,
        r#"["b", "a", "c"]"#,
    ),
    (
        r#"a=depset(["a1","a2"])
b=depset(["b1","b2"])
c=depset(["c1"],transitive=[a,b])
print(c.to_list())"#,
        r#"["a1", "a2", "b1", "b2", "c1"]"#,
    ),
    (
        r#"a=depset(["a1","a2"])
b=depset(["b1","b2"])
c=depset(["c1"],transitive=[a,b],order="postorder")
print(c.to_list())"#,
        r#"["a1", "a2", "b1", "b2", "c1"]"#,
    ),
    (
        r#"a=depset(["a1","a2"])
b=depset(["b1","b2"])
c=depset(["c1"],transitive=[a,b],order="preorder")
print(c.to_list())"#,
        r#"["c1", "a1", "a2", "b1", "b2"]"#,
    ),
    (
        r#"a=depset(["a1","a2"])
b=depset(["b1","b2"])
c=depset(["c1"],transitive=[a,b],order="topological")
print(c.to_list())"#,
        r#"["c1", "a2", "a1", "b2", "b1"]"#,
    ),
    (
        r#"a=depset(["a1","a2"],order="postorder")
b=depset(["b1","b2"],order="postorder")
c=depset(["c1"],transitive=[a,b],order="postorder")
print(c.to_list())"#,
        r#"["a1", "a2", "b1", "b2", "c1"]"#,
    ),
    (
        r#"a=depset(["x","a"],order="preorder")
b=depset(["b","x"],order="preorder")
c=depset(["c","a"],transitive=[a,b],order="preorder")
print(c.to_list())"#,
        r#"["c", "a", "x", "b"]"#,
    ),
    (
        r#"a=depset(["x","a"],order="postorder")
b=depset(["b","x"],order="postorder")
c=depset(["c","a"],transitive=[a,b],order="postorder")
print(c.to_list())"#,
        r#"["x", "a", "b", "c"]"#,
    ),
    (
        r#"a=depset(["x","a"],order="topological")
b=depset(["b","x"],order="topological")
c=depset(["c","a"],transitive=[a,b],order="topological")
print(c.to_list())"#,
        r#"["c", "a", "b", "x"]"#,
    ),
    (
        r#"a=depset(["x","a"])
b=depset(["b","x"])
c=depset(["c","a"],transitive=[a,b])
print(c.to_list())"#,
        r#"["x", "a", "b", "c"]"#,
    ),
    (
        r#"d=depset(["d"])
a=depset(["a"],transitive=[d])
b=depset(["b"],transitive=[d])
c=depset(["c"],transitive=[a,b])
print(c.to_list())"#,
        r#"["d", "a", "b", "c"]"#,
    ),
    (
        r#"d=depset(["d"],order="postorder")
a=depset(["a"],transitive=[d],order="postorder")
b=depset(["b"],transitive=[d],order="postorder")
c=depset(["c"],transitive=[a,b],order="postorder")
print(c.to_list())"#,
        r#"["d", "a", "b", "c"]"#,
    ),
    (
        r#"d=depset(["d"],order="preorder")
a=depset(["a"],transitive=[d],order="preorder")
b=depset(["b"],transitive=[d],order="preorder")
c=depset(["c"],transitive=[a,b],order="preorder")
print(c.to_list())"#,
        r#"["c", "a", "d", "b"]"#,
    ),
    (
        r#"d=depset(["d"],order="topological")
a=depset(["a"],transitive=[d],order="topological")
b=depset(["b"],transitive=[d],order="topological")
c=depset(["c"],transitive=[a,b],order="topological")
print(c.to_list())"#,
        r#"["c", "a", "b", "d"]"#,
    ),
    (
        r#"d=depset(["d"],order="topological")
a=depset(["a"],transitive=[d],order="topological")
b=depset(["b"],transitive=[d],order="topological")
c=depset(["c"],transitive=[b,a],order="topological")
print(c.to_list())"#,
        r#"["c", "b", "a", "d"]"#,
    ),
    (
        r#"a=depset([1,2])
print(bool(a), bool(depset()))"#,
        r#"True False"#,
    ),
    (
        r#"a=depset([1,2])
print(a==depset([1,2]), a==a, a!=depset([1]))"#,
        r#"False True True"#,
    ),
    (
        r#"a=depset([1,2])
print(dir(a))"#,
        r#"["to_list"]"#,
    ),
    (
        r#"a=depset([1,2])
print(a.to_list)"#,
        r#"<built-in method to_list of depset value>"#,
    ),
    (r#"print(depset([1],transitive=None))"#, r#"depset([1])"#),
    (
        r#"print(depset([1],"default",transitive=[depset([2])]))"#,
        r#"depset([2, 1])"#,
    ),
    (
        r#"print(depset([1],order="default",transitive=[depset([2],order="topological")]).to_list())"#,
        r#"[2, 1]"#,
    ),
    (
        r#"print(depset([1],order="topological",transitive=[depset([2])]).to_list())"#,
        r#"[1, 2]"#,
    ),
    (
        r#"print(depset([1],order="postorder",transitive=[depset([2])]).to_list())"#,
        r#"[2, 1]"#,
    ),
    (
        r#"print(depset(transitive=[depset([2],order="postorder"),depset([3],order="preorder")]))"#,
        r#"depset([2, 3])"#,
    ),
    (
        r#"print(depset(transitive=[depset([2],order="postorder"),depset([3])]))"#,
        r#"depset([2, 3])"#,
    ),
    (
        r#"print(depset(transitive=[depset([2],order="postorder"),depset([3])],order="postorder"))"#,
        r#"depset([2, 3], order = "postorder")"#,
    ),
    (
        r#"print(depset([1],order="postorder",transitive=[depset([2])]))"#,
        r#"depset([2, 1], order = "postorder")"#,
    ),
    (
        r#"A=depset(["x","a"],order="topological")
B=depset(["b","x"],order="topological")
C=depset(["c"],transitive=[A,B],order="topological")
print(C.to_list())"#,
        r#"["c", "a", "b", "x"]"#,
    ),
    (
        r#"A=depset(["x","a"],order="topological")
C=depset(["c","x"],transitive=[A],order="topological")
print(C.to_list())"#,
        r#"["c", "x", "a"]"#,
    ),
    (
        r#"A=depset(["a1","a2"],order="topological")
C=depset(["c1","c2"],transitive=[A],order="topological")
print(C.to_list())"#,
        r#"["c1", "c2", "a1", "a2"]"#,
    ),
    (
        r#"D=depset(["d1","d2"],order="topological")
A=depset(["a1","a2"],transitive=[D],order="topological")
C=depset(["c1","c2"],transitive=[A],order="topological")
print(C.to_list())"#,
        r#"["c1", "c2", "a1", "a2", "d1", "d2"]"#,
    ),
    (
        r#"D=depset(["d1","d2"],order="topological")
A=depset(["a1","a2"],transitive=[D],order="topological")
B=depset(["b1","b2"],transitive=[D],order="topological")
C=depset(["c1","c2"],transitive=[A,B],order="topological")
print(C.to_list())"#,
        r#"["c1", "c2", "a1", "a2", "b1", "b2", "d1", "d2"]"#,
    ),
    (
        r#"D=depset(["d1","d2"],order="topological")
A=depset(["a1","a2"],transitive=[D],order="topological")
B=depset(["b1","b2"],order="topological")
C=depset(["c1","c2"],transitive=[A,B],order="topological")
E=depset(["e1"],transitive=[C,D],order="topological")
print(E.to_list())"#,
        r#"["e1", "c1", "c2", "a1", "a2", "b1", "b2", "d1", "d2"]"#,
    ),
    (
        r#"A=depset(["a1","a2"],order="topological")
B=depset(["b1","b2"],order="topological")
C=depset(["c1","c2"],transitive=[A,B],order="topological")
D=depset(["d1"],transitive=[B,A],order="topological")
E=depset(["e1"],transitive=[C,D],order="topological")
print(E.to_list())"#,
        r#"["e1", "c1", "c2", "d1", "b1", "b2", "a1", "a2"]"#,
    ),
    (
        r#"A=depset(["p","q"],order="topological")
B=depset(["q","p"],order="topological")
C=depset(transitive=[A,B],order="topological")
print(C.to_list())"#,
        r#"["q", "p"]"#,
    ),
    (
        r#"A=depset(["p","q"],order="topological")
B=depset(["q","r"],order="topological")
C=depset(transitive=[A,B],order="topological")
print(C.to_list())"#,
        r#"["p", "q", "r"]"#,
    ),
    (
        r#"A=depset(["p","q"],order="topological")
B=depset(["q","r"],order="topological")
C=depset(transitive=[B,A],order="topological")
print(C.to_list())"#,
        r#"["r", "p", "q"]"#,
    ),
    (
        r#"A=depset(["p","q"],order="postorder")
B=depset(["q","r"],order="postorder")
C=depset(transitive=[A,B],order="postorder")
print(C.to_list())"#,
        r#"["p", "q", "r"]"#,
    ),
    (
        r#"A=depset(["p","q"],order="preorder")
B=depset(["q","r"],order="preorder")
C=depset(transitive=[A,B],order="preorder")
print(C.to_list())"#,
        r#"["p", "q", "r"]"#,
    ),
    (
        r#"A=depset(["p","q"],order="postorder")
B=depset(["r","p"],order="postorder")
C=depset(["s","q"],transitive=[A,B],order="postorder")
print(C.to_list())"#,
        r#"["p", "q", "r", "s"]"#,
    ),
    (
        r#"A=depset(["p","q"],order="preorder")
B=depset(["r","p"],order="preorder")
C=depset(["s","q"],transitive=[A,B],order="preorder")
print(C.to_list())"#,
        r#"["s", "q", "p", "r"]"#,
    ),
    (
        r#"A=depset(["p","q"])
B=depset(["r","p"])
C=depset(["s","q"],transitive=[A,B])
print(C.to_list())"#,
        r#"["p", "q", "r", "s"]"#,
    ),
    (
        r#"A=depset(["p","q"],order="topological")
B=depset(["r","p"],order="topological")
C=depset(["s","q"],transitive=[A,B],order="topological")
print(C.to_list())"#,
        r#"["s", "q", "r", "p"]"#,
    ),
    (
        r#"A=depset(["p","p"],order="topological")
print(A.to_list())"#,
        r#"["p"]"#,
    ),
    (
        r#"A=depset(["p","q","p"],order="topological")
print(A.to_list())"#,
        r#"["p", "q"]"#,
    ),
    (
        r#"A=depset(["p","q","p"],order="preorder")
print(A.to_list())"#,
        r#"["p", "q"]"#,
    ),
    (
        r#"A=depset(["p","q","p"],order="postorder")
print(A.to_list())"#,
        r#"["p", "q"]"#,
    ),
    (
        r#"A=depset(["p","q"])
B=depset(["p","q"])
C=depset(transitive=[A,B])
print(C.to_list())"#,
        r#"["p", "q"]"#,
    ),
    (
        r#"A=depset(["p","q"])
C=depset(transitive=[A,A,A])
print(C.to_list())"#,
        r#"["p", "q"]"#,
    ),
    (
        r#"A=depset()
C=depset(transitive=[A])
print(C.to_list(), C)"#,
        r#"[] depset([])"#,
    ),
    (
        r#"A=depset(["p"])
C=depset(transitive=[A])
print(C == A, C.to_list())"#,
        r#"True ["p"]"#,
    ),
    (r#"print(depset([None,None]).to_list())"#, r#"[None]"#),
    (
        r#"T=depset(["t1","t2"],order="topological")
R=depset(["r1","r2"],transitive=[T])
print(R.to_list(), R)"#,
        r#"["t2", "t1", "r1", "r2"] depset(["t2", "t1", "r1", "r2"])"#,
    ),
    (
        r#"T=depset(["t1","t2"],order="topological")
R=depset(["r1","r2"],transitive=[T],order="default")
print(R.to_list())"#,
        r#"["t2", "t1", "r1", "r2"]"#,
    ),
    (
        r#"T=depset(["t1","t2"],order="preorder")
R=depset(["r1","r2"],transitive=[T])
print(R.to_list(), R)"#,
        r#"["t1", "t2", "r1", "r2"] depset(["t1", "t2", "r1", "r2"])"#,
    ),
    (
        r#"T=depset(["t1","t2"],order="postorder")
R=depset(["r1","r2"],transitive=[T])
print(R.to_list(), R)"#,
        r#"["t1", "t2", "r1", "r2"] depset(["t1", "t2", "r1", "r2"])"#,
    ),
    (
        r#"G=depset(["g1","g2"],order="topological")
T=depset(["t1","t2"],transitive=[G],order="topological")
R=depset(["r1","r2"],transitive=[T])
print(R.to_list())"#,
        r#"["g2", "g1", "t2", "t1", "r1", "r2"]"#,
    ),
    (
        r#"G=depset(["g1","g2"],order="topological")
T=depset(["t1","t2"],transitive=[G],order="topological")
R=depset(["r1","r2"],transitive=[T,G])
print(R.to_list())"#,
        r#"["g2", "g1", "t2", "t1", "r1", "r2"]"#,
    ),
    (
        r#"G=depset(["g1","g2"])
T=depset(["t1","t2"],transitive=[G])
R=depset(["r1","r2"],transitive=[T],order="topological")
print(R.to_list())"#,
        r#"["r1", "r2", "t2", "t1", "g2", "g1"]"#,
    ),
    (
        r#"G=depset(["g1","g2"])
T=depset(["t1","t2"],transitive=[G])
R=depset(["r1","r2"],transitive=[T],order="preorder")
print(R.to_list())"#,
        r#"["r1", "r2", "g1", "g2", "t1", "t2"]"#,
    ),
    (
        r#"G=depset(["g1","g2"])
T=depset(["t1","t2"],transitive=[G])
R=depset(["r1","r2"],transitive=[T],order="postorder")
print(R.to_list())"#,
        r#"["g1", "g2", "t1", "t2", "r1", "r2"]"#,
    ),
    (
        r#"G=depset(["g1","g2"])
T=depset(["t1","t2"],transitive=[G])
R=depset(["r1","r2"],transitive=[T,G],order="topological")
print(R.to_list())"#,
        r#"["r1", "r2", "t2", "t1", "g2", "g1"]"#,
    ),
    (
        r#"G=depset(["x","g2"])
T=depset(["t1","x"],transitive=[G])
R=depset(["r1","x"],transitive=[T,G],order="topological")
print(R.to_list())"#,
        r#"["r1", "t1", "g2", "x"]"#,
    ),
    (
        r#"G=depset(["x","g2"])
T=depset(["t1","x"],transitive=[G])
R=depset(["r1","x"],transitive=[T,G],order="preorder")
print(R.to_list())"#,
        r#"["r1", "x", "g2", "t1"]"#,
    ),
    (
        r#"G=depset(["x","g2"])
T=depset(["t1","x"],transitive=[G])
R=depset(["r1","x"],transitive=[T,G],order="postorder")
print(R.to_list())"#,
        r#"["x", "g2", "t1", "r1"]"#,
    ),
    (
        r#"G=depset(["x","g2"])
T=depset(["t1","x"],transitive=[G])
R=depset(["r1","x"],transitive=[T,G])
print(R.to_list())"#,
        r#"["x", "g2", "t1", "r1"]"#,
    ),
    (
        r#"G=depset(["x","g2"])
T=depset(["t1","x"],transitive=[G])
R=depset(["r1","x"],transitive=[G,T])
print(R.to_list())"#,
        r#"["x", "g2", "t1", "r1"]"#,
    ),
    (
        r#"G=depset(["g1","g2"],order="preorder")
T=depset(["t1","t2"],transitive=[G],order="preorder")
R=depset(["r1","r2"],transitive=[T])
print(R.to_list())"#,
        r#"["t1", "t2", "g1", "g2", "r1", "r2"]"#,
    ),
    (
        r#"G=depset(["g1","g2"],order="postorder")
T=depset(["t1","t2"],transitive=[G],order="postorder")
R=depset(["r1","r2"],transitive=[T])
print(R.to_list())"#,
        r#"["g1", "g2", "t1", "t2", "r1", "r2"]"#,
    ),
    (
        r#"G=depset(["g1","g2"],order="topological")
T=depset(["t1","t2"],transitive=[G])
print(T.to_list(), T)"#,
        r#"["g2", "g1", "t1", "t2"] depset(["g2", "g1", "t1", "t2"])"#,
    ),
    (
        r#"G=depset(["g1","g2"],order="topological")
T=depset(["t1","t2"],transitive=[G],order="topological")
print(T.to_list(), T)"#,
        r#"["t1", "t2", "g1", "g2"] depset(["t1", "t2", "g1", "g2"], order = "topological")"#,
    ),
    (
        r#"A=depset(["p"])
C=depset(transitive=[A],order="postorder")
print(C==A, C)"#,
        r#"False depset(["p"], order = "postorder")"#,
    ),
    (
        r#"A=depset(["p"],order="postorder")
C=depset(transitive=[A])
print(C==A, C)"#,
        r#"False depset(["p"])"#,
    ),
    (
        r#"A=depset(["p"],order="postorder")
C=depset(transitive=[A],order="postorder")
print(C==A, C)"#,
        r#"True depset(["p"], order = "postorder")"#,
    ),
    (
        r#"A=depset(["p"])
C=depset(["q"],transitive=[A])
D=depset(transitive=[C])
print(D==C, D==A)"#,
        r#"True False"#,
    ),
    (
        r#"A=depset(["p"])
C=depset(transitive=[A, depset()])
print(C==A)"#,
        r#"True"#,
    ),
    (
        r#"A=depset(["p"])
C=depset(transitive=[depset(), A])
print(C==A)"#,
        r#"True"#,
    ),
    (
        r#"A=depset(["p"])
C=depset([], transitive=[A])
print(C==A)"#,
        r#"True"#,
    ),
    (
        r#"print(depset()==depset(), depset(order="postorder")==depset(), depset(order="postorder")==depset(order="postorder"))"#,
        r#"True False True"#,
    ),
    (r#"print(depset(["p"])==depset(["p"]))"#, r#"False"#),
    (r#"print({depset():1})"#, r#"{depset([]): 1}"#),
    (r#"print(depset([depset()]))"#, r#"depset([depset([])])"#),
    (
        r#"d=depset(["p"])
print(d in [d], [d].index(d))"#,
        r#"True 0"#,
    ),
    (r#"print(depset)"#, r#"<built-in function depset>"#),
    (r#"print(type(depset(["a"]).to_list()))"#, r#"list"#),
    (
        r#"x=depset(["a"]).to_list()
x.append("b")
print(x)"#,
        r#"["a", "b"]"#,
    ),
    (
        r#"print(depset(direct=None,order="postorder",transitive=None))"#,
        r#"depset([], order = "postorder")"#,
    ),
    (
        r#"print(depset(order="postorder"), depset(order="topological"))"#,
        r#"depset([], order = "postorder") depset([], order = "topological")"#,
    ),
    (
        r#"print(depset([1,2,3],order="topological"))"#,
        r#"depset([1, 2, 3], order = "topological")"#,
    ),
    (
        r#"print(depset(["a"]) if depset() else "empty")"#,
        r#"empty"#,
    ),
    (
        r#"print(bool(depset()), bool(depset(transitive=[depset()])))"#,
        r#"False False"#,
    ),
    (
        r#"print(str(depset(["a"], order="preorder")), repr(depset()))"#,
        r#"depset(["a"], order = "preorder") depset([])"#,
    ),
    (
        r#"print(depset([("a",1),("b",2)]))"#,
        r#"depset([("a", 1), ("b", 2)])"#,
    ),
    (r#"print(depset([1.5, 2.5]))"#, r#"depset([1.5, 2.5])"#),
    (
        r#"print(depset([depset(["a"])]).to_list())"#,
        r#"[depset(["a"])]"#,
    ),
    (
        r#"print(depset([depset(["a"]), depset(["b"])]).to_list())"#,
        r#"[depset(["a"]), depset(["b"])]"#,
    ),
];

/// `(source, Bazel 9.2.0's error message)`.
const ERRORS: &[(&str, &str)] = &[
    (
        r#"a=depset([1,2])
print(len(a))"#,
        r#"in call to len(), parameter 'x' got value of type 'depset', want 'iterable or string'"#,
    ),
    (
        r#"a=depset([1,2])
print([x for x in a])"#,
        r#"type 'depset' is not iterable"#,
    ),
    (
        r#"a=depset([1,2])
print(1 in a)"#,
        r#"unsupported binary operation: int in depset"#,
    ),
    (
        r#"a=depset([1,2])
print(a+a)"#,
        r#"unsupported binary operation: depset + depset"#,
    ),
    (
        r#"a=depset([1,2])
print(a|a)"#,
        r#"unsupported binary operation: depset | depset"#,
    ),
    (
        r#"a=depset([1,2])
print(a+[3])"#,
        r#"unsupported binary operation: depset + list"#,
    ),
    (
        r#"a=depset([1,2])
print(hash(a))"#,
        r#"in call to hash(), parameter 'value' got value of type 'depset', want 'string'"#,
    ),
    (
        r#"a=depset([1,2])
print(a[0])"#,
        r#"type 'depset' has no operator [](int)"#,
    ),
    (
        r#"print(depset([[1]]))"#,
        r#"depset elements must not be mutable values"#,
    ),
    (
        r#"print(depset([{}]))"#,
        r#"depset elements must not be mutable values"#,
    ),
    (
        r#"print(depset([1,"a"]))"#,
        r#"cannot add an item of type 'string' to a depset of 'int'"#,
    ),
    (
        r#"print(depset([1],transitive=[depset(["a"])]))"#,
        r#"cannot add an item of type 'string' to a depset of 'int'"#,
    ),
    (
        r#"print(depset([1],transitive=[depset(["a"])]).to_list())"#,
        r#"cannot add an item of type 'string' to a depset of 'int'"#,
    ),
    (
        r#"print(depset(transitive=[depset(["a"]), depset([1])]))"#,
        r#"cannot add an item of type 'int' to a depset of 'string'"#,
    ),
    (
        r#"print(depset([1],order="bogus"))"#,
        r#"Invalid order: bogus"#,
    ),
    (
        r#"print(depset([1],order="stable"))"#,
        r#"Invalid order: stable"#,
    ),
    (
        r#"print(depset([1],order="compile"))"#,
        r#"Invalid order: compile"#,
    ),
    (
        r#"print(depset([1],order="link"))"#,
        r#"Invalid order: link"#,
    ),
    (
        r#"print(depset([1],order="naive_link"))"#,
        r#"Invalid order: naive_link"#,
    ),
    (
        r#"print(depset([1],order=None))"#,
        r#"in call to depset(), parameter 'order' got value of type 'NoneType', want 'string'"#,
    ),
    (
        r#"print(depset([1],order=1))"#,
        r#"in call to depset(), parameter 'order' got value of type 'int', want 'string'"#,
    ),
    (
        r#"print(depset(1))"#,
        r#"in call to depset(), parameter 'direct' got value of type 'int', want 'sequence or NoneType'"#,
    ),
    (
        r#"print(depset("abc"))"#,
        r#"in call to depset(), parameter 'direct' got value of type 'string', want 'sequence or NoneType'"#,
    ),
    (
        r#"print(depset(depset([1])))"#,
        r#"in call to depset(), parameter 'direct' got value of type 'depset', want 'sequence or NoneType'"#,
    ),
    (
        r#"print(depset([1],transitive=depset([1])))"#,
        r#"in call to depset(), parameter 'transitive' got value of type 'depset', want 'sequence or NoneType'"#,
    ),
    (
        r#"print(depset([1],transitive=[1]))"#,
        r#"at index 0 of transitive, got element of type int, want depset"#,
    ),
    (
        r#"print(depset([1],transitive=[[1]]))"#,
        r#"at index 0 of transitive, got element of type list, want depset"#,
    ),
    (
        r#"print(depset(items=[1]))"#,
        r#"depset() got unexpected keyword argument 'items'"#,
    ),
    (
        r#"print(depset([1],[2]))"#,
        r#"in call to depset(), parameter 'order' got value of type 'list', want 'string'"#,
    ),
    (
        r#"print(depset([1],"default",[depset([2])]))"#,
        r#"depset() accepts no more than 2 positional arguments but got 3"#,
    ),
    (
        r#"print(depset([1],"default","x"))"#,
        r#"depset() accepts no more than 2 positional arguments but got 3"#,
    ),
    (
        r#"print(depset(direct=[1],direct2=[2]))"#,
        r#"depset() got unexpected keyword argument 'direct2' (did you mean 'direct'?)"#,
    ),
    (
        r#"print(depset([1],order="postorder",transitive=[depset([2],order="preorder")]))"#,
        r#"Order 'postorder' is incompatible with order 'preorder'"#,
    ),
    (
        r#"print(depset([1],order="postorder",transitive=[depset([2],order="topological")]))"#,
        r#"Order 'postorder' is incompatible with order 'topological'"#,
    ),
    (
        r#"print(depset([1],order="topological",transitive=[depset([2],order="postorder")]))"#,
        r#"Order 'topological' is incompatible with order 'postorder'"#,
    ),
    (
        r#"print(depset([1, "a"]))"#,
        r#"cannot add an item of type 'string' to a depset of 'int'"#,
    ),
    (
        r#"print(depset([None, True, 1.5, (1,2), "s"]).to_list())"#,
        r#"cannot add an item of type 'bool' to a depset of 'NoneType'"#,
    ),
    (
        r#"print(depset([(1,[2])]))"#,
        r#"depset elements must not be mutable values"#,
    ),
    (
        r#"print(depset([1,True,1.0]).to_list())"#,
        r#"cannot add an item of type 'bool' to a depset of 'int'"#,
    ),
    (
        r#"print(depset([0,False]).to_list())"#,
        r#"cannot add an item of type 'bool' to a depset of 'int'"#,
    ),
    (
        r#"print(depset([True,1]))"#,
        r#"cannot add an item of type 'int' to a depset of 'bool'"#,
    ),
    (
        r#"print(depset(["p"]) < depset(["q"]))"#,
        r#"unsupported comparison: depset <=> depset"#,
    ),
    (
        r#"print(depset(["a"]).to_list(1))"#,
        r#"to_list() got unexpected positional argument"#,
    ),
    (
        r#"print(depset(["a"]).to_list(x=1))"#,
        r#"to_list() got unexpected keyword argument 'x'"#,
    ),
    (
        r#"print(depset(["a"]).foo)"#,
        r#"'depset' value has no field or method 'foo'"#,
    ),
    (
        r#"print(depset(["a"]).to_list().to_list())"#,
        r#"'list' value has no field or method 'to_list'"#,
    ),
    (
        r#"print(depset([depset(["a"]), 1]))"#,
        r#"cannot add an item of type 'int' to a depset of 'depset'"#,
    ),
    (
        r#"print(depset([len]))"#,
        r#"depset elements must not be mutable values"#,
    ),
    (
        r#"print(depset([len, print]).to_list())"#,
        r#"depset elements must not be mutable values"#,
    ),
];
