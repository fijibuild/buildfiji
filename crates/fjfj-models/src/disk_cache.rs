//! Shared `--disk_cache` directory with concurrent writers and a garbage
//! collector (bead buildfiji-2h9.6; crate `fjfj-remote`, `DiskCache`).
//!
//! Bazel and fjfj may share one disk cache. Layout is Bazel's: `cas/<hash>`
//! and `ac/<hash>`, each written as temp + rename. Writers are idempotent
//! for the CAS (same digest, same bytes). A GC may delete a CAS blob at any
//! time, including after a reader has seen the AC entry that references it.
//!
//! Rules encoded:
//! - Writes are temp + fsync + rename, so a reader sees a blob complete or
//!   not at all (from the `publish` model).
//! - An AC hit is only a hit if every referenced CAS blob is present at use
//!   time; otherwise treat as a miss and re-execute. With
//!   `verify_cas_on_hit = false` the checker finds a hit whose blob is gone.
//! - GC deletes AC entries before the CAS blobs they reference, so an
//!   entry never outlives its blobs for long; readers still verify.

use stateright::{Model, Property};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Blob {
    Absent,
    /// Temp file being written; invisible under the final name.
    Temp,
    Present,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Reader {
    Idle,
    /// Saw the AC entry; about to use the blob.
    SawAc,
    Hit,
    Miss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct State {
    pub cas: Blob,
    pub ac: Blob,
    /// Two writers (bazel, fjfj) producing the same action result.
    pub writer_done: [bool; 2],
    pub reader: Reader,
    /// A hit that was consumed while the blob was absent: corruption.
    pub bad_hit: bool,
    pub gcs: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Action {
    WriteCasTemp(usize),
    RenameCas(usize),
    WriteAcTemp(usize),
    RenameAc(usize),
    ReadAc,
    UseBlob,
    GcAc,
    GcCas,
}

#[derive(Clone, Copy, Debug)]
pub struct DiskCache {
    pub verify_cas_on_hit: bool,
    pub max_gcs: u8,
}

impl Model for DiskCache {
    type State = State;
    type Action = Action;

    fn init_states(&self) -> Vec<State> {
        vec![State {
            cas: Blob::Absent,
            ac: Blob::Absent,
            writer_done: [false; 2],
            reader: Reader::Idle,
            bad_hit: false,
            gcs: 0,
        }]
    }

    fn actions(&self, s: &State, actions: &mut Vec<Action>) {
        for w in 0..2 {
            if s.writer_done[w] {
                continue;
            }
            match s.cas {
                Blob::Absent => actions.push(Action::WriteCasTemp(w)),
                Blob::Temp => actions.push(Action::RenameCas(w)),
                Blob::Present => match s.ac {
                    Blob::Absent => actions.push(Action::WriteAcTemp(w)),
                    Blob::Temp => actions.push(Action::RenameAc(w)),
                    Blob::Present => {}
                },
            }
        }
        match s.reader {
            Reader::Idle if s.ac == Blob::Present => actions.push(Action::ReadAc),
            Reader::Idle if s.writer_done.iter().all(|&d| d) => actions.push(Action::ReadAc),
            Reader::SawAc => actions.push(Action::UseBlob),
            _ => {}
        }
        if s.gcs < self.max_gcs {
            if s.ac == Blob::Present {
                actions.push(Action::GcAc);
            } else if s.cas == Blob::Present {
                // AC first, then CAS: a blob is never removed while an entry references it.
                actions.push(Action::GcCas);
            }
        }
    }

    fn next_state(&self, s: &State, a: Action) -> Option<State> {
        let mut t = *s;
        match a {
            Action::WriteCasTemp(_) => t.cas = Blob::Temp,
            Action::RenameCas(_) => t.cas = Blob::Present,
            Action::WriteAcTemp(_) => t.ac = Blob::Temp,
            Action::RenameAc(w) => {
                t.ac = Blob::Present;
                // Both writers consider themselves done once the entry exists.
                t.writer_done = [true; 2];
                let _ = w;
            }
            Action::ReadAc => {
                t.reader = if s.ac == Blob::Present {
                    Reader::SawAc
                } else {
                    Reader::Miss
                };
            }
            Action::UseBlob => {
                let present = s.cas == Blob::Present;
                if present {
                    t.reader = Reader::Hit;
                } else if self.verify_cas_on_hit {
                    t.reader = Reader::Miss;
                } else {
                    t.reader = Reader::Hit;
                    t.bad_hit = true;
                }
            }
            Action::GcAc => {
                t.ac = Blob::Absent;
                t.gcs += 1;
            }
            Action::GcCas => {
                t.cas = Blob::Absent;
                t.gcs += 1;
            }
        }
        Some(t)
    }

    fn properties(&self) -> Vec<Property<Self>> {
        vec![
            Property::always("no hit without blob", |_, s: &State| !s.bad_hit),
            Property::always("reader never sees temp", |_, s: &State| {
                // The reader only transitions on Present/Absent, never Temp.
                s.reader != Reader::SawAc || s.ac != Blob::Temp
            }),
            Property::always("quiescence is settlement", |m: &DiskCache, s: &State| {
                let mut enabled = Vec::new();
                m.actions(s, &mut enabled);
                !enabled.is_empty() || matches!(s.reader, Reader::Hit | Reader::Miss)
            }),
            Property::sometimes("hit", |_, s: &State| s.reader == Reader::Hit),
            Property::sometimes("miss after gc", |_, s: &State| {
                s.reader == Reader::Miss && s.gcs > 0
            }),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stateright::Checker;

    #[test]
    fn verified_hits_are_safe_under_concurrent_gc() {
        let checker = DiskCache {
            verify_cas_on_hit: true,
            max_gcs: 2,
        }
        .checker()
        .spawn_bfs()
        .join();
        checker.assert_properties();
    }

    #[test]
    fn trusting_ac_without_cas_check_serves_missing_blob() {
        let checker = DiskCache {
            verify_cas_on_hit: false,
            max_gcs: 2,
        }
        .checker()
        .spawn_bfs()
        .join();
        assert!(checker.discovery("no hit without blob").is_some());
    }
}
