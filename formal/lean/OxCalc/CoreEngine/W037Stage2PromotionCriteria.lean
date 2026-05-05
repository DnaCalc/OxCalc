import Std

namespace OxCalc.CoreEngine.W037.Stage2PromotionCriteria

structure Stage2PromotionEvidence where
  concretePartitionModel : Bool
  schedulerCriteria : Bool
  deterministicPartitionReplay : Bool
  observableResultInvariance : Bool
  crossPartitionDependencyProof : Bool
  operatedDifferentialService : Bool
  packGradeReplayGovernance : Bool
  deriving DecidableEq, Repr

def CanPromoteStage2Policy (evidence : Stage2PromotionEvidence) : Bool :=
  evidence.concretePartitionModel
    && evidence.schedulerCriteria
    && evidence.deterministicPartitionReplay
    && evidence.observableResultInvariance
    && evidence.crossPartitionDependencyProof
    && evidence.operatedDifferentialService
    && evidence.packGradeReplayGovernance

def HasReplayAndObservableEquivalence (evidence : Stage2PromotionEvidence) : Bool :=
  evidence.deterministicPartitionReplay && evidence.observableResultInvariance

def HasBoundedModelOnly (evidence : Stage2PromotionEvidence) : Bool :=
  evidence.concretePartitionModel
    && evidence.schedulerCriteria
    && !evidence.deterministicPartitionReplay

def CurrentW037Stage2Evidence : Stage2PromotionEvidence :=
  { concretePartitionModel := true,
    schedulerCriteria := true,
    deterministicPartitionReplay := false,
    observableResultInvariance := true,
    crossPartitionDependencyProof := false,
    operatedDifferentialService := false,
    packGradeReplayGovernance := false }

theorem currentW037Stage2Evidence_hasBoundedModelOnly :
    HasBoundedModelOnly CurrentW037Stage2Evidence = true := by
  rfl

theorem currentW037Stage2Evidence_hasNoReplayEquivalence :
    HasReplayAndObservableEquivalence CurrentW037Stage2Evidence = false := by
  rfl

theorem currentW037Stage2Evidence_doesNotPromotePolicy :
    CanPromoteStage2Policy CurrentW037Stage2Evidence = false := by
  rfl

theorem stage2PromotionRequiresReplayAndObservableEquivalence
    (evidence : Stage2PromotionEvidence)
    (promoted : CanPromoteStage2Policy evidence = true) :
    HasReplayAndObservableEquivalence evidence = true := by
  cases evidence with
  | mk concrete scheduler replay observable crossPartition operated pack =>
      cases concrete <;> cases scheduler <;> cases replay <;> cases observable <;>
        cases crossPartition <;> cases operated <;> cases pack <;>
          simp [CanPromoteStage2Policy, HasReplayAndObservableEquivalence] at promoted ⊢

structure Stage2CriteriaSummary where
  criteriaRows : Nat
  satisfiedCriteriaRows : Nat
  blockedCriteriaRows : Nat
  stage2PolicyPromoted : Bool
  stage2PromotionCandidate : Bool
  deriving DecidableEq, Repr

def CurrentW037Stage2CriteriaSummary : Stage2CriteriaSummary :=
  { criteriaRows := 7,
    satisfiedCriteriaRows := 3,
    blockedCriteriaRows := 4,
    stage2PolicyPromoted := false,
    stage2PromotionCandidate := false }

theorem currentW037Stage2Criteria_noPolicyPromotion :
    CurrentW037Stage2CriteriaSummary.stage2PolicyPromoted = false := by
  rfl

theorem currentW037Stage2Criteria_notPromotionCandidate :
    CurrentW037Stage2CriteriaSummary.stage2PromotionCandidate = false := by
  rfl

theorem currentW037Stage2Criteria_hasFourBlockedRows :
    CurrentW037Stage2CriteriaSummary.blockedCriteriaRows = 4 := by
  rfl

end OxCalc.CoreEngine.W037.Stage2PromotionCriteria
