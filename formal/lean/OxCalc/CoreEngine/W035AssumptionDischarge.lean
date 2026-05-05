import Std

namespace OxCalc.CoreEngine.W035.AssumptionDischarge

inductive ProofDisposition where
  | provedLocal
  | explicitAxiom
  | externalSeamAssumption
  | opaqueKernelBoundary
  | deferredToLaterLane
  deriving DecidableEq, Repr

structure ObligationRow where
  obligationId : String
  summary : String
  disposition : ProofDisposition
  localEvidenceChecked : Bool
  deriving DecidableEq, Repr

def IsDischargedLocal (row : ObligationRow) : Bool :=
  match row.disposition with
  | ProofDisposition.provedLocal => row.localEvidenceChecked
  | _ => false

def IsExplicitAxiom (row : ObligationRow) : Bool :=
  match row.disposition with
  | ProofDisposition.explicitAxiom => true
  | _ => false

def IsExternalSeamAssumption (row : ObligationRow) : Bool :=
  match row.disposition with
  | ProofDisposition.externalSeamAssumption => true
  | _ => false

def IsOpaqueKernelBoundary (row : ObligationRow) : Bool :=
  match row.disposition with
  | ProofDisposition.opaqueKernelBoundary => true
  | _ => false

def IsDeferredToLaterLane (row : ObligationRow) : Bool :=
  match row.disposition with
  | ProofDisposition.deferredToLaterLane => true
  | _ => false

def SnapshotFenceObligation : ObligationRow :=
  { obligationId := "W035-OBL-007:snapshot-fence",
    summary := "stale snapshot/capability/compatibility fences reject without publish",
    disposition := ProofDisposition.provedLocal,
    localEvidenceChecked := true }

def DependencyClosureObligation : ObligationRow :=
  { obligationId := "W035-OBL-007:dependency-closure",
    summary := "static, runtime, and dynamic-shape dependencies remain in conservative closure",
    disposition := ProofDisposition.provedLocal,
    localEvidenceChecked := true }

def OverlayRetentionObligation : ObligationRow :=
  { obligationId := "W035-OBL-007:overlay-retention",
    summary := "protected overlays are retained and are not marked eviction-eligible",
    disposition := ProofDisposition.provedLocal,
    localEvidenceChecked := true }

def CallableCarrierObligation : ObligationRow :=
  { obligationId := "W035-OBL-004:callable-carrier",
    summary := "OxCalc-visible callable carrier identity is tracked separately from value-only conformance",
    disposition := ProofDisposition.provedLocal,
    localEvidenceChecked := true }

def OxFmlTypedConditionalFormattingObligation : ObligationRow :=
  { obligationId := "W035-OBL-013:oxfml-w073-typed-cf",
    summary := "W073 aggregate and visualization conditional-formatting metadata is typed_rule-only",
    disposition := ProofDisposition.externalSeamAssumption,
    localEvidenceChecked := false }

def FullOxFuncCallableKernelObligation : ObligationRow :=
  { obligationId := "W035-OBL-004:full-oxfunc-callable-kernel",
    summary := "general LET/LAMBDA and callable invocation semantics remain OxFunc-owned kernel truth",
    disposition := ProofDisposition.opaqueKernelBoundary,
    localEvidenceChecked := false }

def MultiReaderOverlayInterleavingObligation : ObligationRow :=
  { obligationId := "W035-OBL-003:multi-reader-overlay-interleaving",
    summary := "non-routine multi-reader overlay release ordering belongs to the TLA lane",
    disposition := ProofDisposition.deferredToLaterLane,
    localEvidenceChecked := false }

theorem snapshotFenceObligation_isDischargedLocal :
    IsDischargedLocal SnapshotFenceObligation = true := by
  rfl

theorem dependencyClosureObligation_isDischargedLocal :
    IsDischargedLocal DependencyClosureObligation = true := by
  rfl

theorem overlayRetentionObligation_isDischargedLocal :
    IsDischargedLocal OverlayRetentionObligation = true := by
  rfl

theorem callableCarrierObligation_isDischargedLocal :
    IsDischargedLocal CallableCarrierObligation = true := by
  rfl

theorem w073TypedConditionalFormatting_isExternalSeamAssumption :
    IsExternalSeamAssumption OxFmlTypedConditionalFormattingObligation = true := by
  rfl

theorem fullOxFuncCallableKernel_isOpaqueBoundary :
    IsOpaqueKernelBoundary FullOxFuncCallableKernelObligation = true := by
  rfl

theorem multiReaderOverlayInterleaving_isDeferred :
    IsDeferredToLaterLane MultiReaderOverlayInterleavingObligation = true := by
  rfl

theorem w073TypedConditionalFormatting_notLocalDischarge :
    IsDischargedLocal OxFmlTypedConditionalFormattingObligation = false := by
  rfl

theorem fullOxFuncCallableKernel_notLocalDischarge :
    IsDischargedLocal FullOxFuncCallableKernelObligation = false := by
  rfl

structure W035TraceCalcOracleMatrixSummary where
  scenarioCount : Nat
  matrixRows : Nat
  coveredRows : Nat
  classifiedUncoveredRows : Nat
  failedRows : Nat
  missingRows : Nat
  deriving DecidableEq, Repr

def TraceCalcOracleMatrixW035 : W035TraceCalcOracleMatrixSummary :=
  { scenarioCount := 30,
    matrixRows := 17,
    coveredRows := 15,
    classifiedUncoveredRows := 2,
    failedRows := 0,
    missingRows := 0 }

def OracleMatrixHasNoFailures (summary : W035TraceCalcOracleMatrixSummary) : Prop :=
  summary.failedRows = 0 /\ summary.missingRows = 0

theorem w035OracleMatrix_hasNoFailures :
    OracleMatrixHasNoFailures TraceCalcOracleMatrixW035 := by
  simp [OracleMatrixHasNoFailures, TraceCalcOracleMatrixW035]

structure W035ImplementationConformanceSummary where
  gapRows : Nat
  implementationWorkDeferrals : Nat
  specEvolutionDeferrals : Nat
  validatedRows : Nat
  failedRows : Nat
  deriving DecidableEq, Repr

def ImplementationConformanceW035 : W035ImplementationConformanceSummary :=
  { gapRows := 6,
    implementationWorkDeferrals := 5,
    specEvolutionDeferrals := 1,
    validatedRows := 6,
    failedRows := 0 }

def ImplementationConformanceHasNoFailures
    (summary : W035ImplementationConformanceSummary) : Prop :=
  summary.failedRows = 0

def ImplementationConformanceHasDeferrals
    (summary : W035ImplementationConformanceSummary) : Prop :=
  summary.implementationWorkDeferrals + summary.specEvolutionDeferrals > 0

theorem w035ImplementationConformance_hasNoFailures :
    ImplementationConformanceHasNoFailures ImplementationConformanceW035 := by
  simp [ImplementationConformanceHasNoFailures, ImplementationConformanceW035]

theorem w035ImplementationConformance_hasDeferrals :
    ImplementationConformanceHasDeferrals ImplementationConformanceW035 := by
  unfold ImplementationConformanceHasDeferrals ImplementationConformanceW035
  decide

structure W035LeanAssumptionSummary where
  localProofRows : Nat
  explicitAxiomRows : Nat
  externalSeamAssumptionRows : Nat
  opaqueKernelBoundaryRows : Nat
  deferredRows : Nat
  fullLeanVerificationPromoted : Bool
  deriving DecidableEq, Repr

def W035LeanAssumptionSummaryValue : W035LeanAssumptionSummary :=
  { localProofRows := 4,
    explicitAxiomRows := 0,
    externalSeamAssumptionRows := 1,
    opaqueKernelBoundaryRows := 1,
    deferredRows := 1,
    fullLeanVerificationPromoted := false }

theorem w035LeanAssumptionSummary_hasNoExplicitAxioms :
    W035LeanAssumptionSummaryValue.explicitAxiomRows = 0 := by
  rfl

theorem w035LeanAssumptionSummary_hasExternalSeamAssumptions :
    W035LeanAssumptionSummaryValue.externalSeamAssumptionRows > 0 := by
  unfold W035LeanAssumptionSummaryValue
  decide

theorem w035LeanAssumptionSummary_noFullLeanPromotion :
    W035LeanAssumptionSummaryValue.fullLeanVerificationPromoted = false := by
  rfl

end OxCalc.CoreEngine.W035.AssumptionDischarge
