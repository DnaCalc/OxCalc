import Std

namespace OxCalc.CoreEngine.W046.FiniteGraphDataflowOrder

abbrev NodeId := Nat

structure Edge where
  owner : NodeId
  target : NodeId
  deriving DecidableEq, Repr

structure CycleGroup where
  members : List NodeId
  deriving DecidableEq, Repr

def ReverseEdgeFacts (edges : List Edge) (reverseEdges : List (NodeId × Edge)) : Prop :=
  ∀ target edge, (target, edge) ∈ reverseEdges ↔ edge ∈ edges ∧ edge.target = target

inductive ReverseReachable (edges : List Edge) : NodeId -> NodeId -> Prop where
  | seed (node : NodeId) : ReverseReachable edges node node
  | step
      {seed target dependent : NodeId}
      (reachable : ReverseReachable edges seed target)
      (edge : Edge)
      (edgeMember : edge ∈ edges)
      (edgeTarget : edge.target = target)
      (edgeOwner : edge.owner = dependent) :
      ReverseReachable edges seed dependent

def ClosureCoversReverseReachable
    (edges : List Edge)
    (seeds closure : List NodeId) : Prop :=
  ∀ seed,
    seed ∈ seeds ->
      ∀ node,
        ReverseReachable edges seed node -> node ∈ closure

def BeforeInOrder (order : List NodeId) (target owner : NodeId) : Prop :=
  ∃ left middle right,
    order = left ++ [target] ++ middle ++ [owner] ++ right

def TopologicalForEdges
    (order formulaOwners : List NodeId)
    (edges : List Edge) : Prop :=
  ∀ edge,
    edge ∈ edges ->
      edge.owner ∈ formulaOwners ->
        edge.target ∈ formulaOwners ->
          BeforeInOrder order edge.target edge.owner

def StableOrPriorReads
    (order stableInputs formulaOwners : List NodeId)
    (edges : List Edge) : Prop :=
  ∀ edge,
    edge ∈ edges ->
      edge.owner ∈ formulaOwners ->
        edge.target ∈ stableInputs ∨ BeforeInOrder order edge.target edge.owner

def HasSelfLoop (edges : List Edge) (node : NodeId) : Prop :=
  ∃ edge, edge ∈ edges ∧ edge.owner = node ∧ edge.target = node

def CycleGroupSupported (edges : List Edge) (group : CycleGroup) : Prop :=
  group.members.length > 1 ∨ ∃ node, group.members = [node] ∧ HasSelfLoop edges node

structure FiniteGraphDataflowOrderModel where
  nodes : List NodeId
  edges : List Edge
  reverseEdges : List (NodeId × Edge)
  seeds : List NodeId
  closure : List NodeId
  formulaOwners : List NodeId
  stableInputs : List NodeId
  order : List NodeId
  cycleGroups : List CycleGroup
  reverseConverse : ReverseEdgeFacts edges reverseEdges
  closureCoversReachability : ClosureCoversReverseReachable edges seeds closure
  orderRespectsFormulaEdges : TopologicalForEdges order formulaOwners edges
  readsAreStableOrPrior : StableOrPriorReads order stableInputs formulaOwners edges
  cycleGroupsSupported : ∀ group, group ∈ cycleGroups -> CycleGroupSupported edges group

theorem model_reverse_converse
    (model : FiniteGraphDataflowOrderModel) :
    ReverseEdgeFacts model.edges model.reverseEdges := by
  exact model.reverseConverse

theorem model_no_under_invalidation
    (model : FiniteGraphDataflowOrderModel)
    {seed node : NodeId}
    (seedMember : seed ∈ model.seeds)
    (reachable : ReverseReachable model.edges seed node) :
    node ∈ model.closure := by
  exact model.closureCoversReachability seed seedMember node reachable

theorem model_order_respects_formula_edges
    (model : FiniteGraphDataflowOrderModel) :
    TopologicalForEdges model.order model.formulaOwners model.edges := by
  exact model.orderRespectsFormulaEdges

theorem model_reads_are_stable_or_prior
    (model : FiniteGraphDataflowOrderModel) :
    StableOrPriorReads model.order model.stableInputs model.formulaOwners model.edges := by
  exact model.readsAreStableOrPrior

theorem model_cycle_groups_supported
    (model : FiniteGraphDataflowOrderModel)
    {group : CycleGroup}
    (member : group ∈ model.cycleGroups) :
    CycleGroupSupported model.edges group := by
  exact model.cycleGroupsSupported group member

-- Shape 1: a chain A -> B -> C, where arrows mean dependent reads target.
def ChainEdgeBA : Edge := { owner := 2, target := 1 }
def ChainEdgeCB : Edge := { owner := 3, target := 2 }
def ChainEdges : List Edge := [ChainEdgeBA, ChainEdgeCB]
def ChainOrder : List NodeId := [1, 2, 3]

theorem chain_reverse_converse :
    ReverseEdgeFacts ChainEdges [(1, ChainEdgeBA), (2, ChainEdgeCB)] := by
  intro target edge
  constructor
  · intro member
    simp [ChainEdges, ChainEdgeBA, ChainEdgeCB] at member ⊢
    rcases member with first | second
    · rcases first with ⟨targetEq, edgeEq⟩
      subst target
      subst edge
      simp
    · rcases second with ⟨targetEq, edgeEq⟩
      subst target
      subst edge
      simp
  · intro witness
    rcases witness with ⟨edgeMember, targetEq⟩
    simp [ChainEdges, ChainEdgeBA, ChainEdgeCB] at edgeMember ⊢
    rcases edgeMember with edgeEq | edgeEq
    · subst edge
      simp at targetEq
      subst target
      simp
    · subst edge
      simp at targetEq
      subst target
      simp

theorem chain_reachable_A_to_C :
    ReverseReachable ChainEdges 1 3 := by
  apply ReverseReachable.step (target := 2) (edge := ChainEdgeCB)
  · apply ReverseReachable.step (target := 1) (edge := ChainEdgeBA)
    · exact ReverseReachable.seed 1
    · simp [ChainEdges, ChainEdgeBA]
    · rfl
    · rfl
  · simp [ChainEdges, ChainEdgeCB]
  · rfl
  · rfl

theorem chain_closure_covers :
    ClosureCoversReverseReachable ChainEdges [1] [1, 2, 3] := by
  intro seed seedMember node reachable
  simp at seedMember
  subst seed
  induction reachable with
  | seed => simp
  | step reachable edge edgeMember edgeTarget edgeOwner ih =>
      simp [ChainEdges, ChainEdgeBA, ChainEdgeCB] at edgeMember
      rcases edgeMember with edgeEq | edgeEq
      · subst edge
        simp at edgeOwner
        simp [← edgeOwner]
      · subst edge
        simp at edgeOwner
        simp [← edgeOwner]

theorem chain_topological :
    TopologicalForEdges ChainOrder [2, 3] ChainEdges := by
  intro edge edgeMember ownerMember targetMember
  simp [ChainEdges, ChainEdgeBA, ChainEdgeCB] at edgeMember
  rcases edgeMember with edgeEq | edgeEq
  · subst edge
    simp at targetMember
  · subst edge
    unfold BeforeInOrder ChainOrder
    exact ⟨[1], [], [], by simp⟩

theorem chain_stable_or_prior :
    StableOrPriorReads ChainOrder [1] [2, 3] ChainEdges := by
  intro edge edgeMember ownerMember
  simp [ChainEdges, ChainEdgeBA, ChainEdgeCB] at edgeMember
  rcases edgeMember with edgeEq | edgeEq
  · subst edge
    left
    simp
  · subst edge
    right
    unfold BeforeInOrder ChainOrder
    exact ⟨[1], [], [], by simp⟩

def ChainModel : FiniteGraphDataflowOrderModel :=
  { nodes := [1, 2, 3],
    edges := ChainEdges,
    reverseEdges := [(1, ChainEdgeBA), (2, ChainEdgeCB)],
    seeds := [1],
    closure := [1, 2, 3],
    formulaOwners := [2, 3],
    stableInputs := [1],
    order := ChainOrder,
    cycleGroups := [],
    reverseConverse := chain_reverse_converse,
    closureCoversReachability := chain_closure_covers,
    orderRespectsFormulaEdges := chain_topological,
    readsAreStableOrPrior := chain_stable_or_prior,
    cycleGroupsSupported := by intro group member; simp at member }

-- Shape 2: a diamond A -> B, A -> C, B/C -> D.
def DiamondEdgeBA : Edge := { owner := 2, target := 1 }
def DiamondEdgeCA : Edge := { owner := 3, target := 1 }
def DiamondEdgeDB : Edge := { owner := 4, target := 2 }
def DiamondEdgeDC : Edge := { owner := 4, target := 3 }
def DiamondEdges : List Edge := [DiamondEdgeBA, DiamondEdgeCA, DiamondEdgeDB, DiamondEdgeDC]
def DiamondOrder : List NodeId := [2, 3, 4]

theorem diamond_topological :
    TopologicalForEdges DiamondOrder [2, 3, 4] DiamondEdges := by
  intro edge edgeMember ownerMember targetMember
  simp [DiamondEdges, DiamondEdgeBA, DiamondEdgeCA, DiamondEdgeDB, DiamondEdgeDC] at edgeMember
  rcases edgeMember with edgeEq | edgeEq | edgeEq | edgeEq
  · subst edge; simp at targetMember
  · subst edge; simp at targetMember
  · subst edge; unfold BeforeInOrder DiamondOrder; exact ⟨[], [3], [], by simp⟩
  · subst edge; unfold BeforeInOrder DiamondOrder; exact ⟨[2], [], [], by simp⟩

theorem diamond_stable_or_prior :
    StableOrPriorReads DiamondOrder [1] [2, 3, 4] DiamondEdges := by
  intro edge edgeMember ownerMember
  simp [DiamondEdges, DiamondEdgeBA, DiamondEdgeCA, DiamondEdgeDB, DiamondEdgeDC] at edgeMember
  rcases edgeMember with edgeEq | edgeEq | edgeEq | edgeEq
  · subst edge; left; simp
  · subst edge; left; simp
  · subst edge; right; unfold BeforeInOrder DiamondOrder; exact ⟨[], [3], [], by simp⟩
  · subst edge; right; unfold BeforeInOrder DiamondOrder; exact ⟨[2], [], [], by simp⟩

-- Shape 3: self-cycle support, routed to cycle/reject rather than order.
def SelfLoopEdge : Edge := { owner := 5, target := 5 }
def SelfLoopGroup : CycleGroup := { members := [5] }

theorem self_loop_group_supported :
    CycleGroupSupported [SelfLoopEdge] SelfLoopGroup := by
  right
  refine ⟨5, ?_, ?_⟩
  · rfl
  · unfold HasSelfLoop
    refine ⟨SelfLoopEdge, ?_, ?_, ?_⟩
    · simp [SelfLoopEdge]
    · simp [SelfLoopEdge]
    · simp [SelfLoopEdge]

-- Shape 4: non-trivial SCC support over two nodes.
def SccEdgeAB : Edge := { owner := 10, target := 11 }
def SccEdgeBA : Edge := { owner := 11, target := 10 }
def TwoNodeSccGroup : CycleGroup := { members := [10, 11] }

theorem two_node_scc_group_supported :
    CycleGroupSupported [SccEdgeAB, SccEdgeBA] TwoNodeSccGroup := by
  unfold CycleGroupSupported TwoNodeSccGroup
  simp

theorem chain_model_reaches_all_dependents :
    3 ∈ ChainModel.closure := by
  exact model_no_under_invalidation ChainModel (by unfold ChainModel; simp) chain_reachable_A_to_C

theorem chain_model_reads :
    StableOrPriorReads ChainModel.order ChainModel.stableInputs ChainModel.formulaOwners ChainModel.edges := by
  exact model_reads_are_stable_or_prior ChainModel

theorem diamond_edges_ordered :
    TopologicalForEdges DiamondOrder [2, 3, 4] DiamondEdges := by
  exact diamond_topological

end OxCalc.CoreEngine.W046.FiniteGraphDataflowOrder
