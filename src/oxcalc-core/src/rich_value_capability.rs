#![forbid(unsafe_code)]

//! Rich-value capability vocabulary and replay identity keys.

use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichValueRank {
    AnyRank,
    Exact(u8),
}

impl RichValueRank {
    #[must_use]
    pub fn stable_key(self) -> String {
        match self {
            Self::AnyRank => "AnyRank".to_string(),
            Self::Exact(rank) => rank.to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichValueIndexType {
    GridCoordinate,
    Ordinal,
}

impl RichValueIndexType {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::GridCoordinate => "GridCoordinate",
            Self::Ordinal => "Ordinal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichValueElementValueClass {
    AnyValue,
}

impl RichValueElementValueClass {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::AnyValue => "AnyValue",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichValueOrderGuarantee {
    DeterministicStable,
    RowMajorStable,
}

impl RichValueOrderGuarantee {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::DeterministicStable => "DeterministicStable",
            Self::RowMajorStable => "RowMajorStable",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichValueExtentClass {
    AnyExtent,
    RectangularGrid,
}

impl RichValueExtentClass {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::AnyExtent => "AnyExtent",
            Self::RectangularGrid => "RectangularGrid",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichValueMaterialisationTargetClass {
    EvalValueOrArray,
}

impl RichValueMaterialisationTargetClass {
    #[must_use]
    pub fn stable_key(self) -> &'static str {
        match self {
            Self::EvalValueOrArray => "EvalValueOrArray",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RichValueCapability {
    Indexable {
        rank: RichValueRank,
        index_type: RichValueIndexType,
        element_value_class: RichValueElementValueClass,
    },
    Enumerable {
        element_value_class: RichValueElementValueClass,
        order_guarantee: RichValueOrderGuarantee,
    },
    Shaped {
        extent_class: RichValueExtentClass,
    },
    Materialisable {
        target_class: RichValueMaterialisationTargetClass,
    },
}

impl RichValueCapability {
    #[must_use]
    pub fn selector_key(&self) -> &'static str {
        match self {
            Self::Indexable { .. } => "Indexable",
            Self::Enumerable { .. } => "Enumerable",
            Self::Shaped { .. } => "Shaped",
            Self::Materialisable { .. } => "Materialisable",
        }
    }

    #[must_use]
    pub fn parameter_names(&self) -> &'static [&'static str] {
        match self {
            Self::Indexable { .. } => &["rank", "index_type", "element_value_class"],
            Self::Enumerable { .. } => &["element_value_class", "order_guarantee"],
            Self::Shaped { .. } => &["extent_class"],
            Self::Materialisable { .. } => &["target_class"],
        }
    }

    #[must_use]
    pub fn stable_key(&self) -> String {
        match self {
            Self::Indexable {
                rank,
                index_type,
                element_value_class,
            } => format!(
                "Indexable(rank={},index_type={},element_value_class={})",
                rank.stable_key(),
                index_type.stable_key(),
                element_value_class.stable_key()
            ),
            Self::Enumerable {
                element_value_class,
                order_guarantee,
            } => format!(
                "Enumerable(element_value_class={},order_guarantee={})",
                element_value_class.stable_key(),
                order_guarantee.stable_key()
            ),
            Self::Shaped { extent_class } => {
                format!("Shaped(extent_class={})", extent_class.stable_key())
            }
            Self::Materialisable { target_class } => {
                format!("Materialisable(target_class={})", target_class.stable_key())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RichValueCapabilitySet {
    capabilities: Vec<RichValueCapability>,
}

impl RichValueCapabilitySet {
    #[must_use]
    pub fn new(capabilities: impl IntoIterator<Item = RichValueCapability>) -> Self {
        let mut capabilities = capabilities.into_iter().collect::<Vec<_>>();
        capabilities.sort_by_key(RichValueCapability::stable_key);
        capabilities.dedup_by_key(|capability| capability.stable_key());
        Self { capabilities }
    }

    #[must_use]
    pub fn capabilities(&self) -> &[RichValueCapability] {
        &self.capabilities
    }

    #[must_use]
    pub fn stable_keys(&self) -> Vec<String> {
        self.capabilities
            .iter()
            .map(RichValueCapability::stable_key)
            .collect()
    }

    #[must_use]
    pub fn stable_key(&self) -> String {
        self.stable_keys().join("+")
    }

    #[must_use]
    pub fn is_superset_of(&self, required: &Self) -> bool {
        let available = self.stable_keys().into_iter().collect::<BTreeSet<_>>();
        required
            .stable_keys()
            .into_iter()
            .all(|required_key| available.contains(&required_key))
    }
}

#[must_use]
pub fn w050_initial_capability_examples() -> Vec<RichValueCapability> {
    vec![
        RichValueCapability::Indexable {
            rank: RichValueRank::Exact(2),
            index_type: RichValueIndexType::GridCoordinate,
            element_value_class: RichValueElementValueClass::AnyValue,
        },
        RichValueCapability::Enumerable {
            element_value_class: RichValueElementValueClass::AnyValue,
            order_guarantee: RichValueOrderGuarantee::DeterministicStable,
        },
        RichValueCapability::Shaped {
            extent_class: RichValueExtentClass::RectangularGrid,
        },
        RichValueCapability::Materialisable {
            target_class: RichValueMaterialisationTargetClass::EvalValueOrArray,
        },
    ]
}

#[must_use]
pub fn w050_initial_required_capability_set_example() -> RichValueCapabilitySet {
    RichValueCapabilitySet::new(w050_initial_capability_examples())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use serde_json::json;

    use super::*;

    fn g1_artifact_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/test-runs/core-engine/w050-g1-rich-capability-vocabulary-001")
    }

    fn vocabulary_artifact_json() -> serde_json::Value {
        let examples = w050_initial_capability_examples();
        let required_set = w050_initial_required_capability_set_example();
        let extended_set = RichValueCapabilitySet::new(examples.iter().cloned().chain([
            RichValueCapability::Indexable {
                rank: RichValueRank::AnyRank,
                index_type: RichValueIndexType::Ordinal,
                element_value_class: RichValueElementValueClass::AnyValue,
            },
        ]));
        let missing_set = RichValueCapabilitySet::new(examples.iter().take(3).cloned());

        json!({
            "run_id": "w050-g1-rich-capability-vocabulary-001",
            "validation_status": "pass",
            "primary_validation_command": "cargo test -p oxcalc-core rich_value_capability -- --nocapture",
            "vocabulary": examples.iter().map(|capability| {
                json!({
                    "selector_key": capability.selector_key(),
                    "parameter_names": capability.parameter_names(),
                    "stable_key_example": capability.stable_key()
                })
            }).collect::<Vec<_>>(),
            "replay_identity": {
                "required_capability_set_key": required_set.stable_key(),
                "ordering_rule": "Capability-set replay identity is sorted by typed stable key and deduplicated.",
                "additive_extension_rule": "New capability kinds or parameter values are additive only through new stable keys; older required-set keys remain byte-stable and do not silently alias new capabilities.",
                "producer_superset_rule": "A producer capability set may satisfy a required set by stable-key superset, but the required set remains the template/replay identity."
            },
            "checks": {
                "order_insensitive_required_set": required_set.stable_key() == RichValueCapabilitySet::new(examples.iter().rev().cloned()).stable_key(),
                "extended_producer_is_superset": extended_set.is_superset_of(&required_set),
                "missing_materialisable_is_not_superset": !missing_set.is_superset_of(&required_set)
            }
        })
    }

    #[test]
    fn rich_value_capability_stable_keys_are_typed() {
        let examples = w050_initial_capability_examples();

        assert_eq!(
            examples
                .iter()
                .map(RichValueCapability::selector_key)
                .collect::<Vec<_>>(),
            vec!["Indexable", "Enumerable", "Shaped", "Materialisable"]
        );
        assert_eq!(
            examples
                .iter()
                .map(RichValueCapability::stable_key)
                .collect::<Vec<_>>(),
            vec![
                "Indexable(rank=2,index_type=GridCoordinate,element_value_class=AnyValue)",
                "Enumerable(element_value_class=AnyValue,order_guarantee=DeterministicStable)",
                "Shaped(extent_class=RectangularGrid)",
                "Materialisable(target_class=EvalValueOrArray)",
            ]
        );
    }

    #[test]
    fn rich_value_capability_sets_are_order_insensitive_and_additive() {
        let examples = w050_initial_capability_examples();
        let required = RichValueCapabilitySet::new(examples.clone());
        let reversed = RichValueCapabilitySet::new(examples.iter().rev().cloned());
        let extended = RichValueCapabilitySet::new(examples.iter().cloned().chain([
            RichValueCapability::Indexable {
                rank: RichValueRank::AnyRank,
                index_type: RichValueIndexType::Ordinal,
                element_value_class: RichValueElementValueClass::AnyValue,
            },
        ]));
        let missing_materialisable = RichValueCapabilitySet::new(examples.into_iter().take(3));

        assert_eq!(required.stable_key(), reversed.stable_key());
        assert!(extended.is_superset_of(&required));
        assert!(!missing_materialisable.is_superset_of(&required));
        assert_ne!(extended.stable_key(), required.stable_key());
    }

    #[test]
    fn checked_in_rich_value_capability_artifact_matches_runtime_vocabulary() {
        let artifact_path = g1_artifact_root().join("run_artifact.json");
        let artifact = serde_json::from_str::<serde_json::Value>(
            &fs::read_to_string(artifact_path).expect("G1 run artifact should be checked in"),
        )
        .expect("G1 run artifact should be valid JSON");

        assert_eq!(artifact, vocabulary_artifact_json());
    }
}
