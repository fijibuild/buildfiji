/-!
# REAPI action key (crate `fjfj-remote`, `action_key.rs`; beads
buildfiji-c71.5, buildfiji-c71.9)

The action key is the SHA-256 of the canonical `Action` message. fjfj and
Bazel 9.2.0 share cache entries iff their encodings agree byte for byte.
This module records the canonical-form invariants; the Rust tests check
them against blobs Bazel itself wrote.
-/
namespace Fjfj.ActionKey

/-- A name is a valid single path component. -/
def ValidName (n : String) : Prop :=
  n ≠ "" ∧ n ≠ "." ∧ n ≠ ".." ∧ ¬ n.contains '/'

/-- A list of names is canonical when strictly increasing (sorted, no
duplicates). This is required of `Directory.files`, `.directories` and
`.symlinks`, of `Command.environment_variables`, `output_paths` and
`Platform.properties`. -/
def Canonical (names : List String) : Prop :=
  names.Pairwise (· < ·)

/-- Bazel fact: every input file is encoded with `is_executable = true`,
so the key is independent of filesystem mode. -/
structure FileNode where
  name : String
  hash : String
  size : Nat
  isExecutable : Bool

def BazelFile (f : FileNode) : Prop := f.isExecutable = true

/-- Bazel fact: `Action.salt` carries `CacheSalt{may_be_executed_remotely}`
and is never empty. -/
structure CacheSalt where
  mayBeExecutedRemotely : Bool
  workspace : String

/-- A canonical directory: sorted unique valid names in each list, and no
name shared between files, directories and symlinks. -/
structure Directory where
  files : List String
  dirs  : List String
  links : List String
  filesCanonical : Canonical files
  dirsCanonical  : Canonical dirs
  linksCanonical : Canonical links
  disjoint : ∀ n, ¬ (n ∈ files ∧ n ∈ dirs) ∧ ¬ (n ∈ files ∧ n ∈ links) ∧ ¬ (n ∈ dirs ∧ n ∈ links)

/-- The empty directory is canonical; its digest is the SHA-256 of the
empty string (`e3b0c442…`). -/
def emptyDirectory : Directory :=
  { files := [], dirs := [], links := [],
    filesCanonical := List.Pairwise.nil, dirsCanonical := List.Pairwise.nil,
    linksCanonical := List.Pairwise.nil,
    disjoint := fun _ => ⟨(fun h => nomatch h.1), (fun h => nomatch h.1), (fun h => nomatch h.1)⟩ }

end Fjfj.ActionKey
