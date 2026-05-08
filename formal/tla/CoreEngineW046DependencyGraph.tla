---- MODULE CoreEngineW046DependencyGraph ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS MaxTransitions

Nodes == {"A", "B", "C"}
DescriptorIds == {"dAB", "dBA", "dDynamic"}
DescriptorKinds ==
  {"StaticDirect", "RelativeBound", "DynamicPotential", "HostSensitive", "CapabilitySensitive", "ShapeTopology", "Unresolved"}
NoTarget == "NoTarget"
CycleGroupsInput == {{"A", "B"}}

ASSUME /\ NoTarget \notin Nodes
       /\ MaxTransitions \in Nat
       /\ MaxTransitions > 0

VARIABLES
  descriptors,
  forwardEdges,
  reverseEdges,
  diagnostics,
  cycleGroups,
  phase,
  transitionHistory

vars == <<descriptors, forwardEdges, reverseEdges, diagnostics, cycleGroups, phase, transitionHistory>>

OwnerOf(d) ==
  IF d = "dAB" THEN "A"
  ELSE IF d = "dBA" THEN "B"
  ELSE "C"

TargetOf(d) ==
  IF d = "dAB" THEN "B"
  ELSE IF d = "dBA" THEN "A"
  ELSE NoTarget

KindOf(d) ==
  IF d = "dAB" THEN "StaticDirect"
  ELSE IF d = "dBA" THEN "RelativeBound"
  ELSE "DynamicPotential"

TargetedDescriptor(d) == TargetOf(d) # NoTarget

ValidTargetedDescriptors ==
  {d \in DescriptorIds : TargetedDescriptor(d) /\ TargetOf(d) \in Nodes}

UntargetedDescriptors ==
  {d \in DescriptorIds : ~TargetedDescriptor(d)}

EdgeRecord(d) ==
  [
    descriptorId |-> d,
    owner |-> OwnerOf(d),
    target |-> TargetOf(d),
    kind |-> KindOf(d)
  ]

DiagnosticKindFor(d) ==
  IF KindOf(d) = "HostSensitive" THEN "HostSensitiveReference"
  ELSE IF KindOf(d) = "DynamicPotential" THEN "DynamicPotentialReference"
  ELSE IF KindOf(d) = "CapabilitySensitive" THEN "CapabilitySensitiveReference"
  ELSE IF KindOf(d) = "ShapeTopology" THEN "ShapeTopologyReference"
  ELSE "UnresolvedReference"

DiagnosticRecord(d) ==
  [
    descriptorId |-> d,
    kind |-> DiagnosticKindFor(d)
  ]

ReverseRecord(edge) ==
  [
    target |-> edge.target,
    edge |-> edge
  ]

HasEdge(owner, target) ==
  \E edge \in forwardEdges : edge.owner = owner /\ edge.target = target

HasSelfLoop(node) == HasEdge(node, node)

Init ==
  /\ descriptors = {}
  /\ forwardEdges = {}
  /\ reverseEdges = {}
  /\ diagnostics = {}
  /\ cycleGroups = {}
  /\ phase = "empty"
  /\ transitionHistory = <<>>

BuildGraph ==
  /\ phase = "empty"
  /\ descriptors' = DescriptorIds
  /\ forwardEdges' = {EdgeRecord(d) : d \in ValidTargetedDescriptors}
  /\ reverseEdges' = {ReverseRecord(edge) : edge \in forwardEdges'}
  /\ diagnostics' = {DiagnosticRecord(d) : d \in UntargetedDescriptors}
  /\ cycleGroups' = CycleGroupsInput
  /\ phase' = "built"
  /\ transitionHistory' = Append(transitionHistory, "BuildGraph")

StutterAfterBuild ==
  /\ phase = "built"
  /\ UNCHANGED vars

Next == BuildGraph \/ StutterAfterBuild

Spec == Init /\ [][Next]_vars

TypeInvariant ==
  /\ descriptors \subseteq DescriptorIds
  /\ forwardEdges \subseteq {EdgeRecord(d) : d \in DescriptorIds}
  /\ reverseEdges \subseteq {ReverseRecord(edge) : edge \in {EdgeRecord(d) : d \in DescriptorIds}}
  /\ diagnostics \subseteq {DiagnosticRecord(d) : d \in DescriptorIds}
  /\ cycleGroups \subseteq SUBSET Nodes
  /\ phase \in {"empty", "built"}

ForwardEdgesHaveValidTargets ==
  phase = "empty" \/
    \A edge \in forwardEdges : edge.owner \in Nodes /\ edge.target \in Nodes

ReverseEdgeConverse ==
  phase = "empty" \/
    /\ \A edge \in forwardEdges : ReverseRecord(edge) \in reverseEdges
    /\ \A reverse \in reverseEdges : reverse.edge \in forwardEdges /\ reverse.target = reverse.edge.target

DiagnosticsPreserved ==
  phase = "empty" \/
    \A d \in UntargetedDescriptors : DiagnosticRecord(d) \in diagnostics

CycleGroupClassification ==
  phase = "empty" \/
    \A group \in cycleGroups :
      /\ group \subseteq Nodes
      /\ (Cardinality(group) > 1 \/ \E node \in group : HasSelfLoop(node))

ExplorationConstraint ==
  Len(transitionHistory) <= MaxTransitions

====
