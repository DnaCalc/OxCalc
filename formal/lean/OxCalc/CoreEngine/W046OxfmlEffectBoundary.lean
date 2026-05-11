import Std

namespace OxCalc.CoreEngine.W046.OxfmlEffectBoundary

abbrev NodeId := String
abbrev Detail := String

inductive Phase where
  | prepareFormula
  | lowerDescriptors
  | graphBuild
  | rebindGate
  | evaluateFormula
  | publication
  | traceProjection
  deriving DecidableEq, Repr

inductive Effect where
  | readValue (reference : String) (context : String)
  | resolveStatic (reference : String) (context : String)
  | resolveDynamic (owner : NodeId) (text : String) (context : String)
  | emitDependency (owner : NodeId) (target : Option NodeId) (kind : String)
  | emitDiagnostic (owner : NodeId) (kind : String)
  | callFunction (functionId : String) (argumentCount : Nat)
  | bindLocal (name : String) (valueRef : String)
  | enterLambda (parameters : List String) (closureId : String)
  | produceCandidate (owner : NodeId) (candidateId : String)
  | rejectCandidate (owner : NodeId) (reason : String)
  | publish (publicationId : String)
  | emitFormatDelta (owner : NodeId) (detail : Detail)
  | emitDisplayDelta (owner : NodeId) (detail : Detail)
  | surfaceRuntimeFact (owner : NodeId) (family : String)
  deriving DecidableEq, Repr

inductive BoundaryFact where
  | candidateResult
  | commitBundle
  | rejectRecord
  | runtimeEffect
  | formatDelta
  | displayDelta
  | letBinding
  | lambdaCarrier
  | registeredExternalPacket
  deriving DecidableEq, Repr

def EffectAllowed : Phase -> Effect -> Prop
  | .prepareFormula, .resolveStatic _ _ => True
  | .prepareFormula, .emitDiagnostic _ _ => True
  | .prepareFormula, .bindLocal _ _ => True
  | .prepareFormula, .enterLambda _ _ => True
  | .lowerDescriptors, .emitDependency _ _ _ => True
  | .lowerDescriptors, .emitDiagnostic _ _ => True
  | .graphBuild, .emitDependency _ _ _ => True
  | .graphBuild, .emitDiagnostic _ _ => True
  | .rebindGate, .resolveDynamic _ _ _ => True
  | .rebindGate, .emitDependency _ _ _ => True
  | .rebindGate, .rejectCandidate _ _ => True
  | .evaluateFormula, .readValue _ _ => True
  | .evaluateFormula, .callFunction _ _ => True
  | .evaluateFormula, .bindLocal _ _ => True
  | .evaluateFormula, .enterLambda _ _ => True
  | .evaluateFormula, .produceCandidate _ _ => True
  | .evaluateFormula, .rejectCandidate _ _ => True
  | .evaluateFormula, .emitDiagnostic _ _ => True
  | .evaluateFormula, .emitFormatDelta _ _ => True
  | .evaluateFormula, .emitDisplayDelta _ _ => True
  | .evaluateFormula, .surfaceRuntimeFact _ _ => True
  | .publication, .publish _ => True
  | .publication, .rejectCandidate _ _ => True
  | .traceProjection, .surfaceRuntimeFact _ _ => True
  | _, _ => False

def IsPublishEffect : Effect -> Prop
  | .publish _ => True
  | _ => False

def IsCandidateEffect : Effect -> Prop
  | .produceCandidate _ _ => True
  | _ => False

def IsRejectEffect : Effect -> Prop
  | .rejectCandidate _ _ => True
  | _ => False

structure SeamRun where
  phase : Phase
  effects : List Effect
  boundaryFacts : List BoundaryFact
  publicationChanged : Bool
  generalOxFuncOpaque : Bool
  phaseLaw : ∀ e, e ∈ effects -> EffectAllowed phase e
  deriving Repr

def NoDirectPublishFromFormula (run : SeamRun) : Prop :=
  run.phase = Phase.evaluateFormula ->
    run.publicationChanged = false ∧ ∀ e, e ∈ run.effects -> ¬ IsPublishEffect e

def RejectPreservesPublication (run : SeamRun) : Prop :=
  (∃ e, e ∈ run.effects ∧ IsRejectEffect e) -> run.publicationChanged = false

def CandidateIsNotPublication (run : SeamRun) : Prop :=
  (∃ e, e ∈ run.effects ∧ IsCandidateEffect e) ->
    run.phase ≠ Phase.publication -> run.publicationChanged = false

def LetLambdaNarrowCarrier (run : SeamRun) : Prop :=
  (BoundaryFact.letBinding ∈ run.boundaryFacts ∨ BoundaryFact.lambdaCarrier ∈ run.boundaryFacts) ->
    run.generalOxFuncOpaque = true

def FormatDisplaySeparated (run : SeamRun) : Prop :=
  BoundaryFact.formatDelta ∈ run.boundaryFacts -> BoundaryFact.displayDelta ∈ run.boundaryFacts ->
    BoundaryFact.formatDelta ≠ BoundaryFact.displayDelta

def HandlerLawModel (run : SeamRun) : Prop :=
  NoDirectPublishFromFormula run ∧
    RejectPreservesPublication run ∧
      CandidateIsNotPublication run ∧
        LetLambdaNarrowCarrier run ∧
          FormatDisplaySeparated run

def SampleFormulaRun : SeamRun :=
  { phase := Phase.evaluateFormula,
    effects := [
      Effect.readValue "A1" "stable_or_prior",
      Effect.bindLocal "x" "A1",
      Effect.enterLambda ["y"] "closure:lambda:1",
      Effect.callFunction "LAMBDA" 1,
      Effect.produceCandidate "node:2" "cand:let-lambda"],
    boundaryFacts := [BoundaryFact.candidateResult, BoundaryFact.letBinding, BoundaryFact.lambdaCarrier],
    publicationChanged := false,
    generalOxFuncOpaque := true,
    phaseLaw := by
      intro e member
      simp at member
      rcases member with rfl | rfl | rfl | rfl | rfl
      all_goals simp [EffectAllowed] }

theorem sample_formula_no_direct_publish :
    NoDirectPublishFromFormula SampleFormulaRun := by
  intro _phaseEq
  constructor
  · rfl
  · intro e member publish
    simp [SampleFormulaRun] at member
    rcases member with rfl | rfl | rfl | rfl | rfl
    all_goals simp [IsPublishEffect] at publish

theorem sample_formula_candidate_not_publication :
    CandidateIsNotPublication SampleFormulaRun := by
  intro _ notPublicationPhase
  exact rfl

theorem sample_formula_let_lambda_narrow :
    LetLambdaNarrowCarrier SampleFormulaRun := by
  intro _
  rfl

def SampleRejectRun : SeamRun :=
  { phase := Phase.evaluateFormula,
    effects := [
      Effect.surfaceRuntimeFact "node:3" "ExecutionRestriction",
      Effect.rejectCandidate "node:3" "typed_host_provider_outcome"],
    boundaryFacts := [BoundaryFact.rejectRecord, BoundaryFact.runtimeEffect],
    publicationChanged := false,
    generalOxFuncOpaque := true,
    phaseLaw := by
      intro e member
      simp at member
      rcases member with rfl | rfl
      all_goals simp [EffectAllowed] }

theorem sample_reject_preserves_publication :
    RejectPreservesPublication SampleRejectRun := by
  intro _
  rfl

def SamplePublicationRun : SeamRun :=
  { phase := Phase.publication,
    effects := [Effect.publish "pub:1"],
    boundaryFacts := [BoundaryFact.commitBundle],
    publicationChanged := true,
    generalOxFuncOpaque := true,
    phaseLaw := by
      intro e member
      simp at member
      rcases member with rfl
      simp [EffectAllowed] }

theorem sample_publication_allows_publish :
    ∃ e, e ∈ SamplePublicationRun.effects ∧ IsPublishEffect e := by
  refine ⟨Effect.publish "pub:1", ?_, ?_⟩
  · simp [SamplePublicationRun]
  · simp [IsPublishEffect]

def SampleFormatDisplayRun : SeamRun :=
  { phase := Phase.evaluateFormula,
    effects := [
      Effect.emitFormatDelta "node:4" "format_delta",
      Effect.emitDisplayDelta "node:4" "display_delta",
      Effect.produceCandidate "node:4" "cand:format"],
    boundaryFacts := [BoundaryFact.candidateResult, BoundaryFact.formatDelta, BoundaryFact.displayDelta],
    publicationChanged := false,
    generalOxFuncOpaque := true,
    phaseLaw := by
      intro e member
      simp at member
      rcases member with rfl | rfl | rfl
      all_goals simp [EffectAllowed] }

theorem sample_format_display_distinct :
    FormatDisplaySeparated SampleFormatDisplayRun := by
  intro _ _
  decide

theorem sample_handler_law_model :
    HandlerLawModel SampleFormulaRun := by
  constructor
  · exact sample_formula_no_direct_publish
  constructor
  · intro rejectWitness
    rcases rejectWitness with ⟨e, member, reject⟩
    simp [SampleFormulaRun] at member
    rcases member with rfl | rfl | rfl | rfl | rfl
    all_goals simp [IsRejectEffect] at reject
  constructor
  · exact sample_formula_candidate_not_publication
  constructor
  · exact sample_formula_let_lambda_narrow
  · intro formatMember _
    simp [SampleFormulaRun] at formatMember

end OxCalc.CoreEngine.W046.OxfmlEffectBoundary
