import Std

namespace OxCalc.CoreEngine.W046.DependencyGraph

abbrev NodeId := Nat
abbrev DescriptorId := String
abbrev EdgeId := String

inductive DescriptorKind where
  | staticDirect
  | relativeBound
  | dynamicPotential
  | hostSensitive
  | capabilitySensitive
  | shapeTopology
  | unresolved
  deriving DecidableEq, Repr

inductive DiagnosticKind where
  | missingOwner
  | missingTarget
  | unresolvedReference
  | hostSensitiveReference
  | dynamicPotentialReference
  | capabilitySensitiveReference
  | shapeTopologyReference
  deriving DecidableEq, Repr

inductive CycleGroupKind where
  | nontrivialScc
  | selfCycle
  deriving DecidableEq, Repr

structure Descriptor where
  descriptorId : DescriptorId
  owner : NodeId
  target? : Option NodeId
  kind : DescriptorKind
  requiresRebindOnStructuralChange : Bool
  deriving DecidableEq, Repr

structure Edge where
  edgeId : EdgeId
  descriptorId : DescriptorId
  owner : NodeId
  target : NodeId
  kind : DescriptorKind
  deriving DecidableEq, Repr

structure Diagnostic where
  descriptorId : DescriptorId
  kind : DiagnosticKind
  deriving DecidableEq, Repr

structure CycleGroup where
  members : List NodeId
  kind : CycleGroupKind
  deriving DecidableEq, Repr

structure Graph where
  snapshotNodes : List NodeId
  descriptors : List Descriptor
  forwardEdges : List Edge
  reverseEdges : List (NodeId × Edge)
  diagnostics : List Diagnostic
  cycleGroups : List CycleGroup
  deriving DecidableEq, Repr

def BuildReverseEdges (edges : List Edge) : List (NodeId × Edge) :=
  edges.map (fun edge => (edge.target, edge))

def EdgeTargetsSnapshotNode (g : Graph) : Prop :=
  forall edge, edge ∈ g.forwardEdges -> edge.owner ∈ g.snapshotNodes ∧ edge.target ∈ g.snapshotNodes

def ReverseEdgeConverse (g : Graph) : Prop :=
  forall target edge,
    (target, edge) ∈ g.reverseEdges <-> edge ∈ g.forwardEdges ∧ edge.target = target

def DiagnosticRequired (g : Graph) (descriptor : Descriptor) : Prop :=
  descriptor.owner ∉ g.snapshotNodes ∨
    (match descriptor.target? with
      | none => True
      | some target => target ∉ g.snapshotNodes)

def DiagnosticForDescriptor (descriptor : Descriptor) (diagnostic : Diagnostic) : Prop :=
  diagnostic.descriptorId = descriptor.descriptorId

def DiagnosticsPreserved (g : Graph) : Prop :=
  forall descriptor,
    descriptor ∈ g.descriptors ->
      DiagnosticRequired g descriptor ->
        exists diagnostic,
          diagnostic ∈ g.diagnostics ∧ DiagnosticForDescriptor descriptor diagnostic

def HasSelfLoop (edges : List Edge) (node : NodeId) : Prop :=
  exists edge, edge ∈ edges ∧ edge.owner = node ∧ edge.target = node

def CycleGroupSupported (edges : List Edge) (group : CycleGroup) : Prop :=
  match group.kind with
  | CycleGroupKind.nontrivialScc => group.members.length > 1
  | CycleGroupKind.selfCycle =>
      exists node, group.members = [node] ∧ HasSelfLoop edges node

def CycleGroupsClassified (g : Graph) : Prop :=
  forall group,
    group ∈ g.cycleGroups ->
      group.members.all (fun node => node ∈ g.snapshotNodes) = true ∧
        CycleGroupSupported g.forwardEdges group

structure BuildGraphSemanticModel (g : Graph) where
  edgeTargetsSnapshotNode : EdgeTargetsSnapshotNode g
  reverseEdgeConverse : ReverseEdgeConverse g
  diagnosticsPreserved : DiagnosticsPreserved g
  cycleGroupsClassified : CycleGroupsClassified g

theorem buildReverseEdges_converse
    (edges : List Edge)
    (target : NodeId)
    (edge : Edge) :
    (target, edge) ∈ BuildReverseEdges edges <-> edge ∈ edges ∧ edge.target = target := by
  unfold BuildReverseEdges
  constructor
  · intro membership
    rcases List.mem_map.mp membership with ⟨sourceEdge, sourceMembership, pairEq⟩
    cases pairEq
    exact ⟨sourceMembership, rfl⟩
  · intro witness
    rcases witness with ⟨edgeMembership, targetEq⟩
    rw [← targetEq]
    exact List.mem_map.mpr ⟨edge, edgeMembership, rfl⟩

theorem graphModel_forwardEdges_have_snapshot_targets
    {g : Graph}
    (model : BuildGraphSemanticModel g)
    {edge : Edge}
    (membership : edge ∈ g.forwardEdges) :
    edge.owner ∈ g.snapshotNodes ∧ edge.target ∈ g.snapshotNodes := by
  exact model.edgeTargetsSnapshotNode edge membership

theorem graphModel_reverse_edges_are_converse
    {g : Graph}
    (model : BuildGraphSemanticModel g)
    (target : NodeId)
    (edge : Edge) :
    (target, edge) ∈ g.reverseEdges <-> edge ∈ g.forwardEdges ∧ edge.target = target := by
  exact model.reverseEdgeConverse target edge

theorem graphModel_diagnostics_preserve_required_descriptors
    {g : Graph}
    (model : BuildGraphSemanticModel g)
    {descriptor : Descriptor}
    (membership : descriptor ∈ g.descriptors)
    (required : DiagnosticRequired g descriptor) :
    exists diagnostic,
      diagnostic ∈ g.diagnostics ∧ DiagnosticForDescriptor descriptor diagnostic := by
  exact model.diagnosticsPreserved descriptor membership required

theorem graphModel_cycle_groups_are_classified
    {g : Graph}
    (model : BuildGraphSemanticModel g)
    {group : CycleGroup}
    (membership : group ∈ g.cycleGroups) :
    group.members.all (fun node => node ∈ g.snapshotNodes) = true ∧
      CycleGroupSupported g.forwardEdges group := by
  exact model.cycleGroupsClassified group membership

def SampleEdgeAB : Edge :=
  { edgeId := "dep:1:2:a",
    descriptorId := "a",
    owner := 1,
    target := 2,
    kind := DescriptorKind.staticDirect }

def SampleEdgeBA : Edge :=
  { edgeId := "dep:2:1:b",
    descriptorId := "b",
    owner := 2,
    target := 1,
    kind := DescriptorKind.relativeBound }

theorem sampleReverseConverse_AB :
    (2, SampleEdgeAB) ∈ BuildReverseEdges [SampleEdgeAB, SampleEdgeBA] := by
  unfold BuildReverseEdges SampleEdgeAB SampleEdgeBA
  simp

theorem sampleReverseConverse_BA :
    (1, SampleEdgeBA) ∈ BuildReverseEdges [SampleEdgeAB, SampleEdgeBA] := by
  unfold BuildReverseEdges SampleEdgeAB SampleEdgeBA
  simp

end OxCalc.CoreEngine.W046.DependencyGraph
