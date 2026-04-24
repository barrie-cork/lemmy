CREATE TYPE severity_tier AS ENUM (
    'Minor',
    'Moderate',
    'Severe'
);

CREATE TYPE case_status_tier AS ENUM (
    'Founder',
    'Regular',
    'Probation'
);

CREATE TYPE jury_assignment_role AS ENUM (
    'Original',
    'Appeal'
);
