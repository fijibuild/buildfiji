//! What a `test_suite` stands for (probed on Bazel 9.2.0, `query tests()` and
//! `build`):
//!
//! - `tests` lists tests and suites; with none listed it is every test rule
//!   of the package that is not tagged `manual`;
//! - `tags` filters the tests the suite itself names (listed or the package's,
//!   not what a nested suite expands to): a plain tag must be among the test's
//!   tags or be its size, a tag with `-` must not be, and a plain `manual`
//!   asks for nothing;
//! - a suite is in a cycle or lists a rule that is neither a test nor a suite
//!   is an error.

use crate::Label;
use std::collections::BTreeSet;

/// What the filter of a suite reads of a test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteTest {
    pub tags: Vec<String>,
    /// `small`, `medium`, `large` or `enormous`: the size counts as a tag.
    pub size: String,
}

/// What a label in a suite is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Member {
    Test(SuiteTest),
    Suite {
        /// The `tests` attribute.
        tests: Vec<Label>,
        /// The `tags` attribute.
        tags: Vec<String>,
    },
    /// Anything else, or nothing by that name.
    Other,
}

/// Where the suites are.
pub trait Suites {
    fn member(&self, label: &Label) -> Result<Member, String>;
    /// The test rules of the package of `label`, `manual` ones among them
    /// (the tags say which), in the order Bazel lists them: by name.
    fn package_tests(&self, label: &Label) -> Result<Vec<(Label, SuiteTest)>, String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SuiteError {
    /// The suite listed `member`, which is neither a test nor a suite: the
    /// suite, then the member.
    NotATest(Box<(Label, Label)>),
    /// The suites that lead back to the first one, in order.
    Cycle(Vec<Label>),
    Other(String),
}

fn passes(tags: &[String], test: &SuiteTest) -> bool {
    let has = |tag: &str| test.size == tag || test.tags.iter().any(|t| t == tag);
    tags.iter().all(|tag| match tag.strip_prefix('-') {
        Some(excluded) => !has(excluded),
        // Asking for `manual` asks for nothing.
        None => tag == "manual" || has(tag),
    })
}

/// The tests `suite` stands for, each once, in the order they were found.
/// `strict` is whether what is no test, and a cycle, is an error: `build`
/// says so, `query tests()` leaves such a member out.
pub fn expand(suites: &dyn Suites, suite: &Label, strict: bool) -> Result<Vec<Label>, SuiteError> {
    let mut out = Vec::new();
    let mut on_path = vec![suite.clone()];
    walk(suites, suite, strict, &mut on_path, &mut out)?;
    let mut seen = BTreeSet::new();
    out.retain(|l| seen.insert(l.clone()));
    Ok(out)
}

fn walk(
    suites: &dyn Suites,
    suite: &Label,
    strict: bool,
    on_path: &mut Vec<Label>,
    out: &mut Vec<Label>,
) -> Result<(), SuiteError> {
    let Member::Suite { tests, tags } = suites.member(suite).map_err(SuiteError::Other)? else {
        return Err(SuiteError::Other(format!("{suite} is not a test_suite")));
    };
    let listed: Vec<(Label, Member)> = if tests.is_empty() {
        suites
            .package_tests(suite)
            .map_err(SuiteError::Other)?
            .into_iter()
            // Without a list, what is tagged `manual` is not in the suite.
            .filter(|(_, t)| !t.tags.iter().any(|tag| tag == "manual"))
            .map(|(label, test)| (label, Member::Test(test)))
            .collect()
    } else {
        tests
            .iter()
            .map(|l| Ok((l.clone(), suites.member(l).map_err(SuiteError::Other)?)))
            .collect::<Result<_, SuiteError>>()?
    };
    for (label, member) in listed {
        match member {
            Member::Test(test) => {
                if passes(&tags, &test) {
                    out.push(label);
                }
            }
            Member::Suite { .. } => {
                if let Some(at) = on_path.iter().position(|l| *l == label) {
                    if !strict {
                        continue;
                    }
                    let mut path = on_path[at..].to_vec();
                    path.push(label);
                    return Err(SuiteError::Cycle(path));
                }
                on_path.push(label.clone());
                walk(suites, &label, strict, on_path, out)?;
                on_path.pop();
            }
            Member::Other if strict => {
                return Err(SuiteError::NotATest(Box::new((suite.clone(), label))));
            }
            Member::Other => {}
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    struct World(BTreeMap<String, Member>);

    fn label(name: &str) -> Label {
        Label {
            repo: String::new(),
            package: "p".into(),
            name: name.into(),
        }
    }

    fn test(tags: &[&str], size: &str) -> Member {
        Member::Test(SuiteTest {
            tags: tags.iter().map(|s| s.to_string()).collect(),
            size: size.into(),
        })
    }

    fn suite(tests: &[&str], tags: &[&str]) -> Member {
        Member::Suite {
            tests: tests.iter().map(|n| label(n)).collect(),
            tags: tags.iter().map(|s| s.to_string()).collect(),
        }
    }

    impl Suites for World {
        fn member(&self, l: &Label) -> Result<Member, String> {
            Ok(self.0.get(&l.name).cloned().unwrap_or(Member::Other))
        }

        fn package_tests(&self, _: &Label) -> Result<Vec<(Label, SuiteTest)>, String> {
            Ok(self
                .0
                .iter()
                .filter_map(|(n, m)| match m {
                    Member::Test(t) => Some((label(n), t.clone())),
                    _ => None,
                })
                .collect())
        }
    }

    /// The package of the probe on Bazel 9.2.0: four tests, `b_test` manual.
    fn world() -> World {
        let mut w = BTreeMap::new();
        w.insert("a_test".to_owned(), test(&["fast"], "small"));
        w.insert("b_test".to_owned(), test(&["slow", "manual"], "large"));
        w.insert("c_test".to_owned(), test(&["fast", "flaky"], "medium"));
        w.insert("d_test".to_owned(), test(&["fast", "slow"], "medium"));
        for (name, tests, tags) in [
            ("all", &[][..], &[][..]),
            ("fast", &[], &["fast"]),
            ("explicit", &["a_test", "b_test"], &[]),
            (
                "explicit_tags",
                &["a_test", "b_test", "c_test", "d_test"],
                &["slow"],
            ),
            (
                "explicit_neg",
                &["a_test", "b_test", "c_test", "d_test"],
                &["-flaky"],
            ),
            ("neg", &[], &["-slow", "fast"]),
            ("neg_only", &[], &["-slow"]),
            ("manual_tag", &[], &["manual"]),
            ("nested", &["explicit", "fast"], &[]),
            ("nested_tags", &["fast"], &["slow"]),
            ("two_tags", &[], &["fast", "flaky"]),
            ("small", &[], &["small"]),
            ("medium", &[], &["medium"]),
            ("large", &[], &["large"]),
            ("not_manual", &[], &["-manual"]),
            ("explicit_not_manual", &["a_test", "b_test"], &["-manual"]),
            ("explicit_fast", &["a_test", "b_test"], &["fast"]),
            ("cycle1", &["cycle2"], &[]),
            ("cycle2", &["cycle1"], &[]),
            ("itself", &["itself"], &[]),
            ("bad", &["gen"], &[]),
        ] {
            w.insert(name.to_owned(), suite(tests, tags));
        }
        World(w)
    }

    fn names(found: Result<Vec<Label>, SuiteError>) -> Vec<String> {
        found.unwrap().into_iter().map(|l| l.name).collect()
    }

    #[test]
    fn a_suite_stands_for_the_tests_bazel_gives_it() {
        let w = world();
        for (suite, want) in [
            ("all", &["a_test", "c_test", "d_test"][..]),
            ("fast", &["a_test", "c_test", "d_test"]),
            ("explicit", &["a_test", "b_test"]),
            ("explicit_tags", &["b_test", "d_test"]),
            ("explicit_neg", &["a_test", "b_test", "d_test"]),
            ("neg", &["a_test", "c_test"]),
            ("neg_only", &["a_test", "c_test"]),
            ("manual_tag", &["a_test", "c_test", "d_test"]),
            ("nested", &["a_test", "b_test", "c_test", "d_test"]),
            ("nested_tags", &["a_test", "c_test", "d_test"]),
            ("two_tags", &["c_test"]),
            ("small", &["a_test"]),
            ("medium", &["c_test", "d_test"]),
            ("large", &[]),
            ("not_manual", &["a_test", "c_test", "d_test"]),
            ("explicit_not_manual", &["a_test"]),
            ("explicit_fast", &["a_test"]),
        ] {
            assert_eq!(names(expand(&w, &label(suite), true)), want, "{suite}");
        }
    }

    #[test]
    fn a_query_leaves_out_what_a_build_refuses() {
        let w = world();
        for suite in ["bad", "cycle1", "itself"] {
            assert!(
                names(expand(&w, &label(suite), false)).is_empty(),
                "{suite}"
            );
        }
    }

    #[test]
    fn a_suite_that_lists_a_non_test_or_itself_is_an_error() {
        let w = world();
        assert_eq!(
            expand(&w, &label("bad"), true),
            Err(SuiteError::NotATest(Box::new((label("bad"), label("gen")))))
        );
        assert_eq!(
            expand(&w, &label("cycle1"), true),
            Err(SuiteError::Cycle(vec![
                label("cycle1"),
                label("cycle2"),
                label("cycle1")
            ]))
        );
        assert_eq!(
            expand(&w, &label("itself"), true),
            Err(SuiteError::Cycle(vec![label("itself"), label("itself")]))
        );
    }
}
