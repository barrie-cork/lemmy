CREATE TYPE case_status AS ENUM (
    'Open',
    'ThresholdMet',
    'JurySelection',
    'InReview',
    'Decided',
    'Appealed',
    'Closed',
    'EmergencyRemove',
    'AdminReview'
);

CREATE TYPE case_target_type AS ENUM (
    'Post',
    'Comment',
    'Person',
    'Community',
    'RemoteInstance'
);

CREATE TYPE case_severity AS ENUM (
    'Low',
    'Medium',
    'High',
    'Critical'
);

CREATE TYPE evidence_visibility AS ENUM (
    'JuryOnly',
    'PrivateAdmin',
    'PublicRedacted'
);

CREATE TYPE jury_assignment_status AS ENUM (
    'Selected',
    'Accepted',
    'Declined',
    'Conflicted',
    'Submitted',
    'Expired'
);

CREATE TYPE jury_decision AS ENUM (
    'NoAction',
    'AdvisoryLabel',
    'Warning',
    'Cooldown',
    'RemoveContent',
    'SuspendLocalUser',
    'SuspendCommunityMember',
    'RecommendFederationAction'
);

CREATE TYPE sanction_scope AS ENUM (
    'Community',
    'Instance',
    'FederatedRecommendation'
);

CREATE TYPE sanction_action AS ENUM (
    'Label',
    'VisibilityReduction',
    'TemporaryRestriction',
    'ContentRemoval',
    'CommunityExclusion',
    'InstanceSuspension',
    'FederationQuarantineRecommendation'
);

CREATE TYPE appeal_status AS ENUM (
    'Requested',
    'Accepted',
    'Rejected',
    'Decided'
);

CREATE TYPE reputation_dimension AS ENUM (
    'ReportingAccuracy',
    'JuryReliability',
    'ParticipationConsistency',
    'EndorsementStrength'
);

CREATE TYPE attestation_type AS ENUM (
    'TrustedReporter',
    'JuryEligible',
    'SanctionNotice',
    'QuarantineRecommendation'
);
