import Fjfj.Engine
import Fjfj.Publish
import Fjfj.Daemon
import Fjfj.ActionKey
import Fjfj.Dynamic
import Fjfj.Persistence
import Fjfj.Configuration
/-!
# fjfj architecture specification

Machine-checked record of fjfj's architecture. Each module constrains one
crate or protocol; `lake build` in CI fails if the spec is inconsistent.
Specs live at the level of keys, values, edges and protocols, not Rust code.

Decisions recorded here (with beads ids) are the source of truth; the
markdown in `docs/design/` explains them in prose.
-/
