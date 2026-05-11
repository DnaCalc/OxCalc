---- MODULE CoreEngineW046FiniteGraphDataflowOrder ----
EXTENDS Naturals, Sequences, FiniteSets, TLC

CONSTANTS MaxTransitions

Nodes == {"A", "B", "C", "D", "E", "F"}
Shapes == {"chain", "diamond", "fanout_rebind", "self_cycle", "two_node_scc"}

VARIABLES
  phase,
  shape,
  edges,
  reverseEdges,
  seeds,
  closure,
  formulaOwners,
  stableInputs,
  order,
  cycleGroups,
  rebindNodes,
  terminal,
  transitionHistory

vars == <<
  phase,
  shape,
  edges,
  reverseEdges,
  seeds,
  closure,
  formulaOwners,
  stableInputs,
  order,
  cycleGroups,
  rebindNodes,
  terminal,
  transitionHistory
>>

Edge(owner, target) == [owner |-> owner, target |-> target]
ReverseRecord(target, owner) == [target |-> target, owner |-> owner]
AppendTransition(label) == Append(transitionHistory, label)

ChainEdges == {Edge("B", "A"), Edge("C", "B")}
DiamondEdges == {Edge("B", "A"), Edge("C", "A"), Edge("D", "B"), Edge("D", "C")}
FanoutRebindEdges == {Edge("B", "A"), Edge("C", "A"), Edge("D", "B"), Edge("E", "B")}
SelfCycleEdges == {Edge("F", "F")}
TwoNodeSccEdges == {Edge("B", "C"), Edge("C", "B")}

ChainReverse == {ReverseRecord("A", "B"), ReverseRecord("B", "C")}
DiamondReverse == {
  ReverseRecord("A", "B"), ReverseRecord("A", "C"),
  ReverseRecord("B", "D"), ReverseRecord("C", "D")}
FanoutRebindReverse == {
  ReverseRecord("A", "B"), ReverseRecord("A", "C"),
  ReverseRecord("B", "D"), ReverseRecord("B", "E")}
SelfCycleReverse == {ReverseRecord("F", "F")}
TwoNodeSccReverse == {ReverseRecord("C", "B"), ReverseRecord("B", "C")}

Init ==
  /\ phase = "start"
  /\ shape = "chain"
  /\ edges = {}
  /\ reverseEdges = {}
  /\ seeds = {}
  /\ closure = {}
  /\ formulaOwners = {}
  /\ stableInputs = {}
  /\ order = <<>>
  /\ cycleGroups = {}
  /\ rebindNodes = {}
  /\ terminal = "none"
  /\ transitionHistory = <<>>

BindChain ==
  /\ phase = "start"
  /\ phase' = "checked"
  /\ shape' = "chain"
  /\ edges' = ChainEdges
  /\ reverseEdges' = ChainReverse
  /\ seeds' = {"A"}
  /\ closure' = {"A", "B", "C"}
  /\ formulaOwners' = {"B", "C"}
  /\ stableInputs' = {"A"}
  /\ order' = <<"B", "C">>
  /\ cycleGroups' = {}
  /\ rebindNodes' = {}
  /\ terminal' = "published"
  /\ transitionHistory' = AppendTransition("chain")

BindDiamond ==
  /\ phase = "start"
  /\ phase' = "checked"
  /\ shape' = "diamond"
  /\ edges' = DiamondEdges
  /\ reverseEdges' = DiamondReverse
  /\ seeds' = {"A"}
  /\ closure' = {"A", "B", "C", "D"}
  /\ formulaOwners' = {"B", "C", "D"}
  /\ stableInputs' = {"A"}
  /\ order' = <<"B", "C", "D">>
  /\ cycleGroups' = {}
  /\ rebindNodes' = {}
  /\ terminal' = "published"
  /\ transitionHistory' = AppendTransition("diamond")

BindFanoutRebind ==
  /\ phase = "start"
  /\ phase' = "checked"
  /\ shape' = "fanout_rebind"
  /\ edges' = FanoutRebindEdges
  /\ reverseEdges' = FanoutRebindReverse
  /\ seeds' = {"A", "B"}
  /\ closure' = {"A", "B", "C", "D", "E"}
  /\ formulaOwners' = {"B", "C", "D", "E"}
  /\ stableInputs' = {"A"}
  /\ order' = <<>>
  /\ cycleGroups' = {}
  /\ rebindNodes' = {"B"}
  /\ terminal' = "rejected"
  /\ transitionHistory' = AppendTransition("fanout_rebind")

BindSelfCycle ==
  /\ phase = "start"
  /\ phase' = "checked"
  /\ shape' = "self_cycle"
  /\ edges' = SelfCycleEdges
  /\ reverseEdges' = SelfCycleReverse
  /\ seeds' = {"F"}
  /\ closure' = {"F"}
  /\ formulaOwners' = {"F"}
  /\ stableInputs' = {}
  /\ order' = <<>>
  /\ cycleGroups' = {{"F"}}
  /\ rebindNodes' = {}
  /\ terminal' = "rejected"
  /\ transitionHistory' = AppendTransition("self_cycle")

BindTwoNodeScc ==
  /\ phase = "start"
  /\ phase' = "checked"
  /\ shape' = "two_node_scc"
  /\ edges' = TwoNodeSccEdges
  /\ reverseEdges' = TwoNodeSccReverse
  /\ seeds' = {"B"}
  /\ closure' = {"B", "C"}
  /\ formulaOwners' = {"B", "C"}
  /\ stableInputs' = {}
  /\ order' = <<>>
  /\ cycleGroups' = {{"B", "C"}}
  /\ rebindNodes' = {}
  /\ terminal' = "rejected"
  /\ transitionHistory' = AppendTransition("two_node_scc")

TerminalStutter ==
  /\ phase = "checked"
  /\ UNCHANGED vars

Next ==
  \/ BindChain
  \/ BindDiamond
  \/ BindFanoutRebind
  \/ BindSelfCycle
  \/ BindTwoNodeScc
  \/ TerminalStutter

Spec == Init /\ [][Next]_vars

TypeInvariant ==
  /\ phase \in {"start", "checked"}
  /\ shape \in Shapes
  /\ edges \subseteq [owner: Nodes, target: Nodes]
  /\ reverseEdges \subseteq [target: Nodes, owner: Nodes]
  /\ seeds \subseteq Nodes
  /\ closure \subseteq Nodes
  /\ formulaOwners \subseteq Nodes
  /\ stableInputs \subseteq Nodes
  /\ cycleGroups \subseteq SUBSET Nodes
  /\ rebindNodes \subseteq Nodes
  /\ terminal \in {"none", "published", "rejected"}

ReverseConverse ==
  phase = "checked" =>
    /\ \A e \in edges : ReverseRecord(e.target, e.owner) \in reverseEdges
    /\ \A r \in reverseEdges : Edge(r.owner, r.target) \in edges

ClosureCoversExpectedDependents ==
  phase = "checked" =>
    CASE shape = "chain" -> closure = {"A", "B", "C"}
      [] shape = "diamond" -> closure = {"A", "B", "C", "D"}
      [] shape = "fanout_rebind" -> closure = {"A", "B", "C", "D", "E"}
      [] shape = "self_cycle" -> closure = {"F"}
      [] shape = "two_node_scc" -> closure = {"B", "C"}
      [] OTHER -> FALSE

OrderRespectsKnownFiniteShapes ==
  phase = "checked" /\ terminal = "published" =>
    CASE shape = "chain" -> order = <<"B", "C">>
      [] shape = "diamond" -> order = <<"B", "C", "D">>
      [] OTHER -> TRUE

StablePriorReadShapes ==
  phase = "checked" /\ terminal = "published" =>
    CASE shape = "chain" -> stableInputs = {"A"} /\ formulaOwners = {"B", "C"}
      [] shape = "diamond" -> stableInputs = {"A"} /\ formulaOwners = {"B", "C", "D"}
      [] OTHER -> TRUE

CyclesRejectRatherThanOrder ==
  phase = "checked" /\ cycleGroups # {} => terminal = "rejected" /\ order = <<>>

RebindRejectsRatherThanPublishes ==
  phase = "checked" /\ rebindNodes # {} => terminal = "rejected"

PublishedOnlyAcyclicNoRebind ==
  phase = "checked" /\ terminal = "published" => cycleGroups = {} /\ rebindNodes = {}

ExplorationConstraint ==
  Len(transitionHistory) <= MaxTransitions

====
