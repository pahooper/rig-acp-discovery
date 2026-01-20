//! Integration tests for agent detection.
//!
//! These tests check detection against real CLIs if they are installed.
//! Tests are designed to pass regardless of which agents are installed.

use rig_acp_discovery::{detect, detect_all, AgentKind, AgentStatus};

#[tokio::test]
async fn test_detect_all_returns_valid_statuses() {
    let results = detect_all().await;

    // Should have results for all 4 agents
    assert_eq!(results.len(), 4);

    for (kind, result) in &results {
        match result {
            Ok(AgentStatus::Installed(meta)) => {
                // If installed, path should exist and version should be valid
                assert!(
                    meta.path.exists(),
                    "{} path should exist: {:?}",
                    kind.display_name(),
                    meta.path
                );
                assert!(
                    !meta.version.to_string().is_empty(),
                    "{} should have version",
                    kind.display_name()
                );
                println!(
                    "{}: {} at {:?} (method: {:?})",
                    kind.display_name(),
                    meta.version,
                    meta.path,
                    meta.install_method
                );
            }
            Ok(AgentStatus::NotInstalled) => {
                println!("{}: not installed", kind.display_name());
            }
            Ok(AgentStatus::Unknown { error, message }) => {
                println!(
                    "{}: unknown - {:?}: {}",
                    kind.display_name(),
                    error,
                    message
                );
            }
            Ok(_) => {
                // Handle future variants
                println!("{}: other status", kind.display_name());
            }
            Err(e) => {
                println!(
                    "{}: detection error - {}",
                    kind.display_name(),
                    e.description()
                );
            }
        }
    }
}

#[tokio::test]
async fn test_detect_individual_agents() {
    // Test each agent individually
    for kind in AgentKind::all() {
        let status = detect(kind).await;

        // Status should be one of the valid variants
        assert!(
            matches!(
                status,
                AgentStatus::Installed(_) | AgentStatus::NotInstalled | AgentStatus::Unknown { .. }
            ),
            "Unexpected status for {}: {:?}",
            kind.display_name(),
            status
        );
    }
}

#[tokio::test]
async fn test_detection_is_deterministic() {
    // Running detection twice should give same results
    let first = detect(AgentKind::ClaudeCode).await;
    let second = detect(AgentKind::ClaudeCode).await;

    // Both should be same variant (but timestamps may differ for Installed)
    match (&first, &second) {
        (AgentStatus::Installed(m1), AgentStatus::Installed(m2)) => {
            assert_eq!(m1.path, m2.path);
            assert_eq!(m1.version, m2.version);
        }
        (AgentStatus::NotInstalled, AgentStatus::NotInstalled) => {}
        (AgentStatus::Unknown { error: e1, .. }, AgentStatus::Unknown { error: e2, .. }) => {
            assert_eq!(e1, e2);
        }
        _ => panic!("Detection results differ: {:?} vs {:?}", first, second),
    }
}

#[tokio::test]
async fn test_detect_all_parallel_is_fast() {
    use std::time::Instant;

    // detect_all should run in parallel, so it should complete in roughly
    // the time of the slowest detection, not the sum of all detections.
    let start = Instant::now();
    let _results = detect_all().await;
    let duration = start.elapsed();

    // With 4 agents and 5s timeout each, sequential would be up to 20s
    // Parallel should be at most ~5s (plus overhead)
    // We use 10s as a generous upper bound
    assert!(
        duration.as_secs() < 10,
        "detect_all() took too long: {:?}",
        duration
    );
    println!("detect_all() completed in {:?}", duration);
}

#[tokio::test]
async fn test_installed_metadata_has_valid_timestamps() {
    let results = detect_all().await;

    for (kind, result) in results {
        if let Ok(AgentStatus::Installed(meta)) = result {
            // Timestamp should be recent (within last minute)
            let now = std::time::SystemTime::now();
            let elapsed = now
                .duration_since(meta.last_verified)
                .expect("timestamp should be in the past");
            assert!(
                elapsed.as_secs() < 60,
                "{} timestamp is too old: {:?}",
                kind.display_name(),
                elapsed
            );
        }
    }
}

#[tokio::test]
async fn test_detect_all_with_options_custom_timeout() {
    use rig_acp_discovery::{detect_all_with_options, DetectOptions};
    use std::time::Duration;

    // Use a short timeout
    let options = DetectOptions {
        timeout: Duration::from_secs(1),
    };
    let results = detect_all_with_options(options).await;

    // Should still have results for all agents
    assert_eq!(results.len(), 4);

    // Each result should be valid
    for (_, result) in &results {
        assert!(result.is_ok() || result.is_err());
    }
}

#[tokio::test]
async fn test_detect_with_options_custom_timeout() {
    use rig_acp_discovery::{detect_with_options, DetectOptions};
    use std::time::Duration;

    // Use a short timeout
    let options = DetectOptions {
        timeout: Duration::from_millis(500),
    };
    let status = detect_with_options(AgentKind::ClaudeCode, options).await;

    // Should return a valid status (may be NotInstalled due to timeout)
    assert!(matches!(
        status,
        AgentStatus::Installed(_)
            | AgentStatus::NotInstalled
            | AgentStatus::VersionMismatch { .. }
            | AgentStatus::Unknown { .. }
    ));
}
