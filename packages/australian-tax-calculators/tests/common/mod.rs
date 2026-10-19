//! Fixtures shared by the integration tests.
use australian_tax_calculators::metadata::CoefficientRow;

/// Schedule 8's combined Schedule 1 and component coefficients for scales 1
/// and 2, as published, with the component table each one adds.
pub const COMBINED: [(i64, &str, &[CoefficientRow]); 2] = [
    (
        1,
        "no_threshold",
        &[
            (Some(188), "0.1500", "0.1500"),
            (Some(371), "0.2084", "11.0185"),
            (Some(515), "0.1790", "0.1066"),
            (Some(932), "0.3227", "74.1674"),
            (Some(987), "0.3200", "71.6508"),
            (Some(2144), "0.4700", "219.7124"),
            (Some(2246), "0.4900", "262.6035"),
            (Some(2727), "0.5600", "419.8343"),
            (Some(3303), "0.4900", "228.8816"),
            (None, "0.5700", "493.1893"),
        ],
    ),
    (
        2,
        "threshold_or_foreign",
        &[
            (Some(362), "0", "0"),
            (Some(538), "0.1500", "54.3462"),
            (Some(673), "0.2500", "108.2135"),
            (Some(721), "0.1700", "54.3473"),
            (Some(865), "0.1790", "60.8377"),
            (Some(1282), "0.3227", "185.1935"),
            (Some(1337), "0.3200", "181.7319"),
            (Some(2494), "0.4700", "382.2935"),
            (Some(2596), "0.4900", "432.1846"),
            (Some(3577), "0.5600", "613.9154"),
            (Some(3653), "0.4900", "363.4627"),
            (None, "0.5700", "655.7704"),
        ],
    ),
];
