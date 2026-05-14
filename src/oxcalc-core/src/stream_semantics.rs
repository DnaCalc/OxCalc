#![forbid(unsafe_code)]

//! Profile-governed stream semantics selector for external invalidation.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::dependency::InvalidationClosure;
use crate::repository::{
    CalculationRepository, ExternalInvalidationDirtySeed, SubscriptionTopicId, TopicEnvelope,
    TopicEnvelopeUpdate,
};

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
        self.dispatch_collected_topic_updates(repository, updates)
    }

    pub fn dispatch_external_invalidation_updates(
        &self,
        repository: &mut CalculationRepository,
        updates: impl IntoIterator<Item = TopicEnvelopeUpdate>,
    ) -> ExternalInvalidationDispatch {
        let updates = updates.into_iter().collect::<Vec<_>>();
        let stream_dispatch = self.dispatch_collected_topic_updates(repository, updates.clone());
        let dirty_seed_sources = self.dirty_seed_sources(&updates, &stream_dispatch);
        let dirty_seeds = dirty_seed_sources
            .into_iter()
            .flat_map(|(topic_id, topic_sequence)| {
                repository.external_invalidation_dirty_seeds(&topic_id, topic_sequence)
            })
            .collect::<Vec<_>>();
        let invalidation_closure = repository.derive_external_invalidation_closure(&dirty_seeds);

        ExternalInvalidationDispatch {
            stream_dispatch,
            dirty_seeds,
            invalidation_closure,
        }
    }

    fn dispatch_collected_topic_updates(
        &self,
        repository: &mut CalculationRepository,
        updates: Vec<TopicEnvelopeUpdate>,
    ) -> StreamUpdateDispatch {
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

    fn dirty_seed_sources(
        &self,
        updates: &[TopicEnvelopeUpdate],
        stream_dispatch: &StreamUpdateDispatch,
    ) -> Vec<(SubscriptionTopicId, u64)> {
        if stream_dispatch.behavior.replay_records_topic_envelope {
            return stream_dispatch
                .applied_envelopes
                .iter()
                .map(|envelope| (envelope.topic_id.clone(), envelope.topic_sequence))
                .collect();
        }

        let mut seen = BTreeSet::new();
        for update in updates {
            seen.insert((update.topic_id.clone(), update.topic_sequence));
        }
        seen.into_iter().collect()
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalInvalidationDispatch {
    pub stream_dispatch: StreamUpdateDispatch,
    pub dirty_seeds: Vec<ExternalInvalidationDirtySeed>,
    pub invalidation_closure: InvalidationClosure,
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use oxfml_core::EvaluationBackend;
    use oxfml_core::consumer::runtime::{RuntimeEnvironment, RuntimeFormulaRequest};
    use oxfml_core::interface::TypedContextQueryBundle;
    use oxfml_core::seam::ValuePayload;
    use oxfml_core::source::FormulaSourceRecord;

    use crate::coordinator::{AcceptedCandidateResult, TreeCalcCoordinator};
    use crate::dependency::{
        DependencyDescriptor, DependencyDescriptorKind, InvalidationReasonKind,
    };
    use crate::recalc::NodeCalcState;
    use crate::recalc_wave::OxfmlRecalcWave;
    use crate::repository::{
        CalculationRepository, FormulaSlotRecord, FormulaSourceIdentity, SubscriptionHandle,
        SubscriptionRegistryEntry, SubscriptionTopicId, TopicEnvelopeUpdate,
    };
    use crate::structural::{
        BindArtifactId, FormulaArtifactId, StructuralNode, StructuralNodeKind, StructuralSnapshot,
        StructuralSnapshotId, TreeNodeId,
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
                    child_ids: vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)],
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
                StructuralNode {
                    node_id: TreeNodeId(4),
                    kind: StructuralNodeKind::Calculation,
                    symbol: "C".to_string(),
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

    fn formula_slot(node_id: u64, formula_id: &str, text: &str) -> FormulaSlotRecord {
        FormulaSlotRecord {
            owner_node_id: TreeNodeId(node_id),
            formula_artifact_id: FormulaArtifactId(formula_id.to_string()),
            bind_artifact_id: Some(BindArtifactId(format!("bind:{formula_id}"))),
            source_identity: FormulaSourceIdentity {
                formula_stable_id: format!("slot:{node_id}"),
                formula_text_version: 1,
                formula_token: Some(format!("token:{formula_id}")),
            },
            opaque_source_text: text.to_string(),
        }
    }

    fn subscription(topic: &str, stable_id: &str, handle: &str) -> SubscriptionRegistryEntry {
        SubscriptionRegistryEntry {
            topic_id: SubscriptionTopicId(topic.to_string()),
            formula_stable_id: stable_id.to_string(),
            subscription_handle: SubscriptionHandle(handle.to_string()),
            topic_descriptor: format!("rtd:{topic}"),
        }
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

    fn request(formula_stable_id: &str, formula_text: &str) -> RuntimeFormulaRequest<'static> {
        RuntimeFormulaRequest::new(
            FormulaSourceRecord::new(formula_stable_id, 1, formula_text),
            TypedContextQueryBundle::default(),
        )
        .with_backend(EvaluationBackend::OxFuncBacked)
    }

    fn candidate(snapshot: &StructuralSnapshot) -> AcceptedCandidateResult {
        AcceptedCandidateResult {
            candidate_result_id: "candidate:external:price".to_string(),
            structural_snapshot_id: snapshot.snapshot_id(),
            artifact_token_basis: "artifact:external:price".to_string(),
            compatibility_basis: "compat:external:price".to_string(),
            target_set: vec![TreeNodeId(2)],
            value_updates: BTreeMap::from([(TreeNodeId(2), "fresh".to_string())]),
            dependency_shape_updates: Vec::new(),
            runtime_effects: Vec::new(),
            diagnostic_events: Vec::new(),
        }
    }

    fn external_repository(snapshot: &StructuralSnapshot) -> CalculationRepository {
        let mut repository = CalculationRepository::new(snapshot.clone());
        repository
            .upsert_formula_slot(TreeNodeId(2), formula_slot(2, "formula:a", "=RTD(...)"))
            .unwrap();
        repository
            .upsert_formula_slot(TreeNodeId(3), formula_slot(3, "formula:b", "=RTD(...)"))
            .unwrap();
        repository
            .register_subscription(subscription("topic:rtd:price", "slot:2", "sub:slot2:price"))
            .unwrap();
        repository
            .register_subscription(subscription("topic:rtd:price", "slot:3", "sub:slot3:price"))
            .unwrap();
        repository.rebuild_dependency_graph(&[DependencyDescriptor {
            descriptor_id: "dep:report:price".to_string(),
            source_reference_handle: None,
            owner_node_id: TreeNodeId(4),
            target_node_id: Some(TreeNodeId(2)),
            kind: DependencyDescriptorKind::StaticDirect,
            carrier_detail: "report reads price".to_string(),
            requires_rebind_on_structural_change: false,
        }]);
        repository
            .seed_published_value(TreeNodeId(2), "old")
            .expect("published seed target exists");
        repository
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

    #[test]
    fn external_invalidation_dispatch_fans_out_without_side_channel_publication() {
        let snapshot = snapshot();
        let mut repository = external_repository(&snapshot);

        let profile = StreamSemanticsProfile::new(
            "profile:stream:v1",
            StreamSemanticsVersion::TopicEnvelopeV1,
        );
        let dispatch = profile.dispatch_external_invalidation_updates(
            &mut repository,
            [topic_update(
                "topic:rtd:price",
                7,
                "payload:rtd:price:7",
                "007",
                "event:rtd:price:7",
            )],
        );

        assert_eq!(dispatch.stream_dispatch.applied_envelopes.len(), 1);
        assert_eq!(
            dispatch
                .dirty_seeds
                .iter()
                .map(|seed| (seed.formula_stable_id.as_str(), seed.node_id))
                .collect::<Vec<_>>(),
            vec![("slot:2", TreeNodeId(2)), ("slot:3", TreeNodeId(3))]
        );
        assert_eq!(
            dispatch.invalidation_closure.impacted_order,
            vec![TreeNodeId(2), TreeNodeId(3), TreeNodeId(4)]
        );
        assert_eq!(
            dispatch.invalidation_closure.records[&TreeNodeId(2)].reasons,
            vec![InvalidationReasonKind::ExternallyInvalidated]
        );
        assert_eq!(
            dispatch.invalidation_closure.records[&TreeNodeId(4)].reasons,
            vec![InvalidationReasonKind::UpstreamPublication]
        );

        let closure = dispatch.invalidation_closure.clone();
        repository.apply_invalidation_closure(&closure);
        assert_eq!(
            repository.node_states()[&TreeNodeId(2)],
            NodeCalcState::Needed
        );
        assert_eq!(
            repository.node_states()[&TreeNodeId(3)],
            NodeCalcState::Needed
        );
        assert_eq!(
            repository.node_states()[&TreeNodeId(4)],
            NodeCalcState::DirtyPending
        );
        assert_eq!(repository.published_values()[&TreeNodeId(2)], "old");

        let mut wave = OxfmlRecalcWave::new("wave:external:price", RuntimeEnvironment::new());
        let request = request("slot:2", "=SUM(1,2)");
        wave.ensure_prepared(&request)
            .expect("external dirty path still uses session prepare");
        wave.derive_dependencies(1, closure.impacted_order.len())
            .expect("external dirty path still uses ordinary dependency phase");
        let run = wave
            .invoke(request, dispatch.dirty_seeds.len())
            .expect("external dirty path still uses ordinary session invoke");
        assert_eq!(
            run.candidate_result.value_delta.published_payload,
            ValuePayload::Number("3".to_string())
        );

        let mut coordinator = TreeCalcCoordinator::new(snapshot.clone());
        coordinator.seed_published_view(
            &BTreeMap::from([(TreeNodeId(2), "old".to_string())]),
            Some("publication:seed"),
            &[],
        );
        assert_eq!(coordinator.counters().publication_count, 0);
        assert_eq!(coordinator.published_view().values[&TreeNodeId(2)], "old");

        coordinator
            .admit_candidate_work(candidate(&snapshot))
            .expect("ordinary candidate admission should accept the external rerun result");
        coordinator
            .record_accepted_candidate_result("candidate:external:price")
            .expect("ordinary candidate result should be accepted before publish");
        let publication = coordinator
            .accept_and_publish("publication:external:price")
            .expect("ordinary coordinator commit should publish the external rerun result");

        assert_eq!(publication.published_view_delta[&TreeNodeId(2)], "fresh");
        assert_eq!(coordinator.counters().publication_count, 1);
        assert_eq!(coordinator.published_view().values[&TreeNodeId(2)], "fresh");
    }

    #[test]
    fn external_invalidation_v0_fans_out_without_envelope_mutation() {
        let snapshot = snapshot();
        let mut repository = external_repository(&snapshot);
        let profile = StreamSemanticsProfile::new(
            "profile:stream:v0",
            StreamSemanticsVersion::ExternalInvalidationV0,
        );

        let dispatch = profile.dispatch_external_invalidation_updates(
            &mut repository,
            [topic_update(
                "topic:rtd:price",
                8,
                "payload:rtd:price:8",
                "008",
                "event:rtd:price:8",
            )],
        );

        assert!(dispatch.stream_dispatch.applied_envelopes.is_empty());
        assert!(repository.topic_envelopes().is_empty());
        assert_eq!(dispatch.dirty_seeds.len(), 2);
        assert_eq!(
            dispatch.invalidation_closure.records[&TreeNodeId(2)].reasons,
            vec![InvalidationReasonKind::ExternallyInvalidated]
        );
    }
}
