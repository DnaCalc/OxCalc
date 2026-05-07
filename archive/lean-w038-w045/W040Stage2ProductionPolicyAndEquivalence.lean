import Std

namespace OxCalc.CoreEngine.W040.Stage2ProductionPolicyAndEquivalence

structure Stage2ProductionEvidence where
  boundedPartitionReplay : Bool
  partitionOrderPermutationReplay : Bool
  observableResultInvariance : Bool
  dynamicSoftReferenceReplay : Bool
  snapshotFenceCounterpart : Bool
  capabilityViewFenceCounterpart : Bool
  boundedPartitionAnalyzerModel : Bool
  productionPartitionAnalyzerSoundness : Bool
  fairnessSchedulerCoverage : Bool
  operatedCrossEngineDifferentialService : Bool
  packGradeReplayGovernance : Bool
  deriving DecidableEq, Repr

def HasDeclaredProfileReplay (evidence : Stage2ProductionEvidence) : Bool :=
  evidence.boundedPartitionReplay
    && evidence.partitionOrderPermutationReplay
    && evidence.observableResultInvariance
    && evidence.dynamicSoftReferenceReplay

def HasFenceCounterparts (evidence : Stage2ProductionEvidence) : Bool :=
  evidence.snapshotFenceCounterpart && evidence.capabilityViewFenceCounterpart

def HasBoundedAnalyzerEvidence (evidence : Stage2ProductionEvidence) : Bool :=
  evidence.boundedPartitionAnalyzerModel
    && evidence.observableResultInvariance
    && HasFenceCounterparts evidence

def CanPromoteStage2ProductionPolicy (evidence : Stage2ProductionEvidence) : Bool :=
  HasDeclaredProfileReplay evidence
    && HasFenceCounterparts evidence
    && evidence.boundedPartitionAnalyzerModel
    && evidence.productionPartitionAnalyzerSoundness
    && evidence.fairnessSchedulerCoverage
    && evidence.operatedCrossEngineDifferentialService
    && evidence.packGradeReplayGovernance

def CurrentW040Stage2Evidence : Stage2ProductionEvidence :=
  { boundedPartitionReplay := true,
    partitionOrderPermutationReplay := true,
    observableResultInvariance := true,
    dynamicSoftReferenceReplay := true,
    snapshotFenceCounterpart := true,
    capabilityViewFenceCounterpart := true,
    boundedPartitionAnalyzerModel := true,
    productionPartitionAnalyzerSoundness := false,
    fairnessSchedulerCoverage := false,
    operatedCrossEngineDifferentialService := false,
    packGradeReplayGovernance := false }

theorem currentW040Stage2Evidence_hasDeclaredProfileReplay :
    HasDeclaredProfileReplay CurrentW040Stage2Evidence = true := by
  rfl

theorem currentW040Stage2Evidence_hasFenceCounterparts :
    HasFenceCounterparts CurrentW040Stage2Evidence = true := by
  rfl

theorem currentW040Stage2Evidence_hasBoundedAnalyzerEvidence :
    HasBoundedAnalyzerEvidence CurrentW040Stage2Evidence = true := by
  rfl

theorem currentW040Stage2Evidence_doesNotPromotePolicy :
    CanPromoteStage2ProductionPolicy CurrentW040Stage2Evidence = false := by
  rfl

theorem stage2PromotionRequiresDeclaredProfileReplay
    (evidence : Stage2ProductionEvidence)
    (promoted : CanPromoteStage2ProductionPolicy evidence = true) :
    HasDeclaredProfileReplay evidence = true := by
  cases evidence with
  | mk replay permutation observable dynamic snapshot capability bounded analyzer fairness operated pack =>
      cases replay <;> cases permutation <;> cases observable <;> cases dynamic <;>
        cases snapshot <;> cases capability <;> cases bounded <;>
          cases analyzer <;> cases fairness <;> cases operated <;> cases pack <;>
            simp [
              CanPromoteStage2ProductionPolicy,
              HasDeclaredProfileReplay,
              HasFenceCounterparts
            ] at promoted ⊢

theorem stage2PromotionRequiresFenceCounterparts
    (evidence : Stage2ProductionEvidence)
    (promoted : CanPromoteStage2ProductionPolicy evidence = true) :
    HasFenceCounterparts evidence = true := by
  cases evidence with
  | mk replay permutation observable dynamic snapshot capability bounded analyzer fairness operated pack =>
      cases replay <;> cases permutation <;> cases observable <;> cases dynamic <;>
        cases snapshot <;> cases capability <;> cases bounded <;>
          cases analyzer <;> cases fairness <;> cases operated <;> cases pack <;>
            simp [
              CanPromoteStage2ProductionPolicy,
              HasDeclaredProfileReplay,
              HasFenceCounterparts
            ] at promoted ⊢

theorem stage2PromotionRequiresProductionAnalyzerSoundness
    (evidence : Stage2ProductionEvidence)
    (promoted : CanPromoteStage2ProductionPolicy evidence = true) :
    evidence.productionPartitionAnalyzerSoundness = true := by
  cases evidence with
  | mk replay permutation observable dynamic snapshot capability bounded analyzer fairness operated pack =>
      cases replay <;> cases permutation <;> cases observable <;> cases dynamic <;>
        cases snapshot <;> cases capability <;> cases bounded <;>
          cases analyzer <;> cases fairness <;> cases operated <;> cases pack <;>
            simp [
              CanPromoteStage2ProductionPolicy,
              HasDeclaredProfileReplay,
              HasFenceCounterparts
            ] at promoted ⊢

theorem stage2PromotionRequiresFairnessSchedulerCoverage
    (evidence : Stage2ProductionEvidence)
    (promoted : CanPromoteStage2ProductionPolicy evidence = true) :
    evidence.fairnessSchedulerCoverage = true := by
  cases evidence with
  | mk replay permutation observable dynamic snapshot capability bounded analyzer fairness operated pack =>
      cases replay <;> cases permutation <;> cases observable <;> cases dynamic <;>
        cases snapshot <;> cases capability <;> cases bounded <;>
          cases analyzer <;> cases fairness <;> cases operated <;> cases pack <;>
            simp [
              CanPromoteStage2ProductionPolicy,
              HasDeclaredProfileReplay,
              HasFenceCounterparts
            ] at promoted ⊢

structure W040Stage2PolicySummary where
  policyRows : Nat
  satisfiedPolicyRows : Nat
  exactRemainingBlockerRows : Nat
  snapshotFenceCounterpartEvidenced : Bool
  capabilityViewFenceCounterpartEvidenced : Bool
  boundedPartitionAnalyzerEvidenced : Bool
  stage2PolicyPromoted : Bool
  deriving DecidableEq, Repr

def CurrentW040Stage2PolicySummary : W040Stage2PolicySummary :=
  { policyRows := 12,
    satisfiedPolicyRows := 8,
    exactRemainingBlockerRows := 4,
    snapshotFenceCounterpartEvidenced := true,
    capabilityViewFenceCounterpartEvidenced := true,
    boundedPartitionAnalyzerEvidenced := true,
    stage2PolicyPromoted := false }

theorem currentW040Stage2PolicySummary_hasEightSatisfiedRows :
    CurrentW040Stage2PolicySummary.satisfiedPolicyRows = 8 := by
  rfl

theorem currentW040Stage2PolicySummary_hasFourExactBlockers :
    CurrentW040Stage2PolicySummary.exactRemainingBlockerRows = 4 := by
  rfl

theorem currentW040Stage2PolicySummary_evidencesFenceCounterparts :
    CurrentW040Stage2PolicySummary.snapshotFenceCounterpartEvidenced = true
      /\ CurrentW040Stage2PolicySummary.capabilityViewFenceCounterpartEvidenced = true := by
  constructor <;> rfl

theorem currentW040Stage2PolicySummary_noPolicyPromotion :
    CurrentW040Stage2PolicySummary.stage2PolicyPromoted = false := by
  rfl

end OxCalc.CoreEngine.W040.Stage2ProductionPolicyAndEquivalence
