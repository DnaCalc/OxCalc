#![forbid(unsafe_code)]

//! Per-edge value cache for differential evaluation.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CallSiteId(pub String);

impl CallSiteId {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HoleBindingFingerprint(pub String);

impl HoleBindingFingerprint {
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct EdgeValueCacheKey {
    pub call_site_id: CallSiteId,
    pub hole_binding_fingerprint: HoleBindingFingerprint,
}

impl EdgeValueCacheKey {
    #[must_use]
    pub fn new(
        call_site_id: impl Into<String>,
        hole_binding_fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            call_site_id: CallSiteId::new(call_site_id),
            hole_binding_fingerprint: HoleBindingFingerprint::new(hole_binding_fingerprint),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeValueCacheRetentionClass {
    W054PendingEphemeralPerEdgeValueCache,
}

impl EdgeValueCacheRetentionClass {
    #[must_use]
    pub fn selector_key(self) -> &'static str {
        match self {
            Self::W054PendingEphemeralPerEdgeValueCache => "W054PendingEphemeralPerEdgeValueCache",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeValueCacheExclusionReason {
    VolatileFunction,
    EffectfulPath,
}

impl EdgeValueCacheExclusionReason {
    #[must_use]
    pub fn selector_key(self) -> &'static str {
        match self {
            Self::VolatileFunction => "VolatileFunction",
            Self::EffectfulPath => "EffectfulPath",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EdgeValueCacheEligibility {
    Cacheable,
    Excluded(EdgeValueCacheExclusionReason),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeValueCachePathFacts {
    pub volatile: bool,
    pub effectful: bool,
}

impl EdgeValueCachePathFacts {
    #[must_use]
    pub fn eligibility(self) -> EdgeValueCacheEligibility {
        if self.volatile {
            return EdgeValueCacheEligibility::Excluded(
                EdgeValueCacheExclusionReason::VolatileFunction,
            );
        }
        if self.effectful {
            return EdgeValueCacheEligibility::Excluded(
                EdgeValueCacheExclusionReason::EffectfulPath,
            );
        }
        EdgeValueCacheEligibility::Cacheable
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeValueCacheEntry {
    pub key: EdgeValueCacheKey,
    pub value_payload: String,
    pub derivation_epoch: u64,
    pub retention_class: EdgeValueCacheRetentionClass,
    pub insertion_sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EdgeValueCachePolicy {
    pub max_entries: usize,
    pub retention_class: EdgeValueCacheRetentionClass,
}

impl EdgeValueCachePolicy {
    #[must_use]
    pub fn w054_pending(max_entries: usize) -> Self {
        assert!(max_entries > 0, "edge value cache must be bounded");
        Self {
            max_entries,
            retention_class: EdgeValueCacheRetentionClass::W054PendingEphemeralPerEdgeValueCache,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeValueCacheLookup {
    Hit(EdgeValueCacheEntry),
    Miss,
    Excluded(EdgeValueCacheExclusionReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EdgeValueCacheStoreResult {
    Stored {
        entry: EdgeValueCacheEntry,
        evicted_key: Option<EdgeValueCacheKey>,
    },
    Excluded(EdgeValueCacheExclusionReason),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeValueCache {
    policy: EdgeValueCachePolicy,
    entries: BTreeMap<EdgeValueCacheKey, EdgeValueCacheEntry>,
    next_insertion_sequence: u64,
}

impl EdgeValueCache {
    #[must_use]
    pub fn new(policy: EdgeValueCachePolicy) -> Self {
        Self {
            policy,
            entries: BTreeMap::new(),
            next_insertion_sequence: 0,
        }
    }

    #[must_use]
    pub fn policy(&self) -> &EdgeValueCachePolicy {
        &self.policy
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    #[must_use]
    pub fn lookup(
        &self,
        key: &EdgeValueCacheKey,
        eligibility: EdgeValueCacheEligibility,
    ) -> EdgeValueCacheLookup {
        match eligibility {
            EdgeValueCacheEligibility::Cacheable => self
                .entries
                .get(key)
                .cloned()
                .map_or(EdgeValueCacheLookup::Miss, EdgeValueCacheLookup::Hit),
            EdgeValueCacheEligibility::Excluded(reason) => EdgeValueCacheLookup::Excluded(reason),
        }
    }

    pub fn store(
        &mut self,
        key: EdgeValueCacheKey,
        eligibility: EdgeValueCacheEligibility,
        value_payload: impl Into<String>,
        derivation_epoch: u64,
    ) -> EdgeValueCacheStoreResult {
        if let EdgeValueCacheEligibility::Excluded(reason) = eligibility {
            return EdgeValueCacheStoreResult::Excluded(reason);
        }

        let evicted_key =
            if !self.entries.contains_key(&key) && self.entries.len() >= self.policy.max_entries {
                self.evict_oldest_entry()
            } else {
                None
            };

        let entry = EdgeValueCacheEntry {
            key: key.clone(),
            value_payload: value_payload.into(),
            derivation_epoch,
            retention_class: self.policy.retention_class,
            insertion_sequence: self.next_insertion_sequence,
        };
        self.next_insertion_sequence = self.next_insertion_sequence.saturating_add(1);
        self.entries.insert(key, entry.clone());

        EdgeValueCacheStoreResult::Stored { entry, evicted_key }
    }

    fn evict_oldest_entry(&mut self) -> Option<EdgeValueCacheKey> {
        let oldest_key = self
            .entries
            .iter()
            .map(|(key, entry)| (entry.insertion_sequence, key.clone()))
            .min()
            .map(|(_, key)| key)?;
        self.entries.remove(&oldest_key);
        Some(oldest_key)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;

    fn f1_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-f1-per-edge-value-cache-001")
    }

    fn lookup_status(lookup: &EdgeValueCacheLookup) -> &'static str {
        match lookup {
            EdgeValueCacheLookup::Hit(_) => "hit",
            EdgeValueCacheLookup::Miss => "miss",
            EdgeValueCacheLookup::Excluded(_) => "excluded",
        }
    }

    fn cache_validation_artifact_json() -> serde_json::Value {
        let policy = EdgeValueCachePolicy::w054_pending(2);
        let mut cache = EdgeValueCache::new(policy.clone());
        let key = EdgeValueCacheKey::new("call:sum:row1", "holefp:a1");
        let changed_binding = EdgeValueCacheKey::new("call:sum:row1", "holefp:a2");

        cache.store(
            key.clone(),
            EdgeValueCacheEligibility::Cacheable,
            "value:row1:3",
            1,
        );
        let hit = cache.lookup(&key, EdgeValueCacheEligibility::Cacheable);
        let miss = cache.lookup(&changed_binding, EdgeValueCacheEligibility::Cacheable);
        let volatile = EdgeValueCachePathFacts {
            volatile: true,
            effectful: true,
        }
        .eligibility();
        let effectful = EdgeValueCachePathFacts {
            volatile: false,
            effectful: true,
        }
        .eligibility();

        json!({
            "run_id": "w050-f1-per-edge-value-cache-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core per_edge_value_cache -- --nocapture",
            "cache_key_fields": [
                "call_site_id",
                "hole_binding_fingerprint"
            ],
            "retention_class": policy.retention_class.selector_key(),
            "eviction_policy": {
                "policy": "MaxEntriesOldestFirst",
                "max_entries": policy.max_entries
            },
            "validation_cases": [
                {
                    "case_id": "hit_same_call_site_and_hole_binding_fingerprint",
                    "lookup": lookup_status(&hit),
                    "value_payload": match hit {
                        EdgeValueCacheLookup::Hit(entry) => entry.value_payload,
                        _ => String::new(),
                    }
                },
                {
                    "case_id": "miss_changed_hole_binding_fingerprint",
                    "lookup": lookup_status(&miss)
                },
                {
                    "case_id": "excluded_volatile_path",
                    "eligibility": match volatile {
                        EdgeValueCacheEligibility::Excluded(reason) => reason.selector_key(),
                        EdgeValueCacheEligibility::Cacheable => "Cacheable",
                    }
                },
                {
                    "case_id": "excluded_effectful_path",
                    "eligibility": match effectful {
                        EdgeValueCacheEligibility::Excluded(reason) => reason.selector_key(),
                        EdgeValueCacheEligibility::Cacheable => "Cacheable",
                    }
                }
            ]
        })
    }

    #[test]
    fn per_edge_value_cache_hits_same_call_site_and_hole_binding_fingerprint() {
        let mut cache = EdgeValueCache::new(EdgeValueCachePolicy::w054_pending(4));
        let key = EdgeValueCacheKey::new("call:sum:row1", "holefp:a1");

        let stored = cache.store(
            key.clone(),
            EdgeValueCacheEligibility::Cacheable,
            "value:row1:3",
            1,
        );
        assert!(matches!(stored, EdgeValueCacheStoreResult::Stored { .. }));

        let lookup = cache.lookup(&key, EdgeValueCacheEligibility::Cacheable);
        assert!(matches!(lookup, EdgeValueCacheLookup::Hit(_)));
        let EdgeValueCacheLookup::Hit(entry) = lookup else {
            panic!("lookup should hit");
        };
        assert_eq!(entry.value_payload, "value:row1:3");
        assert_eq!(entry.retention_class, cache.policy().retention_class);
    }

    #[test]
    fn per_edge_value_cache_misses_when_hole_binding_fingerprint_changes() {
        let mut cache = EdgeValueCache::new(EdgeValueCachePolicy::w054_pending(4));
        cache.store(
            EdgeValueCacheKey::new("call:sum:row1", "holefp:a1"),
            EdgeValueCacheEligibility::Cacheable,
            "value:row1:3",
            1,
        );

        let changed_binding =
            EdgeValueCacheKey::new("call:sum:row1", "holefp:a1-after-input-change");
        assert_eq!(
            cache.lookup(&changed_binding, EdgeValueCacheEligibility::Cacheable),
            EdgeValueCacheLookup::Miss
        );
    }

    #[test]
    fn per_edge_value_cache_excludes_volatile_and_effectful_paths() {
        let mut cache = EdgeValueCache::new(EdgeValueCachePolicy::w054_pending(4));
        let key = EdgeValueCacheKey::new("call:rand:row1", "holefp:volatile");
        let volatile = EdgeValueCachePathFacts {
            volatile: true,
            effectful: true,
        }
        .eligibility();
        let effectful = EdgeValueCachePathFacts {
            volatile: false,
            effectful: true,
        }
        .eligibility();

        assert_eq!(
            cache.store(key.clone(), volatile, "value:volatile", 1),
            EdgeValueCacheStoreResult::Excluded(EdgeValueCacheExclusionReason::VolatileFunction)
        );
        assert!(cache.is_empty());
        assert_eq!(
            cache.lookup(&key, effectful),
            EdgeValueCacheLookup::Excluded(EdgeValueCacheExclusionReason::EffectfulPath)
        );
    }

    #[test]
    fn per_edge_value_cache_eviction_is_bounded_oldest_first() {
        let mut cache = EdgeValueCache::new(EdgeValueCachePolicy::w054_pending(2));
        let first = EdgeValueCacheKey::new("call:first", "holefp:1");
        let second = EdgeValueCacheKey::new("call:second", "holefp:2");
        let third = EdgeValueCacheKey::new("call:third", "holefp:3");

        cache.store(
            first.clone(),
            EdgeValueCacheEligibility::Cacheable,
            "value:first",
            1,
        );
        cache.store(
            second.clone(),
            EdgeValueCacheEligibility::Cacheable,
            "value:second",
            1,
        );
        let stored = cache.store(
            third.clone(),
            EdgeValueCacheEligibility::Cacheable,
            "value:third",
            2,
        );

        assert_eq!(cache.len(), 2);
        assert_eq!(
            stored,
            EdgeValueCacheStoreResult::Stored {
                entry: EdgeValueCacheEntry {
                    key: third.clone(),
                    value_payload: "value:third".to_string(),
                    derivation_epoch: 2,
                    retention_class:
                        EdgeValueCacheRetentionClass::W054PendingEphemeralPerEdgeValueCache,
                    insertion_sequence: 2,
                },
                evicted_key: Some(first.clone()),
            }
        );
        assert_eq!(
            cache.lookup(&first, EdgeValueCacheEligibility::Cacheable),
            EdgeValueCacheLookup::Miss
        );
        assert!(matches!(
            cache.lookup(&second, EdgeValueCacheEligibility::Cacheable),
            EdgeValueCacheLookup::Hit(_)
        ));
        assert!(matches!(
            cache.lookup(&third, EdgeValueCacheEligibility::Cacheable),
            EdgeValueCacheLookup::Hit(_)
        ));
    }

    #[test]
    fn checked_in_per_edge_value_cache_artifact_matches_runtime_validation() {
        let artifact_path = f1_artifact_root().join("run_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("F1 run artifact should be checked in"),
        )
        .expect("F1 run artifact should be valid JSON");

        assert_eq!(artifact, cache_validation_artifact_json());
    }
}
