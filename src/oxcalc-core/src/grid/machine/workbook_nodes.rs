//! The workbook calculation node space and the sheet-identity routing
//! invariant (W062 D3 §2, bead `calc-5kqg.32` / R4.4).
//!
//! Three constituent node kinds — grid cells (sheet-qualified via
//! [`ExcelGridCellAddress`]), scoped names ([`ScopedNameKey`] at the workbook
//! layer), and tree nodes ([`TreeNodeId`]) — are unified under
//! [`WorkbookCalcNodeId`]. This is the total node space over which the
//! workbook dirty closure (R4.6) and the workbook oracle (R4.5) will operate;
//! this bead lands the space and its constructors only.
//!
//! The other half of R4.4 is the **routing invariant**: sheet identity on
//! [`ExcelGridCellAddress`]/[`GridRect`] — carried since the grid's inception
//! but never consulted in dependency routing — becomes *authoritative*. An
//! edge whose address names a different sheet than the per-sheet graph it is
//! being registered into is rejected at registration with a typed
//! [`GridRefError::ForeignSheetDependency`], closing the "identity carried but
//! never consulted" gap (D3 §2). The enforcement point is the per-sheet
//! [`GridDependencyIndex`](super::invalidation) registration path; see
//! [`OwningSheetIdentity`] below for the identity a per-sheet index is stamped
//! with.

use std::cell::Cell;

use oxfunc_core::functions::rand_fn::RandomProvider;

use crate::grid::coords::ExcelGridCellAddress;
use crate::grid::geometry::GridRect;
use crate::structural::TreeNodeId;
use crate::workbook_settings::WorkbookSettingChanged;

/// The scope of a defined name at the workbook layer.
///
/// Excel resolves a bare name token against sheet scope first, then workbook
/// scope (sheet-scope-shadows-workbook-scope precedence; D3 §2.2). D3's
/// contract is that edge registration receives a *scope-resolved* name and
/// registers against the resolved key — this enum is that resolved scope.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NameScope {
    /// Workbook-global scope.
    Workbook,
    /// Scoped to a specific sheet (identified by its stable tree node id;
    /// D1 C1/C8: sheets are Sheet-role children and `TreeNodeId` is the
    /// stable identity).
    Sheet(TreeNodeId),
}

/// A scope-qualified defined-name key at the workbook layer.
///
/// Per-sheet indexes keep their bare `String` name keys (they are sheet-scoped
/// by construction); the workbook layer introduces this scope-qualified key
/// and owns the mapping (D3 §2.2). `normalized` is the resolved, normalized
/// name text.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScopedNameKey {
    pub scope: NameScope,
    pub normalized: String,
}

impl ScopedNameKey {
    /// Construct a workbook-scoped name key.
    #[must_use]
    pub fn workbook(normalized: impl Into<String>) -> Self {
        Self {
            scope: NameScope::Workbook,
            normalized: normalized.into(),
        }
    }

    /// Construct a sheet-scoped name key.
    #[must_use]
    pub fn sheet(sheet: TreeNodeId, normalized: impl Into<String>) -> Self {
        Self {
            scope: NameScope::Sheet(sheet),
            normalized: normalized.into(),
        }
    }
}

/// The workbook calculation node space.
///
/// Grid cells join at cell granularity (carrying full workbook+sheet identity
/// already); scoped names are first-class nodes so name→name and name→cell
/// edges exist; the tree joins at name granularity via [`TreeNodeId`]
/// (D3 §2.1 / §8). This is a *total* space over the three constituent kinds —
/// every workbook-calc node is exactly one of these — which is what lets the
/// closure and oracle range over a single `BTreeSet<WorkbookCalcNodeId>`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum WorkbookCalcNodeId {
    /// A grid cell, at cell granularity, carrying workbook+sheet identity.
    GridCell(ExcelGridCellAddress),
    /// A scoped defined name at the workbook layer.
    Name(ScopedNameKey),
    /// A tree node.
    TreeNode(TreeNodeId),
}

impl WorkbookCalcNodeId {
    /// Construct a grid-cell node.
    #[must_use]
    pub fn grid_cell(address: ExcelGridCellAddress) -> Self {
        Self::GridCell(address)
    }

    /// Construct a scoped-name node.
    #[must_use]
    pub fn name(key: ScopedNameKey) -> Self {
        Self::Name(key)
    }

    /// Construct a tree-node node.
    #[must_use]
    pub const fn tree_node(node: TreeNodeId) -> Self {
        Self::TreeNode(node)
    }
}

/// Workbook-level dirty seeds — the single workbook-level seed vocabulary
/// (D3 §2.1, X1).
///
/// Sheet-local seeds are *addressed* (a `Sheet { sheet, seed }` reuses the
/// per-sheet [`GridDirtySeed`](super::invalidation) vocabulary), never global.
/// D1 C4 `WorkbookSettingChanged` seeds enter as [`WorkbookDirtySeed::Setting`].
/// Document-surface verbs emit seeds; they never touch dirty state directly.
///
/// Cannot derive `Eq`/`Ord` because [`WorkbookSettingChanged`] carries setting
/// payloads that are only `PartialEq` — the closure keys its dirty *set* on
/// [`WorkbookCalcNodeId`] (which is fully ordered), not on the seed vocabulary.
#[derive(Debug, Clone, PartialEq)]
pub enum WorkbookDirtySeed {
    /// A sheet-local seed addressed to a specific sheet; reuses the per-sheet
    /// dirty-seed vocabulary unchanged.
    Sheet {
        sheet: TreeNodeId,
        seed: super::invalidation::GridDirtySeed,
    },
    /// A scoped name became dirty.
    Name(ScopedNameKey),
    /// A tree node became dirty.
    TreeNode(TreeNodeId),
    /// One workbook-wide volatile tick (D3 §7).
    Volatile,
    /// A workbook-wide external-input tick.
    External,
    /// A typed D1 C4 workbook-setting change.
    Setting(WorkbookSettingChanged),
}

/// The authoritative owning-sheet identity a per-sheet dependency index is
/// stamped with, so the routing invariant can be enforced at registration.
///
/// A per-sheet [`GridInvalidationRef`](super::invalidation) / index carries the
/// `(workbook_id, sheet_id)` of the sheet it belongs to. When set, every
/// dependent and every cell/range dependency address routed into that index
/// must resolve to this sheet; a foreign address is a routing bug and is
/// rejected with [`GridRefError::ForeignSheetDependency`] rather than silently
/// mis-filed (D3 §1 routing invariant).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OwningSheetIdentity {
    pub workbook_id: String,
    pub sheet_id: String,
}

impl OwningSheetIdentity {
    #[must_use]
    pub fn new(workbook_id: impl Into<String>, sheet_id: impl Into<String>) -> Self {
        Self {
            workbook_id: workbook_id.into(),
            sheet_id: sheet_id.into(),
        }
    }

    /// Does `address` belong to the sheet this identity names?
    #[must_use]
    pub fn owns_address(&self, address: &ExcelGridCellAddress) -> bool {
        address.workbook_id == self.workbook_id && address.sheet_id == self.sheet_id
    }

    /// Does `rect` belong to the sheet this identity names?
    #[must_use]
    pub fn owns_rect(&self, rect: &GridRect) -> bool {
        rect.workbook_id == self.workbook_id && rect.sheet_id == self.sheet_id
    }
}

// ---------------------------------------------------------------------------
// Workbook volatile tick (W062 D3 §7, bead `calc-5kqg.36` / R4.8).
// ---------------------------------------------------------------------------

/// The single volatile observation minted once per workbook recalc
/// **transaction** and observed coherently by every evaluation in it — every
/// sheet, every scoped name, and every tree node (D3 §7).
///
/// - `NOW()`/`TODAY()` read [`timestamp_serial`](Self::timestamp_serial): one
///   Excel date serial for the whole transaction, so `NOW()` on Sheet1 equals
///   `NOW()` on Sheet3 equals `NOW()` in a tree node.
/// - `RAND()`/`RANDBETWEEN()`/`RANDARRAY()` draw from a stream derived from
///   [`rng_seed`](Self::rng_seed) **plus the evaluating node id**
///   ([`random_provider_for_node`](Self::random_provider_for_node)), so
///   evaluation *order* does not change values (two cells' draws are
///   independent of which evaluates first — a concurrency-prep property).
/// - `tick_id`/`timestamp_serial`/`rng_seed` are recorded in the recalc report
///   for replay: replaying the recorded tick reproduces the transaction's
///   volatiles exactly.
///
/// Because the oracle and the optimized lane are both handed the *same* tick
/// for a transaction, the differential compares volatiles **exactly** rather
/// than excluding them — that is what keeps the harness honest (D3 §7).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorkbookRecalcTick {
    /// Monotone identity of this transaction's tick (diagnostic / replay key).
    pub tick_id: u64,
    /// The Excel date serial every `NOW()`/`TODAY()` in this transaction reads.
    pub timestamp_serial: f64,
    /// The seed every node-keyed `RAND*` stream in this transaction derives from.
    pub rng_seed: u64,
}

impl WorkbookRecalcTick {
    /// Construct a tick with explicit fields. Tests inject a fixed tick this
    /// way (no wall-clock assertions); the recalc drivers mint one with
    /// [`mint`](Self::mint) at the transaction boundary.
    #[must_use]
    pub const fn new(tick_id: u64, timestamp_serial: f64, rng_seed: u64) -> Self {
        Self {
            tick_id,
            timestamp_serial,
            rng_seed,
        }
    }

    /// Mint a fresh tick at a workbook recalc-transaction boundary, reading the
    /// wall clock once for the whole transaction. `tick_id` and `rng_seed` are
    /// derived from the same clock reading (nanoseconds since the Unix epoch);
    /// `timestamp_serial` is the corresponding Excel 1900-system date serial.
    ///
    /// Clock source is target-gated because the two supported targets expose
    /// different epoch clocks, but both feed the identical serial arithmetic:
    /// - **native**: `std::time::SystemTime::now()` gives nanoseconds since the
    ///   Unix epoch directly.
    /// - **wasm32**: `SystemTime` has no backend on `wasm32-unknown-unknown`
    ///   (it panics with "time not implemented on this platform"), so we read
    ///   `js_sys::Date::now()` — milliseconds since the Unix epoch as an `f64` —
    ///   and scale it to nanoseconds. Both branches produce a `u128` nanosecond
    ///   count that flows through the same `nanos_u64`/`unix_seconds`/serial math.
    #[must_use]
    pub fn mint() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let nanos: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        #[cfg(target_arch = "wasm32")]
        let nanos: u128 = {
            // `Date::now()` is milliseconds since the Unix epoch (non-negative);
            // scale to nanoseconds to match the native path's units.
            let millis = js_sys::Date::now();
            if millis.is_finite() && millis >= 0.0 {
                (millis * 1_000_000.0) as u128
            } else {
                0
            }
        };
        let nanos_u64 = (nanos & u128::from(u64::MAX)) as u64;
        let unix_seconds = (nanos / 1_000_000_000) as f64;
        // Excel 1900 date system: serial 25569.0 is 1970-01-01 (the Unix epoch),
        // one day is 86_400 seconds.
        let timestamp_serial = 25_569.0 + unix_seconds / 86_400.0;
        Self {
            tick_id: nanos_u64,
            timestamp_serial,
            rng_seed: nanos_u64,
        }
    }

    /// A node-keyed random provider for the node identified by `node_key`.
    ///
    /// Stream construction (documented contract): the draw sequence for a node
    /// is `splitmix64` iterated over a per-node base state
    /// `mix(rng_seed) ^ mix(hash64(node_key))`, where `hash64` is FNV-1a over
    /// the node-key bytes. The first `RAND*` call in a node returns the first
    /// output, the second call the second output, and so on (a per-node call
    /// counter). Two properties follow directly:
    ///
    /// - **Order independence:** a node's stream depends only on `rng_seed` and
    ///   its own `node_key`, never on which node (or which worklist position)
    ///   evaluated first — permuting the evaluation order leaves every node's
    ///   `RAND*` values unchanged.
    /// - **Determinism + per-node distinctness:** a fixed tick reproduces every
    ///   node's stream exactly, and distinct node keys almost-surely produce
    ///   distinct streams (distinct base states).
    ///
    /// `node_key` must be a stable identity of the evaluating node — for a grid
    /// cell, `"{workbook}:{sheet}:R{row}C{col}"`; for a tree node / scoped name,
    /// its own stable key. Callers construct one provider per node evaluation.
    #[must_use]
    pub fn random_provider_for_node(&self, node_key: &str) -> WorkbookVolatileRandomProvider {
        WorkbookVolatileRandomProvider::new(self.rng_seed, node_key)
    }
}

/// Node-keyed pseudo-random stream for one node's `RAND*` draws within a
/// [`WorkbookRecalcTick`] (D3 §7). See
/// [`WorkbookRecalcTick::random_provider_for_node`] for the stream contract.
///
/// W062 R4.14 (D3 §10) interior-mutability disposition: the `state: Cell<u64>`
/// call counter is retained (not reworked) and is Send-safe + evaluation-local
/// by construction. The `RandomProvider` trait (OxFunc) takes `&self` on
/// `random_unit`, so the draw counter must live behind interior mutability; a
/// value-threaded form is impossible without an upstream trait break (out of
/// scope). It does not violate the concurrency constraint because: (1) callers
/// construct exactly one provider per node evaluation
/// (`WorkbookRecalcTick::random_provider_for_node`), so the `Cell` is never
/// shared across nodes or threads; (2) `Cell<u64>: Send`, so a future
/// staged-concurrency executor (W053) can move a node's provider onto its
/// evaluation thread; (3) the stream base state is `mix(rng_seed) ^
/// mix(hash64(node_key))` — a node's draws depend only on the tick seed and its
/// own key, never on evaluation order or worklist position, so order-independent
/// volatiles (D3 §10 constraint 5) hold regardless of the mutable counter.
#[derive(Debug)]
pub struct WorkbookVolatileRandomProvider {
    /// Per-node base state: `mix(rng_seed) ^ mix(hash64(node_key))`. Advanced by
    /// `splitmix64` on each draw.
    state: Cell<u64>,
}

impl WorkbookVolatileRandomProvider {
    #[must_use]
    fn new(rng_seed: u64, node_key: &str) -> Self {
        let base = splitmix64(rng_seed) ^ splitmix64(fnv1a64(node_key.as_bytes()));
        Self {
            state: Cell::new(base),
        }
    }
}

impl RandomProvider for WorkbookVolatileRandomProvider {
    fn random_unit(&self) -> f64 {
        // Advance the per-node stream one step and map the mixed output to
        // `[0, 1)`. `&self` is required by the trait, so the call counter lives
        // in a `Cell` (single-threaded evaluation — no `Sync` needed).
        let next = self.state.get().wrapping_add(0x9E37_79B9_7F4A_7C15);
        self.state.set(next);
        let mixed = splitmix64(next);
        // Take the top 53 bits for a uniform double in [0, 1), matching the
        // usual `u64 -> f64` unit-interval construction.
        ((mixed >> 11) as f64) * (1.0 / ((1_u64 << 53) as f64))
    }
}

/// One `splitmix64` finalizing mix — a well-distributed bijection on `u64`.
#[must_use]
const fn splitmix64(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// FNV-1a 64-bit hash of `bytes` — folds a node key into the stream base state.
#[must_use]
const fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xCBF2_9CE4_8422_2325;
    let mut i = 0;
    while i < bytes.len() {
        hash ^= bytes[i] as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01B3);
        i += 1;
    }
    hash
}

#[cfg(test)]
mod recalc_tick_tests {
    use super::*;

    fn draws(tick: WorkbookRecalcTick, node_key: &str, count: usize) -> Vec<f64> {
        let provider = tick.random_provider_for_node(node_key);
        (0..count).map(|_| provider.random_unit()).collect()
    }

    #[test]
    fn now_serial_is_coherent_for_every_node_in_one_tick() {
        // D3 §7: `NOW()` reads the tick's single serial, so every node in a
        // transaction observes the same timestamp regardless of node key.
        let tick = WorkbookRecalcTick::new(7, 45_678.25, 0xABCD);
        assert_eq!(tick.timestamp_serial, 45_678.25);
        // The serial is a property of the tick, not the node — identical for
        // "Sheet1", "Sheet3", and a tree node.
        for _node in ["wb:Sheet1:R1C1", "wb:Sheet3:R9C9", "treenode:node:5"] {
            assert_eq!(tick.timestamp_serial, 45_678.25);
        }
    }

    #[test]
    fn rand_draws_are_in_unit_interval() {
        let tick = WorkbookRecalcTick::new(1, 0.0, 0x1234_5678);
        for value in draws(tick, "wb:S1:R1C1", 64) {
            assert!(
                (0.0..1.0).contains(&value),
                "RAND draw {value} outside [0,1)"
            );
        }
    }

    #[test]
    fn rand_is_deterministic_given_a_fixed_tick() {
        // Same tick + same node key ⇒ identical stream (replay reproduces it).
        let tick = WorkbookRecalcTick::new(42, 12_345.5, 0xDEAD_BEEF);
        assert_eq!(
            draws(tick, "wb:Sheet1:R2C3", 16),
            draws(tick, "wb:Sheet1:R2C3", 16)
        );
    }

    #[test]
    fn rand_streams_differ_per_node() {
        // Distinct node keys draw distinct streams under the same tick.
        let tick = WorkbookRecalcTick::new(3, 0.0, 0x99);
        let a = draws(tick, "wb:Sheet1:R1C1", 8);
        let b = draws(tick, "wb:Sheet1:R1C2", 8);
        let c = draws(tick, "treenode:node:5", 8);
        assert_ne!(a, b);
        assert_ne!(a, c);
        assert_ne!(b, c);
    }

    #[test]
    fn rand_is_order_independent() {
        // D3 §7 concurrency-prep property: a node's draws depend only on
        // (rng_seed, node_key), never on the order nodes evaluate in. Drawing
        // node A fully then node B must give the same per-node values as
        // interleaving A and B, or drawing B before A.
        let tick = WorkbookRecalcTick::new(5, 0.0, 0x5EED);
        let a_first = draws(tick, "nodeA", 8);
        let b_first = draws(tick, "nodeB", 8);

        // Interleave the two nodes' draws.
        let pa = tick.random_provider_for_node("nodeA");
        let pb = tick.random_provider_for_node("nodeB");
        let mut a_interleaved = Vec::new();
        let mut b_interleaved = Vec::new();
        for _ in 0..8 {
            b_interleaved.push(pb.random_unit());
            a_interleaved.push(pa.random_unit());
        }
        assert_eq!(a_first, a_interleaved, "node A stream changed with order");
        assert_eq!(b_first, b_interleaved, "node B stream changed with order");
    }

    #[test]
    fn rng_seed_changes_the_streams() {
        // Two recalcs (two ticks with different seeds) differ — F9 re-ticks.
        let node = "wb:Sheet1:R1C1";
        let first = draws(WorkbookRecalcTick::new(1, 0.0, 111), node, 8);
        let second = draws(WorkbookRecalcTick::new(2, 0.0, 222), node, 8);
        assert_ne!(first, second);
    }
}
