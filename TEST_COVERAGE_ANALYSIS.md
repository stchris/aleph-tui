# Test Coverage Analysis for aleph-tui

**Analysis Date:** 2026-01-14
**Version:** 0.5.0
**Current Test Count:** 4 tests (3 passing, 1 ignored)

## Executive Summary

The aleph-tui codebase currently has **minimal test coverage** with only 4 unit tests covering approximately **15-20% of the codebase**. Tests focus exclusively on data deserialization (JSON/TOML parsing) while critical application logic, API integration, UI rendering, and event handling remain **completely untested**.

**Critical Risk Areas:**
- No API client testing (network operations, authentication, error handling)
- No state management or navigation logic testing
- No event handling or keyboard input testing
- No configuration loading and validation testing
- No channel-based background fetch testing

---

## Current Test Coverage Breakdown

### ✅ **Currently Tested** (2 modules, 4 tests)

#### 1. **Data Models** (`src/models.rs` - 3 tests)
- ✅ `test_status_deserialization()`: Comprehensive test for Status API response parsing
  - Tests all nested structures (StatusResult, Batch, Queue, Task, Collection, Links)
  - Validates ISO8601 duration parsing
  - Tests optional field handling
  - Uses 343KB realistic fixture data
- ✅ `test_metadata_deserialization()`: Metadata API response parsing
  - Tests app metadata fields (title, version, ftm_version)
- ⚠️ `test_deserialization_no_collection()`: **IGNORED** - Tests None collection handling

#### 2. **Configuration** (`src/app.rs` - 1 test)
- ✅ `test_de_profiles()`: TOML profile deserialization
  - Tests basic profile parsing with default setting
  - **Gap:** Only tests the happy path, missing error cases

### ❌ **Completely Untested** (5 modules, ~900 lines)

| Module | Lines | Functionality | Risk Level |
|--------|-------|--------------|------------|
| `src/app.rs` | ~320 untested | Background fetch, state mgmt, navigation | 🔴 CRITICAL |
| `src/update.rs` | 29 | Keyboard event handling | 🔴 CRITICAL |
| `src/event.rs` | 87 | Terminal event loop | 🟡 HIGH |
| `src/ui.rs` | 243 | TUI rendering | 🟢 MEDIUM |
| `src/tui.rs` | 66 | Terminal lifecycle | 🟢 MEDIUM |
| `src/main.rs` | 69 | CLI argument handling, initialization | 🟡 HIGH |

---

## Priority 1: Critical Gaps (Must Fix)

### 1. **API Client & Network Operations** (`src/app.rs`)
**Risk:** High - Network failures, auth errors, malformed responses can crash the app

**Untested Code:**
- `App::start_fetch()` (lines 196-212): Background fetch initiation
- `App::do_fetch()` (lines 215-267): HTTP requests, authentication, JSON parsing
- `App::poll_fetch_result()` (lines 271-288): Channel-based result handling
- `App::maybe_start_fetch()` (lines 292-297): Automatic fetch timing

**Recommended Tests:**
```rust
#[cfg(test)]
mod api_tests {
    use super::*;
    use tokio::test;

    // Mock HTTP server tests
    #[tokio::test]
    async fn test_successful_fetch() {
        // Mock server returning valid Status + Metadata JSON
    }

    #[tokio::test]
    async fn test_fetch_with_invalid_auth() {
        // Mock server returning 401 Unauthorized
    }

    #[tokio::test]
    async fn test_fetch_with_network_timeout() {
        // Simulate network timeout
    }

    #[tokio::test]
    async fn test_fetch_with_malformed_json() {
        // Mock server returning invalid JSON
    }

    #[tokio::test]
    async fn test_concurrent_fetch_prevention() {
        // Verify only one fetch runs at a time
    }

    #[tokio::test]
    async fn test_fetch_interval_timing() {
        // Verify fetch_interval config is respected
    }

    #[test]
    fn test_poll_fetch_result_empty_channel() {
        // Verify graceful handling when no result ready
    }

    #[test]
    fn test_poll_fetch_result_disconnected_channel() {
        // Verify handling when channel closes unexpectedly
    }
}
```

**Tools:** Use `wiremock` or `mockito` crate for HTTP mocking

### 2. **Keyboard Event Handling** (`src/update.rs`)
**Risk:** Critical - User interaction is the primary interface

**Untested Code:**
- All keyboard mappings (quit, navigation, profile switcher)
- Context-dependent behavior (profile selector vs main view)
- Modifier key handling (Ctrl+C)

**Recommended Tests:**
```rust
#[cfg(test)]
mod update_tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    #[test]
    fn test_quit_on_q() {
        let mut app = create_test_app();
        update(&mut app, KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE));
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_on_escape() {
        let mut app = create_test_app();
        update(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
        assert!(app.should_quit);
    }

    #[test]
    fn test_quit_on_ctrl_c() {
        let mut app = create_test_app();
        update(&mut app, KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL));
        assert!(app.should_quit);
    }

    #[test]
    fn test_toggle_profile_selector() {
        let mut app = create_test_app();
        assert_eq!(app.current_view, CurrentView::Main);

        update(&mut app, KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
        assert_eq!(app.current_view, CurrentView::ProfileSwitcher);

        update(&mut app, KeyEvent::new(KeyCode::Char('p'), KeyModifiers::NONE));
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_navigation_in_main_view() {
        let mut app = create_test_app_with_collections();
        update(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        // Assert collection selection changed
    }

    #[test]
    fn test_navigation_in_profile_selector() {
        let mut app = create_test_app_with_multiple_profiles();
        app.toggle_profile_selector();
        update(&mut app, KeyEvent::new(KeyCode::Down, KeyModifiers::NONE));
        // Assert profile selection changed
    }

    #[test]
    fn test_vim_keys_j_k() {
        // Test 'j' (down) and 'k' (up) navigation
    }

    #[test]
    fn test_enter_closes_profile_selector() {
        let mut app = create_test_app();
        app.toggle_profile_selector();
        update(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
        assert_eq!(app.current_view, CurrentView::Main);
    }

    #[test]
    fn test_enter_in_main_view_no_effect() {
        // Verify Enter key does nothing in main view
    }
}
```

### 3. **State Management & Navigation** (`src/app.rs`)
**Risk:** Medium-High - Incorrect navigation can make data inaccessible

**Untested Code:**
- `collection_up()`/`collection_down()` (lines 356-368)
- `profile_up()`/`profile_down()` (lines 330-354)
- `set_profile()` (lines 314-324)
- `toggle_profile_selector()` (lines 303-308)

**Recommended Tests:**
```rust
#[cfg(test)]
mod navigation_tests {
    use super::*;

    #[test]
    fn test_collection_down_increments_selection() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(0));
        app.collection_down();
        assert_eq!(app.collection_tablestate.selected(), Some(1));
    }

    #[test]
    fn test_collection_down_at_boundary() {
        // Verify can't go past last item
    }

    #[test]
    fn test_collection_up_decrements_selection() {
        let mut app = create_test_app_with_collections();
        app.collection_tablestate.select(Some(2));
        app.collection_up();
        assert_eq!(app.collection_tablestate.selected(), Some(1));
    }

    #[test]
    fn test_collection_up_at_zero() {
        // Verify can't go below 0
    }

    #[test]
    fn test_profile_switching_clears_state() {
        let mut app = create_test_app_with_data();
        app.profile_down();

        // Verify status and metadata are cleared
        assert!(app.status.results.is_empty());
        assert_eq!(app.error_message, "");
    }

    #[test]
    fn test_set_profile_by_name() {
        let mut app = create_test_app();
        app.set_profile("profile2".to_string()).unwrap();
        assert_eq!(app.current_profile().name, "profile2");
    }

    #[test]
    fn test_set_profile_nonexistent() {
        let mut app = create_test_app();
        let result = app.set_profile("nonexistent".to_string());
        assert!(result.is_err());
    }
}
```

---

## Priority 2: High-Impact Gaps

### 4. **Configuration Loading & Validation** (`src/app.rs`)
**Risk:** Medium - Invalid config can prevent startup

**Untested Code:**
- `App::new()` (lines 156-192): Config file reading, parsing, validation
- `Config` custom deserializer (lines 64-123): Complex TOML parsing logic

**Recommended Tests:**
```rust
#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_config_with_multiple_profiles() {
        let toml = r#"
            default = "prod"

            [profiles]
                [profiles.dev]
                url = "http://localhost:8080"
                token = "dev-token"

                [profiles.prod]
                url = "https://prod.example.com"
                token = "prod-token"
        "#;

        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.profiles.len(), 2);
        assert_eq!(cfg.default, "prod");
        assert_eq!(cfg.profiles[0].name, "dev");
        assert_eq!(cfg.profiles[1].name, "prod");
    }

    #[test]
    fn test_config_missing_url() {
        // Verify error when profile missing 'url' field
    }

    #[test]
    fn test_config_missing_token() {
        // Verify error when profile missing 'token' field
    }

    #[test]
    fn test_config_invalid_default() {
        // Verify error when default profile doesn't exist
    }

    #[test]
    fn test_config_custom_fetch_interval() {
        let toml = r#"
            default = "test"
            fetch_interval = 10

            [profiles]
                [profiles.test]
                url = "http://test"
                token = "token"
        "#;

        let cfg: Config = toml::from_str(toml).unwrap();
        assert_eq!(cfg.fetch_interval, 10);
    }

    #[test]
    fn test_config_default_fetch_interval() {
        // Verify default is 5 seconds
    }

    #[test]
    #[should_panic(expected = "Failed to read config file")]
    fn test_app_new_missing_config_file() {
        // Verify graceful error when config file doesn't exist
        // (This may require using a test-specific home directory)
    }
}
```

### 5. **Duration Parsing** (`src/models.rs`)
**Risk:** Medium - Incorrect duration calculations affect UI display

**Untested Code:**
- `duration_serde::deserialize()` (lines 9-42): ISO8601 duration parsing

**Recommended Tests:**
```rust
#[cfg(test)]
mod duration_tests {
    use super::*;

    #[test]
    fn test_parse_seconds_only() {
        let json = r#"{"duration": "PT30S"}"#;
        // Verify 30 seconds parsed correctly
    }

    #[test]
    fn test_parse_minutes_and_seconds() {
        let json = r#"{"duration": "PT5M30S"}"#;
        // Verify 5 minutes 30 seconds = 330 seconds
    }

    #[test]
    fn test_parse_hours_minutes_seconds() {
        let json = r#"{"duration": "PT2H30M15S"}"#;
        // Verify total seconds calculation
    }

    #[test]
    fn test_parse_days() {
        let json = r#"{"duration": "P2DT1H"}"#;
        // Verify day to hour conversion (2 days + 1 hour)
    }

    #[test]
    fn test_parse_fractional_seconds() {
        let json = r#"{"duration": "PT1.5S"}"#;
        // Verify fractional seconds including nanoseconds
    }

    #[test]
    fn test_parse_none_duration() {
        let json = r#"{"duration": null}"#;
        // Verify None is returned for null
    }

    #[test]
    fn test_parse_invalid_duration() {
        let json = r#"{"duration": "invalid"}"#;
        // Verify error for invalid ISO8601 string
    }
}
```

### 6. **Command Line Argument Handling** (`src/main.rs`)
**Risk:** Medium - Incorrect args handling affects user experience

**Untested Code:**
- Profile selection via CLI argument
- `--version` flag
- `--help` flag

**Recommended Tests:**
```rust
#[cfg(test)]
mod cli_tests {
    use super::*;

    #[test]
    fn test_help_flag() {
        // Verify --help prints usage and exits
    }

    #[test]
    fn test_version_flag() {
        // Verify --version prints version and exits
    }

    #[test]
    fn test_profile_argument() {
        // Verify passing profile name as argument
    }

    #[test]
    fn test_invalid_profile_argument() {
        // Verify error when non-existent profile specified
    }

    #[test]
    fn test_no_arguments() {
        // Verify defaults to config default profile
    }
}
```

---

## Priority 3: Medium Priority Gaps

### 7. **Stage/StageOrStages Polymorphic Deserialization**
**Risk:** Low-Medium - Used in models but behavior unclear

**Untested Code:**
- `StageOrStages` enum (lines 96-101): Handles both single Stage and Vec<Stage>

**Recommended Tests:**
```rust
#[test]
fn test_stage_or_stages_single() {
    let json = r#"{"job_id": "1", "stage": "analyze", "finished": 5, "running": 2, "pending": 1}"#;
    let stage: StageOrStages = serde_json::from_str(json).unwrap();
    // Verify it's the Stage variant
}

#[test]
fn test_stage_or_stages_array() {
    let json = r#"[
        {"job_id": "1", "stage": "analyze", "finished": 5, "running": 2, "pending": 1},
        {"job_id": "2", "stage": "ingest", "finished": 3, "running": 1, "pending": 0}
    ]"#;
    let stages: StageOrStages = serde_json::from_str(json).unwrap();
    // Verify it's the Stages(Vec) variant with 2 items
}
```

### 8. **Terminal Event Loop** (`src/event.rs`)
**Risk:** Low - Well-isolated, mostly library code

**Recommended Tests:**
```rust
#[cfg(test)]
mod event_tests {
    use super::*;

    #[tokio::test]
    async fn test_event_handler_receives_key_events() {
        // Test that key presses are received
    }

    #[tokio::test]
    async fn test_event_handler_tick_timing() {
        // Verify tick events fire at expected intervals
    }

    #[test]
    fn test_event_handler_shutdown() {
        // Verify clean shutdown without panics
    }
}
```

---

## Priority 4: Nice-to-Have

### 9. **UI Rendering** (`src/ui.rs`)
**Risk:** Low - Visual issues don't cause crashes, but affect UX

**Challenge:** UI testing is complex and may require snapshot testing or integration tests

**Approach:**
- Focus on **data preparation** rather than actual rendering
- Test layout calculation logic if extracted
- Use integration tests for full rendering (see next section)

### 10. **Terminal Lifecycle** (`src/tui.rs`)
**Risk:** Very Low - Thin wrapper around ratatui library

**Approach:** Consider integration tests rather than unit tests

---

## Testing Infrastructure Recommendations

### 1. **Add Code Coverage Tooling**

Add to `Cargo.toml`:
```toml
[dev-dependencies]
tokio-test = "0.4"
wiremock = "0.6"  # For API mocking
mockito = "1.0"   # Alternative HTTP mocking
tempfile = "3.0"  # For config file testing

[profile.test]
inherits = "dev"
```

**Coverage Tools:**
```bash
# Install cargo-llvm-cov (recommended)
cargo install cargo-llvm-cov

# Generate coverage report
cargo llvm-cov --html --open

# Or use tarpaulin
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

**CI Integration (.github/workflows/build.yml):**
```yaml
- name: Run tests with coverage
  run: cargo llvm-cov --lcov --output-path lcov.info

- name: Upload coverage to Codecov
  uses: codecov/codecov-action@v4
  with:
    files: lcov.info
```

### 2. **Test Helpers & Fixtures**

Create `src/test_helpers.rs`:
```rust
#[cfg(test)]
pub mod test_helpers {
    use super::*;

    /// Create app with minimal valid config for testing
    pub fn create_test_app() -> App {
        let config = Config {
            default: "test".to_string(),
            profiles: vec![
                Profile {
                    index: 0,
                    name: "test".to_string(),
                    url: "http://localhost:8080".to_string(),
                    token: "test-token".to_string(),
                }
            ],
            fetch_interval: 5,
        };

        let (tx, rx) = mpsc::channel(1);

        App {
            status: Status::default(),
            metadata: Metadata::default(),
            config,
            current_profile: 0,
            should_quit: false,
            version: "0.5.0-test".to_string(),
            error_message: String::default(),
            collection_tablestate: TableState::default(),
            current_view: CurrentView::Main,
            profile_tablestate: TableState::default(),
            last_fetch: Local::now(),
            is_fetching: false,
            fetch_result_rx: rx,
            fetch_result_tx: tx,
        }
    }

    /// Create app with multiple profiles
    pub fn create_test_app_with_multiple_profiles() -> App {
        // ...
    }

    /// Create app with mock status data
    pub fn create_test_app_with_collections() -> App {
        let mut app = create_test_app();
        app.status = load_test_status();
        app
    }

    /// Load test status from fixture file
    pub fn load_test_status() -> Status {
        let json = std::fs::read_to_string("testdata/results.json").unwrap();
        serde_json::from_str(&json).unwrap()
    }
}
```

### 3. **Integration Tests**

Create `tests/integration_test.rs`:
```rust
use aleph_tui::*;

#[tokio::test]
async fn test_full_app_lifecycle() {
    // Test: Config loading -> API fetch -> UI render -> Navigation -> Quit
}

#[tokio::test]
async fn test_profile_switching_e2e() {
    // Test: Start with profile1 -> Switch to profile2 -> Verify refetch
}

#[test]
fn test_keyboard_navigation_flow() {
    // Test: Down -> Down -> Up -> Profile Switcher -> Select
}
```

### 4. **Property-Based Testing**

For complex deserialization logic:
```toml
[dev-dependencies]
proptest = "1.0"
```

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_duration_parsing_never_panics(
        hours in 0u32..100,
        minutes in 0u32..60,
        seconds in 0u32..60,
    ) {
        let duration_str = format!("PT{}H{}M{}S", hours, minutes, seconds);
        // Test that parsing never panics regardless of input
    }
}
```

---

## Coverage Targets

| Phase | Target Coverage | Timeline | Priority Areas |
|-------|----------------|----------|----------------|
| **Phase 1** | 40-50% | 1-2 weeks | API client, keyboard handling, navigation |
| **Phase 2** | 60-70% | 2-3 weeks | Config loading, duration parsing, CLI args |
| **Phase 3** | 75-85% | 3-4 weeks | Event loop, edge cases, integration tests |

---

## Specific Action Items

### Immediate (This Week)
1. ✅ Set up `cargo-llvm-cov` for coverage reporting
2. ✅ Create `test_helpers.rs` with common fixtures
3. ✅ Write API client tests with HTTP mocking
4. ✅ Write keyboard event handling tests
5. ✅ Enable ignored test in `models.rs` or fix/remove it

### Short Term (Next 2 Weeks)
6. ✅ Write navigation logic tests (up/down movements)
7. ✅ Write configuration parsing tests (edge cases)
8. ✅ Add duration parsing edge case tests
9. ✅ Write CLI argument handling tests
10. ✅ Set up CI coverage reporting (Codecov)

### Medium Term (Next Month)
11. ✅ Write integration tests for full user flows
12. ✅ Add property-based tests for deserializers
13. ✅ Achieve 60%+ code coverage
14. ✅ Document testing patterns in CONTRIBUTING.md

---

## Notes on Testing Challenges

### 1. **Testing App::new()**
Challenge: Reads from `~/.config/aleph-tui.toml`
Solution:
- Refactor to accept config path as parameter
- Use `tempfile` crate for test configs
- Or keep as integration test only

### 2. **Testing Async Fetch Operations**
Challenge: Real network requests are slow and unreliable
Solution:
- Use `wiremock` or `mockito` for HTTP mocking
- Consider extracting HTTP client into trait for dependency injection

### 3. **Testing TUI Rendering**
Challenge: Terminal output is hard to assert on
Solution:
- Focus on data preparation logic, not rendering
- Use snapshot testing for full frames if needed
- Ratatui provides TestBackend for testing

### 4. **The Ignored Test**
The test `test_deserialization_no_collection()` is currently ignored. Actions:
- Investigate why it's ignored (does it fail? is testdata/export.json incomplete?)
- Either fix and enable it, or remove if no longer relevant

---

## Conclusion

The aleph-tui project would significantly benefit from expanded test coverage, particularly in the areas of:

1. **API integration and error handling** (highest risk)
2. **User input and navigation** (highest user impact)
3. **Configuration loading and validation** (startup reliability)

By implementing the Priority 1 and Priority 2 tests above, you would:
- Increase coverage from ~15% to **60-70%**
- Protect against regression in critical user flows
- Enable confident refactoring
- Catch edge cases before they reach production

**Recommended First Steps:**
1. Set up coverage tooling
2. Implement API client tests with mocking
3. Implement keyboard event handler tests
4. Create test helper utilities

This foundation will make all subsequent testing easier and more maintainable.
