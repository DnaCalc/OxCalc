import Std

namespace OxCalc.CoreEngine.W034.DependenciesOverlays

abbrev NodeId := Nat
abbrev SnapshotEpoch := Nat
abbrev CompatibilityBasis := String

structure DependencyFacts where
  staticDeps : List NodeId
  runtimeDeps : List NodeId
  dynamicShapeUpdateDeps : List NodeId
  deriving DecidableEq, Repr

def DependencyClosure (facts : DependencyFacts) (extra : List NodeId) : List NodeId :=
  facts.staticDeps ++ facts.runtimeDeps ++ facts.dynamicShapeUpdateDeps ++ extra

def ConservativeAffectedSet (required actual : List NodeId) : Prop :=
  forall node, node ∈ required -> node ∈ actual

def NoUnderInvalidation (required actual : List NodeId) : Prop :=
  ConservativeAffectedSet required actual

theorem dependencyClosure_containsStatic
    (facts : DependencyFacts)
    (extra : List NodeId) :
    ConservativeAffectedSet facts.staticDeps (DependencyClosure facts extra) := by
  intro node membership
  unfold DependencyClosure
  simp [membership]

theorem dependencyClosure_containsRuntime
    (facts : DependencyFacts)
    (extra : List NodeId) :
    ConservativeAffectedSet facts.runtimeDeps (DependencyClosure facts extra) := by
  intro node membership
  unfold DependencyClosure
  simp [membership]

theorem dependencyClosure_containsDynamicShapeUpdates
    (facts : DependencyFacts)
    (extra : List NodeId) :
    ConservativeAffectedSet facts.dynamicShapeUpdateDeps (DependencyClosure facts extra) := by
  intro node membership
  unfold DependencyClosure
  simp [membership]

theorem dependencyClosure_noUnderInvalidation
    (facts : DependencyFacts)
    (extra : List NodeId) :
    NoUnderInvalidation
      (facts.staticDeps ++ facts.runtimeDeps ++ facts.dynamicShapeUpdateDeps)
      (DependencyClosure facts extra) := by
  intro node membership
  unfold DependencyClosure
  exact List.mem_append_left extra membership

structure OverlayFact where
  ownerNodeId : NodeId
  snapshotEpoch : SnapshotEpoch
  compatibilityBasis : CompatibilityBasis
  isProtected : Bool
  evictionEligible : Bool
  deriving DecidableEq, Repr

def OverlaySafe (overlays : List OverlayFact) : Prop :=
  forall overlay,
    overlay ∈ overlays ->
      overlay.isProtected = true ->
        overlay.evictionEligible = false

def ProtectedOverlayRetained (before after : List OverlayFact) : Prop :=
  forall overlay,
    overlay ∈ before ->
      overlay.isProtected = true ->
        overlay ∈ after

def RetainProtectedOverlays (overlays : List OverlayFact) : List OverlayFact :=
  overlays.filter (fun overlay => overlay.isProtected)

def MarkEvictionEligibleAfterUnpin (overlay : OverlayFact) : OverlayFact :=
  if overlay.isProtected then
    overlay
  else
    { overlay with evictionEligible := true }

theorem retainedProtectedOverlays_retainsProtected
    (overlays : List OverlayFact) :
    ProtectedOverlayRetained overlays (RetainProtectedOverlays overlays) := by
  intro overlay membership isProtected
  unfold RetainProtectedOverlays
  exact List.mem_filter.mpr ⟨membership, isProtected⟩

theorem protectedOverlay_markEvictionEligibleAfterUnpin_notEligible
    (overlay : OverlayFact)
    (isProtected : overlay.isProtected = true)
    (safeBefore : overlay.evictionEligible = false) :
    (MarkEvictionEligibleAfterUnpin overlay).evictionEligible = false := by
  unfold MarkEvictionEligibleAfterUnpin
  simp [isProtected, safeBefore]

theorem overlaySafe_afterProtectedRetention
    (overlays : List OverlayFact)
    (safe : OverlaySafe overlays) :
    OverlaySafe (RetainProtectedOverlays overlays) := by
  intro overlay retained isProtected
  unfold RetainProtectedOverlays at retained
  have original : overlay ∈ overlays := by
    exact (List.mem_filter.mp retained).left
  exact safe overlay original isProtected

structure W034DependencyOverlayEnvelope where
  staticClosureChecked : Bool
  runtimeClosureChecked : Bool
  dynamicShapeUpdateClosureChecked : Bool
  protectedRetentionChecked : Bool
  protectedEvictionSafetyChecked : Bool
  deriving DecidableEq, Repr

def W034DependencyOverlayProofEnvelope : W034DependencyOverlayEnvelope :=
  { staticClosureChecked := true,
    runtimeClosureChecked := true,
    dynamicShapeUpdateClosureChecked := true,
    protectedRetentionChecked := true,
    protectedEvictionSafetyChecked := true }

theorem w034DependencyOverlayEnvelope_hasDynamicShapeUpdateClosure :
    W034DependencyOverlayProofEnvelope.dynamicShapeUpdateClosureChecked = true := by
  rfl

end OxCalc.CoreEngine.W034.DependenciesOverlays
