use super::*;

#[test]
fn explicit_environments_map_to_stable_profiles() {
    let cases = [
        (Environment::Quiet, AcousticProfileId::Quiet),
        (Environment::Conversation, AcousticProfileId::Conversation),
        (Environment::Noisy, AcousticProfileId::Noisy),
        (Environment::Online, AcousticProfileId::Online),
        (Environment::LongDistance, AcousticProfileId::LongDistance),
    ];
    for (environment, expected) in cases {
        let selection = select_profile(SelectionInput {
            environment,
            reliability_priority: 50,
            payload_bytes: 100,
            calibration: None,
        })
        .unwrap();
        assert_eq!(selection.primary.id, expected);
        assert_eq!(selection.confidence_percent, 100);
    }
}

#[test]
fn auto_uses_calibration_and_has_fixed_fallback() {
    let calibration = CalibrationMetrics::new(-28.0, 12.0, 0.0, 40).unwrap();
    let selection = select_profile(SelectionInput {
        environment: Environment::Auto,
        reliability_priority: 50,
        payload_bytes: 100,
        calibration: Some(calibration),
    })
    .unwrap();
    assert_eq!(selection.detected_environment, Environment::Conversation);
    assert_eq!(selection.primary.id, AcousticProfileId::Conversation);
    assert_eq!(
        selection.fallback.last(),
        Some(&AcousticProfileId::FixedFallback)
    );
}

#[test]
fn priority_and_payload_size_adjust_auto_choice() {
    let reliable = select_profile(SelectionInput {
        environment: Environment::Quiet,
        reliability_priority: 90,
        payload_bytes: 64,
        calibration: None,
    })
    .unwrap();
    assert_eq!(reliable.primary.id, AcousticProfileId::Balanced);

    let fast = select_profile(SelectionInput {
        environment: Environment::Auto,
        reliability_priority: 10,
        payload_bytes: 1_024,
        calibration: None,
    })
    .unwrap();
    assert_eq!(fast.primary.id, AcousticProfileId::Quiet);
}

#[test]
fn duration_increases_with_payload_and_robustness() {
    let quiet = AcousticProfile::for_id(AcousticProfileId::Quiet);
    let noisy = AcousticProfile::for_id(AcousticProfileId::Noisy);
    assert!(estimate_duration_ms(quiet, 200) > estimate_duration_ms(quiet, 100));
    assert!(estimate_duration_ms(noisy, 100) > estimate_duration_ms(quiet, 100));
    let balanced = AcousticProfile::for_id(AcousticProfileId::Balanced);
    assert!(estimate_payload_duration_ms(noisy, 100) > estimate_payload_duration_ms(balanced, 100));
}

#[test]
fn invalid_calibration_and_priority_are_rejected() {
    assert!(CalibrationMetrics::new(-20.0, 10.0, 1.1, 0).is_err());
    assert!(select_profile(SelectionInput {
        environment: Environment::Auto,
        reliability_priority: 101,
        payload_bytes: 0,
        calibration: None,
    })
    .is_err());
}
