import Std

namespace OxCalc.CoreEngine.W036.CallableBoundaryInventory

inductive CallableOwner where
  | oxcalc
  | oxfml
  | oxfunc
  | conformanceLane
  deriving DecidableEq, Repr

inductive CallableBoundaryStatus where
  | carrierFragmentProvedLocal
  | metadataProjectionDeferred
  | externalSeamAssumption
  | opaqueKernelBoundary
  deriving DecidableEq, Repr

structure CallableBoundaryRow where
  rowId : String
  obligationId : String
  owner : CallableOwner
  status : CallableBoundaryStatus
  checkedLocalCarrierProof : Bool
  blockerBead : Option String
  deriving DecidableEq, Repr

def CanUseAsOxCalcCallableProof (row : CallableBoundaryRow) : Bool :=
  match row.owner, row.status with
  | CallableOwner.oxcalc, CallableBoundaryStatus.carrierFragmentProvedLocal =>
      row.checkedLocalCarrierProof
  | _, _ => false

def RequiresOpaqueOrExternalAuthority (row : CallableBoundaryRow) : Bool :=
  match row.status with
  | CallableBoundaryStatus.externalSeamAssumption => true
  | CallableBoundaryStatus.opaqueKernelBoundary => true
  | _ => false

def RequiresConformanceFollowup (row : CallableBoundaryRow) : Bool :=
  match row.owner, row.status with
  | CallableOwner.conformanceLane, CallableBoundaryStatus.metadataProjectionDeferred => true
  | _, _ => false

def HasBlockerBead (row : CallableBoundaryRow) : Bool :=
  row.blockerBead.isSome

def DirectCallableCarrierRow : CallableBoundaryRow :=
  { rowId := "w036.callable.direct-carrier",
    obligationId := "W036-OBL-008",
    owner := CallableOwner.oxcalc,
    status := CallableBoundaryStatus.carrierFragmentProvedLocal,
    checkedLocalCarrierProof := true,
    blockerBead := none }

def RuntimeEffectVisibilityRow : CallableBoundaryRow :=
  { rowId := "w036.callable.runtime-effect-visibility",
    obligationId := "W036-OBL-004",
    owner := CallableOwner.oxcalc,
    status := CallableBoundaryStatus.carrierFragmentProvedLocal,
    checkedLocalCarrierProof := true,
    blockerBead := none }

def CallableMetadataProjectionRow : CallableBoundaryRow :=
  { rowId := "w036.callable.metadata-projection",
    obligationId := "W036-OBL-008",
    owner := CallableOwner.conformanceLane,
    status := CallableBoundaryStatus.metadataProjectionDeferred,
    checkedLocalCarrierProof := false,
    blockerBead := some "calc-rqq.4" }

def HostSensitiveLambdaEffectRow : CallableBoundaryRow :=
  { rowId := "w036.callable.host-sensitive-lambda-effect",
    obligationId := "W036-OBL-004",
    owner := CallableOwner.oxfunc,
    status := CallableBoundaryStatus.opaqueKernelBoundary,
    checkedLocalCarrierProof := false,
    blockerBead := some "calc-rqq.4" }

def FullOxFuncLambdaKernelRow : CallableBoundaryRow :=
  { rowId := "w036.callable.full-oxfunc-lambda-kernel",
    obligationId := "W036-OBL-004",
    owner := CallableOwner.oxfunc,
    status := CallableBoundaryStatus.opaqueKernelBoundary,
    checkedLocalCarrierProof := false,
    blockerBead := some "calc-rqq.4" }

def OxFmlCallableCarrierSeamRow : CallableBoundaryRow :=
  { rowId := "w036.callable.oxfml-carrier-seam",
    obligationId := "W036-OBL-017",
    owner := CallableOwner.oxfml,
    status := CallableBoundaryStatus.externalSeamAssumption,
    checkedLocalCarrierProof := false,
    blockerBead := none }

theorem directCallableCarrier_promotableAsCarrierProof :
    CanUseAsOxCalcCallableProof DirectCallableCarrierRow = true := by
  rfl

theorem runtimeEffectVisibility_promotableAsCarrierProof :
    CanUseAsOxCalcCallableProof RuntimeEffectVisibilityRow = true := by
  rfl

theorem callableMetadataProjection_notPromotableAsCarrierProof :
    CanUseAsOxCalcCallableProof CallableMetadataProjectionRow = false := by
  rfl

theorem callableMetadataProjection_requiresConformanceFollowup :
    RequiresConformanceFollowup CallableMetadataProjectionRow = true := by
  rfl

theorem callableMetadataProjection_hasBlocker :
    HasBlockerBead CallableMetadataProjectionRow = true := by
  rfl

theorem hostSensitiveLambdaEffect_opaqueOrExternal :
    RequiresOpaqueOrExternalAuthority HostSensitiveLambdaEffectRow = true := by
  rfl

theorem fullOxFuncLambdaKernel_opaqueOrExternal :
    RequiresOpaqueOrExternalAuthority FullOxFuncLambdaKernelRow = true := by
  rfl

theorem fullOxFuncLambdaKernel_notPromotableAsCarrierProof :
    CanUseAsOxCalcCallableProof FullOxFuncLambdaKernelRow = false := by
  rfl

theorem oxfmlCallableCarrierSeam_requiresExternalAuthority :
    RequiresOpaqueOrExternalAuthority OxFmlCallableCarrierSeamRow = true := by
  rfl

def CarrierFragmentSeparatedFromKernel
    (carrier kernel : CallableBoundaryRow) : Bool :=
  CanUseAsOxCalcCallableProof carrier
    && RequiresOpaqueOrExternalAuthority kernel
    && !CanUseAsOxCalcCallableProof kernel

theorem directCarrier_isSeparatedFromFullOxFuncKernel :
    CarrierFragmentSeparatedFromKernel
      DirectCallableCarrierRow
      FullOxFuncLambdaKernelRow = true := by
  rfl

structure W036CallableBoundarySummary where
  localCarrierProofRows : Nat
  metadataProjectionDeferredRows : Nat
  externalSeamRows : Nat
  opaqueKernelRows : Nat
  explicitAxiomRows : Nat
  fullOxFuncKernelPromoted : Bool
  fullCallableMetadataProjectionPromoted : Bool
  deriving DecidableEq, Repr

def W036CallableBoundarySummaryValue : W036CallableBoundarySummary :=
  { localCarrierProofRows := 2,
    metadataProjectionDeferredRows := 1,
    externalSeamRows := 1,
    opaqueKernelRows := 2,
    explicitAxiomRows := 0,
    fullOxFuncKernelPromoted := false,
    fullCallableMetadataProjectionPromoted := false }

theorem w036CallableBoundary_hasNoExplicitAxioms :
    W036CallableBoundarySummaryValue.explicitAxiomRows = 0 := by
  rfl

theorem w036CallableBoundary_doesNotPromoteFullOxFuncKernel :
    W036CallableBoundarySummaryValue.fullOxFuncKernelPromoted = false := by
  rfl

theorem w036CallableBoundary_doesNotPromoteMetadataProjection :
    W036CallableBoundarySummaryValue.fullCallableMetadataProjectionPromoted = false := by
  rfl

end OxCalc.CoreEngine.W036.CallableBoundaryInventory
