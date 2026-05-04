import Std

namespace OxCalc.CoreEngine.W034.LetLambdaReplay

abbrev NodeId := Nat
abbrev ValueText := String
abbrev CallableId := String
abbrev InvocationContractRef := String

inductive CallableOriginKind where
  | inlineLambda
  | helperBoundLambda
  | adoptedDefinedName
  | returnedLambda
  | opaqueExternal
  deriving DecidableEq, Repr

inductive CaptureMode where
  | noCapture
  | lexicalExact
  | lexicalOpaque
  deriving DecidableEq, Repr

structure CallableSurface where
  callableId : CallableId
  originKind : CallableOriginKind
  captureMode : CaptureMode
  arityShape : Nat
  invocationContractRef : InvocationContractRef
  dependencyVisible : Bool
  runtimeEffectVisible : Bool
  publishedValue : ValueText
  deriving DecidableEq, Repr

def RequiresRuntimeVisibility (surface : CallableSurface) : Prop :=
  surface.dependencyVisible = true \/ surface.runtimeEffectVisible = true

def HasInvocationContract (surface : CallableSurface) : Prop :=
  surface.invocationContractRef != ""

def FullCallableSurfaceRefines (trace observed : CallableSurface) : Prop :=
  observed.publishedValue = trace.publishedValue
    /\ observed.callableId = trace.callableId
    /\ observed.originKind = trace.originKind
    /\ observed.captureMode = trace.captureMode
    /\ observed.arityShape = trace.arityShape
    /\ observed.invocationContractRef = trace.invocationContractRef
    /\ observed.dependencyVisible = trace.dependencyVisible
    /\ observed.runtimeEffectVisible = trace.runtimeEffectVisible

def ValueSurfaceRefines (trace observed : CallableSurface) : Prop :=
  observed.publishedValue = trace.publishedValue

theorem fullCallableSurface_impliesValueSurface
    (trace observed : CallableSurface)
    (full : FullCallableSurfaceRefines trace observed) :
    ValueSurfaceRefines trace observed := by
  exact full.left

theorem fullCallableSurface_impliesCallableIdentity
    (trace observed : CallableSurface)
    (full : FullCallableSurfaceRefines trace observed) :
    observed.callableId = trace.callableId := by
  exact full.right.left

def TraceHigherOrderSurface : CallableSurface :=
  { callableId := "callable:make_adder:return",
    originKind := CallableOriginKind.returnedLambda,
    captureMode := CaptureMode.lexicalExact,
    arityShape := 1,
    invocationContractRef := "PACK.lambda.invocation_contract",
    dependencyVisible := true,
    runtimeEffectVisible := false,
    publishedValue := "17" }

def TreeCalcValueOnlySurface : CallableSurface :=
  { callableId := "",
    originKind := CallableOriginKind.opaqueExternal,
    captureMode := CaptureMode.lexicalOpaque,
    arityShape := 1,
    invocationContractRef := "",
    dependencyVisible := true,
    runtimeEffectVisible := false,
    publishedValue := "17" }

theorem w034HigherOrder_valueOnlyDoesNotProveFullCallableSurface :
    ValueSurfaceRefines TraceHigherOrderSurface TreeCalcValueOnlySurface
      /\ ¬ FullCallableSurfaceRefines TraceHigherOrderSurface TreeCalcValueOnlySurface := by
  constructor
  · rfl
  · intro full
    have identityEq :
        TreeCalcValueOnlySurface.callableId = TraceHigherOrderSurface.callableId :=
      fullCallableSurface_impliesCallableIdentity
        TraceHigherOrderSurface
        TreeCalcValueOnlySurface
        full
    have identityMismatch :
        TreeCalcValueOnlySurface.callableId ≠ TraceHigherOrderSurface.callableId := by
      decide
    exact identityMismatch identityEq

structure ValueObservation where
  nodeId : NodeId
  value : ValueText
  deriving DecidableEq, Repr

def EmptyView : NodeId -> Option ValueText :=
  fun _ => none

def ApplyObservation
    (view : NodeId -> Option ValueText)
    (observation : ValueObservation) :
    NodeId -> Option ValueText :=
  fun node =>
    if node = observation.nodeId then
      some observation.value
    else
      view node

def ApplyTwo (left top : ValueObservation) : NodeId -> Option ValueText :=
  ApplyObservation (ApplyObservation EmptyView left) top

def ApplyThree
    (left top check : ValueObservation) :
    NodeId -> Option ValueText :=
  ApplyObservation (ApplyTwo left top) check

theorem independentTwoWrites_commute
    (left top : ValueObservation)
    (distinct : left.nodeId ≠ top.nodeId) :
    ApplyTwo left top = ApplyTwo top left := by
  funext node
  unfold ApplyTwo ApplyObservation EmptyView
  by_cases hTop : node = top.nodeId
  · have hTopNotLeft : ¬ top.nodeId = left.nodeId := by
      intro h
      exact distinct h.symm
    simp [hTop, hTopNotLeft]
  · by_cases hLeft : node = left.nodeId
    · have hLeftNotTop : ¬ left.nodeId = top.nodeId := distinct
      simp [hLeft, hLeftNotTop]
    · simp [hTop, hLeft]

theorem independentOrderBeforeCheck_samePublishedView
    (left top check : ValueObservation)
    (distinct : left.nodeId ≠ top.nodeId) :
    ApplyThree left top check = ApplyThree top left check := by
  unfold ApplyThree
  rw [independentTwoWrites_commute left top distinct]

def W034LeftObservation : ValueObservation :=
  { nodeId := 4, value := "4" }

def W034TopObservation : ValueObservation :=
  { nodeId := 5, value := "5" }

def W034CheckObservation : ValueObservation :=
  { nodeId := 6, value := "9" }

theorem w034IndependentOrder_projectionEquivalent :
    ApplyThree W034LeftObservation W034TopObservation W034CheckObservation
      = ApplyThree W034TopObservation W034LeftObservation W034CheckObservation := by
  apply independentOrderBeforeCheck_samePublishedView
  decide

end OxCalc.CoreEngine.W034.LetLambdaReplay
