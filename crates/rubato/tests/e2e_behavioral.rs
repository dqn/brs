#[path = "e2e_support/mod.rs"]
mod e2e_support;

mod e2e_behavioral {
    mod harness_basics;
    mod state_transitions;

    // Phase 5 E2E test suites
    mod audio_integration;
    mod gameplay_autoplay;
    mod gameplay_manual_input;
    mod render_verification;
    mod score_pipeline;
    mod state_lifecycle;

    // Phase 5f
    mod gauge_types;

    // Phase 5b/5d
    mod audio_sequences;
    mod score_handoff;

    // Full gameplay flow
    mod full_flow;

    // Phase 5c/5e/5f
    mod edge_cases;
    mod render_correctness;
    mod timer_animation;

    // Phase 6: Scenario builder
    mod scenario_tests;

    // Phase 8: E2E expansion
    mod concurrency_smoke;
    mod handoff_verification;
    mod transition_paths;
}
