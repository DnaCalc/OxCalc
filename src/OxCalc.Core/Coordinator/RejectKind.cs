namespace OxCalc.Core.Coordinator;

public enum RejectKind
{
    SnapshotMismatch = 0,
    ArtifactTokenMismatch = 1,
    ProfileVersionMismatch = 2,
    CapabilityMismatch = 3,
    PublicationFenceMismatch = 4,
    DynamicDependencyFailure = 5,
    SyntheticCycleReject = 6,
    HostInjectedFailure = 7,
}
