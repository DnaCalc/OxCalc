---- MODULE CoreEngineW048CycleRegions ----
EXTENDS Naturals, FiniteSets, Sequences

(***************************************************************************)
(* W048 model sketch. This module names the cycle-region obligations checked *)
(* by scripts/check-w048-formal-cycle-artifacts.py over concrete run JSON.   *)
(***************************************************************************)

CONSTANTS Nodes, Layers, ForwardEdges, ReverseEdges, CycleRegions

EdgeNodes(e) == e \in Nodes \X Nodes

ForwardReverseConverse ==
  \A e \in ForwardEdges : <<e[2], e[1]>> \in ReverseEdges

ReverseForwardConverse ==
  \A e \in ReverseEdges : <<e[2], e[1]>> \in ForwardEdges

CycleRegionNonEmpty ==
  \A r \in CycleRegions : Cardinality(r) >= 1

CycleRegionIsSelfLoopOrNontrivial(r) ==
  Cardinality(r) > 1 \/ \E n \in r : <<n, n>> \in ForwardEdges

CycleRegionsAreSccWitnesses ==
  \A r \in CycleRegions : CycleRegionIsSelfLoopOrNontrivial(r)

NoPublicationOnReject(rejected, publication_bundle) ==
  rejected => publication_bundle = NULL

ReleaseReentryPublishes(post_edit_state, owner_value, downstream_value) ==
  /\ post_edit_state = "published"
  /\ owner_value # NULL
  /\ downstream_value # NULL

W048GraphInvariant ==
  /\ ForwardReverseConverse
  /\ ReverseForwardConverse
  /\ CycleRegionNonEmpty
  /\ CycleRegionsAreSccWitnesses

====
