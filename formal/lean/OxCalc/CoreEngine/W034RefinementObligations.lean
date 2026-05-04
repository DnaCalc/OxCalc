import Std

namespace OxCalc.CoreEngine.W034.RefinementObligations

inductive ComparisonState where
  | matchedExactValueSurface
  | matchedNoPublicationSurface
  | matchedLifecycleSurface
  | declaredLocalGap
  | missingArtifact
  | unexpectedMismatch
  deriving DecidableEq, Repr

def IsConformanceMatch (state : ComparisonState) : Bool :=
  match state with
  | ComparisonState.matchedExactValueSurface => true
  | ComparisonState.matchedNoPublicationSurface => true
  | ComparisonState.matchedLifecycleSurface => true
  | ComparisonState.declaredLocalGap => false
  | ComparisonState.missingArtifact => false
  | ComparisonState.unexpectedMismatch => false

def RequiresHandoffOrFix (state : ComparisonState) : Bool :=
  match state with
  | ComparisonState.unexpectedMismatch => true
  | ComparisonState.missingArtifact => true
  | _ => false

structure W034ComparisonSummary where
  exactValueMatches : Nat
  noPublicationMatches : Nat
  lifecycleMatches : Nat
  declaredLocalGaps : Nat
  missingArtifacts : Nat
  unexpectedMismatches : Nat
  deriving DecidableEq, Repr

def NoUnexpectedMismatch (summary : W034ComparisonSummary) : Prop :=
  summary.missingArtifacts = 0 /\ summary.unexpectedMismatches = 0

def HasDeclaredLocalGap (summary : W034ComparisonSummary) : Prop :=
  summary.declaredLocalGaps > 0

def FullIndependentEvaluatorDiversityPromoted (summary : W034ComparisonSummary) : Prop :=
  summary.declaredLocalGaps = 0
    /\ summary.missingArtifacts = 0
    /\ summary.unexpectedMismatches = 0

def W034IndependentConformanceSummary : W034ComparisonSummary :=
  { exactValueMatches := 5,
    noPublicationMatches := 3,
    lifecycleMatches := 1,
    declaredLocalGaps := 6,
    missingArtifacts := 0,
    unexpectedMismatches := 0 }

theorem declaredLocalGap_isNotConformanceMatch :
    IsConformanceMatch ComparisonState.declaredLocalGap = false := by
  rfl

theorem unexpectedMismatch_requiresHandoffOrFix :
    RequiresHandoffOrFix ComparisonState.unexpectedMismatch = true := by
  rfl

theorem missingArtifact_requiresHandoffOrFix :
    RequiresHandoffOrFix ComparisonState.missingArtifact = true := by
  rfl

theorem w034IndependentConformance_hasNoUnexpectedMismatch :
    NoUnexpectedMismatch W034IndependentConformanceSummary := by
  simp [NoUnexpectedMismatch, W034IndependentConformanceSummary]

theorem w034IndependentConformance_hasDeclaredLocalGap :
    HasDeclaredLocalGap W034IndependentConformanceSummary := by
  unfold HasDeclaredLocalGap W034IndependentConformanceSummary
  decide

theorem w034IndependentConformance_doesNotPromoteFullIndependentEvaluatorDiversity :
    ¬ FullIndependentEvaluatorDiversityPromoted W034IndependentConformanceSummary := by
  intro promoted
  have noGaps : 6 = 0 := by
    simpa [W034IndependentConformanceSummary] using promoted.left
  cases noGaps

structure W034LeanProofEnvelope where
  publicationFenceProofsChecked : Bool
  dependencyOverlayProofsChecked : Bool
  letLambdaReplayProofsChecked : Bool
  refinementClassificationChecked : Bool
  stage2ContentionPromoted : Bool
  deriving DecidableEq, Repr

def W034LeanProofEnvelopeValue : W034LeanProofEnvelope :=
  { publicationFenceProofsChecked := true,
    dependencyOverlayProofsChecked := true,
    letLambdaReplayProofsChecked := true,
    refinementClassificationChecked := true,
    stage2ContentionPromoted := false }

theorem w034LeanProofEnvelope_noStage2Promotion :
    W034LeanProofEnvelopeValue.stage2ContentionPromoted = false := by
  rfl

end OxCalc.CoreEngine.W034.RefinementObligations
