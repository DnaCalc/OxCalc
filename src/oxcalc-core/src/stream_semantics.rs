#![forbid(unsafe_code)]

//! Profile-governed stream semantics selector for external invalidation.

use serde::{Deserialize, Serialize};

use crate::repository::{CalculationRepository, TopicEnvelope, TopicEnvelopeUpdate};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StreamSemanticsVersion {
    ExternalInvalidationV0,
    TopicEnvelopeV1,
    RtdLifecycleV2,
}

impl StreamSemanticsVersion {
    #[must_use]
    pub fn selector_key(self) -> &'static str {
        match self {
            Self::ExternalInvalidationV0 => "ExternalInvalidationV0",
            Self::TopicEnvelopeV1 => "TopicEnvelopeV1",
            Self::RtdLifecycleV2 => "RtdLifecycleV2",
        }
    }

    #[must_use]
    pub fn behavior(self) -> StreamSemanticsBehavior {
        match self {
            Self::ExternalInvalidationV0 => StreamSemanticsBehavior {
                replay_records_topic_envelope: false,
                deterministic_update_ordering: false,
                dedupe_by_event_identity: false,
                hooks: vec![StreamSemanticsBehaviorHook::ExternalInvalidationDirtySeed],
            },
            Self::TopicEnvelopeV1 => StreamSemanticsBehavior {
                replay_records_topic_envelope: true,
                deterministic_update_ordering: true,
                dedupe_by_event_identity: true,
                hooks: vec![
                    StreamSemanticsBehaviorHook::ExternalInvalidationDirtySeed,
                    StreamSemanticsBehaviorHook::TopicEnvelopeReplayInput,
                    StreamSemanticsBehaviorHook::TopicEnvelopeOrderingDedupe,
                ],
            },
            Self::RtdLifecycleV2 => StreamSemanticsBehavior {
                replay_records_topic_envelope: true,
                deterministic_update_ordering: true,
                dedupe_by_event_identity: true,
                hooks: vec![
                    StreamSemanticsBehaviorHook::ExternalInvalidationDirtySeed,
                    StreamSemanticsBehaviorHook::TopicEnvelopeReplayInput,
                    StreamSemanticsBehaviorHook::TopicEnvelopeOrderingDedupe,
                    StreamSemanticsBehaviorHook::RtdLifecycleTracking,
                ],
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum StreamSemanticsBehaviorHook {
    ExternalInvalidationDirtySeed,
    TopicEnvelopeReplayInput,
    TopicEnvelopeOrderingDedupe,
    RtdLifecycleTracking,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamSemanticsBehavior {
    pub replay_records_topic_envelope: bool,
    pub deterministic_update_ordering: bool,
    pub dedupe_by_event_identity: bool,
    pub hooks: Vec<StreamSemanticsBehaviorHook>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamSemanticsProfile {
    pub profile_version: String,
    pub stream_semantics_version: StreamSemanticsVersion,
}

impl StreamSemanticsProfile {
    #[must_use]
    pub fn new(
        profile_version: impl Into<String>,
        stream_semantics_version: StreamSemanticsVersion,
    ) -> Self {
        Self {
            profile_version: profile_version.into(),
            stream_semantics_version,
        }
    }

    #[must_use]
    pub fn replay_profile_key(&self) -> String {
        format!(
            "{}|stream_semantics:{}",
            self.profile_version,
            self.stream_semantics_version.selector_key()
        )
    }

    #[must_use]
    pub fn behavior(&self) -> StreamSemanticsBehavior {
        self.stream_semantics_version.behavior()
    }

    pub fn dispatch_topic_updates(
        &self,
        repository: &mut CalculationRepository,
        updates: impl IntoIterator<Item = TopicEnvelopeUpdate>,
    ) -> StreamUpdateDispatch {
        let updates = updates.into_iter().collect::<Vec<_>>();
        let observed_update_count = updates.len();
        let behavior = self.behavior();
        let applied_envelopes = if behavior.replay_records_topic_envelope {
            repository.apply_topic_envelope_updates(updates)
        } else {
            Vec::new()
        };

        StreamUpdateDispatch {
            profile_version: self.profile_version.clone(),
            stream_semantics_version: self.stream_semantics_version,
            behavior,
            observed_update_count,
            applied_envelopes,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StreamUpdateDispatch {
    pub profile_version: String,
    pub stream_semantics_version: StreamSemanticsVersion,
    pub behavior: StreamSemanticsBehavior,
    pub observed_update_count: usize,
    pub applied_envelopes: Vec<TopicEnvelope>,
}

#[cfg(test)]
mod tests {
    use crate::repository::{CalculationRepository, SubscriptionTopicId, TopicEnvelopeUpdate};
    use crate::structural::{
        StructuralNode, StructuralNodeKind, StructuralSnapshot, StructuralSnapshotId, TreeNodeId,
    };

    use super::*;

    fn snapshot() -> StructuralSnapshot {
        StructuralSnapshot::create(
            StructuralSnapshotId(1),
            TreeNodeId(1),
            [
                StructuralNode {
                    node_id: TreeNodeId(1),
                    kind: StructuralNodeKind::Root,
                    symbol: "Root".to_string(),
                    parent_id: None,
                    child_ids: vec![TreeNodeId(2), TreeNodeId(3)],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(2),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "A".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
                StructuralNode {
                    node_id: TreeNodeId(3),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "B".to_string(),
                    parent_id: Some(TreeNodeId(1)),
                    child_ids: vec![],
                    formula_artifact_id: None,
                    bind_artifact_id: None,
                    constant_value: None,
                },
            ],
        )
        .unwrap()
    }

    fn topic_update(
        topic: &str,
        sequence: u64,
        payload_ref: &str,
        ordering_key: &str,
        dedupe_identity: &str,
    ) -> TopicEnvelopeUpdate {
        TopicEnvelopeUpdate {
            topic_id: SubscriptionTopicId(topic.to_string()),
            topic_sequence: sequence,
            payload_ref: payload_ref.to_string(),
            ordering_key: ordering_key.to_string(),
            dedupe_identity: dedupe_identity.to_string(),
        }
    }

    fn topic_updates() -> Vec<TopicEnvelopeUpdate> {
        vec![
            topic_update(
                "topic:rtd:status",
                1,
                "payload:rtd:status:1",
                "002",
                "event:rtd:status:1",
            ),
            topic_update(
                "topic:rtd:price",
                2,
                "payload:rtd:price:2",
                "003",
                "event:rtd:price:2",
            ),
            topic_update(
                "topic:rtd:price",
                1,
                "payload:rtd:price:1",
                "001",
                "event:rtd:price:1",
            ),
            topic_update(
                "topic:rtd:price",
                2,
                "payload:rtd:price:duplicate",
                "004",
                "event:rtd:price:2",
            ),
        ]
    }

    #[test]
    fn stream_semantics_profile_serializes_selector() {
        let profile = StreamSemanticsProfile::new(
            "profile:stream:v1",
            StreamSemanticsVersion::TopicEnvelopeV1,
        );

        assert_eq!(
            profile.replay_profile_key(),
            "profile:stream:v1|stream_semantics:TopicEnvelopeV1"
        );
        let json = serde_json::to_value(&profile).unwrap();
        assert_eq!(json["profile_version"], "profile:stream:v1");
        assert_eq!(json["stream_semantics_version"], "TopicEnvelopeV1");

        let round_trip: StreamSemanticsProfile = serde_json::from_value(json).unwrap();
        assert_eq!(round_trip, profile);
    }

    #[test]
    fn stream_semantics_selector_dispatches_three_versions() {
        let updates = topic_updates();
        let v0_profile = StreamSemanticsProfile::new(
            "profile:stream:v0",
            StreamSemanticsVersion::ExternalInvalidationV0,
        );
        let mut v0_repository = CalculationRepository::new(snapshot());
        let v0_dispatch = v0_profile.dispatch_topic_updates(&mut v0_repository, updates.clone());

        assert_eq!(v0_dispatch.observed_update_count, 4);
        assert!(v0_dispatch.applied_envelopes.is_empty());
        assert!(v0_repository.topic_envelopes().is_empty());
        assert_eq!(
            v0_dispatch.behavior.hooks,
            vec![StreamSemanticsBehaviorHook::ExternalInvalidationDirtySeed]
        );

        let v1_profile = StreamSemanticsProfile::new(
            "profile:stream:v1",
            StreamSemanticsVersion::TopicEnvelopeV1,
        );
        let mut v1_forward_repository = CalculationRepository::new(snapshot());
        let mut v1_reverse_repository = CalculationRepository::new(snapshot());
        let v1_forward =
            v1_profile.dispatch_topic_updates(&mut v1_forward_repository, updates.clone());
        let v1_reverse = v1_profile.dispatch_topic_updates(
            &mut v1_reverse_repository,
            updates.clone().into_iter().rev(),
        );

        assert_eq!(v1_forward.applied_envelopes, v1_reverse.applied_envelopes);
        assert_eq!(
            v1_forward_repository.topic_envelopes(),
            v1_reverse_repository.topic_envelopes()
        );
        assert!(
            v1_forward
                .behavior
                .hooks
                .contains(&StreamSemanticsBehaviorHook::TopicEnvelopeOrderingDedupe)
        );
        assert!(
            !v1_forward
                .behavior
                .hooks
                .contains(&StreamSemanticsBehaviorHook::RtdLifecycleTracking)
        );

        let v2_profile = StreamSemanticsProfile::new(
            "profile:stream:v2",
            StreamSemanticsVersion::RtdLifecycleV2,
        );
        let mut v2_repository = CalculationRepository::new(snapshot());
        let v2_dispatch = v2_profile.dispatch_topic_updates(&mut v2_repository, updates);

        assert_eq!(v2_dispatch.applied_envelopes.len(), 3);
        assert!(
            v2_dispatch
                .behavior
                .hooks
                .contains(&StreamSemanticsBehaviorHook::RtdLifecycleTracking)
        );
        let price = v2_repository
            .topic_envelope(&SubscriptionTopicId("topic:rtd:price".to_string()))
            .expect("price envelope exists under RTD lifecycle selector");
        assert_eq!(price.topic_sequence, 2);
        assert_eq!(price.last_observed_payload_ref, "payload:rtd:price:2");
    }
}
