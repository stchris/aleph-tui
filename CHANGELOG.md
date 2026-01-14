# Changelog

All notable changes to this project will be documented in this file.
## [0.5.0] - 2026-01-14

### Added

- Openaleph support
- Support Openaleph tasks in detail view
- Switch to a flex layout so the info_box expands automatically

### Fixed

- Update rust crate reqwest to 0.13
- Update rust crate ratatui to 0.30
- Prevent UI blocking during data fetch

### Other

- Remove human_panic
- Merge pull request #51 from stchris/renovate/serde_json-1.x-lockfile
- Merge pull request #54 from stchris/renovate/reqwest-0.x
- Merge pull request #47 from stchris/renovate/reqwest-0.x-lockfile
- Merge pull request #52 from stchris/renovate/serde_json-1.x-lockfile
- Merge pull request #49 from stchris/renovate/home-0.x-lockfile
- Enable automerge
- Update cargo-dist
- Merge pull request #48 from stchris/renovate/tokio-1.x-lockfile
- Merge pull request #53 from stchris/renovate/ratatui-0.x
- Merge pull request #58 from stchris/claude/fix-ui-redraw-blocking-BnL8E

## [0.4.3] - 2025-09-16

### Other

- Update CHANGELOG.md
- Merge pull request #43 from stchris/renovate/major-github-artifact-actions

## [0.4.2] - 2025-09-16

### Fixed

- Update rust crate human-panic to v2.0.2
- Update rust crate reqwest to v0.12.9
- Update rust crate serde to v1.0.215
- Update rust crate serde_json to v1.0.133
- Update rust crate tokio to v1.41.1
- Update rust crate human-panic to v2.0.3

### Other

- Add renovate.json
- Merge pull request #10 from stchris/renovate/configure
- Merge pull request #11 from stchris/renovate/human-panic-2.x-lockfile
- Merge pull request #12 from stchris/renovate/reqwest-0.x-lockfile
- Merge pull request #14 from stchris/renovate/serde-monorepo
- Merge pull request #15 from stchris/renovate/serde_json-1.x-lockfile
- Merge pull request #18 from stchris/renovate/actions-checkout-4.x
- Merge pull request #17 from stchris/renovate/tokio-1.x-lockfile
- Cargo update
- Merge pull request #35 from stchris/renovate/major-github-artifact-actions
- Better use of format strings
- Merge pull request #33 from stchris/renovate/human-panic-2.x-lockfile
- Cargo update
- Hump de(pendency) bump
- Update deprecated ratatui methods
- Handle missing config file gracefully instead of panic
- Merge pull request #42 from jlstro/main

## [0.4.1] - 2024-10-24

### Added

- Show status details per collection when browsing results
- Integrate human_panic for much nicer panic handling
- Make it async

### Other

- Nicer display of time deltas
- Updated changelog
- Make the info panel slightly larger
- Merge pull request #9 from stchris/bugfix/larger-info-pane

## [0.3.2] - 2024-09-04

### Other

- Update cargo-dist

## [0.3.1] - 2024-09-04

### Other

- Add support for 4.0.0 status API
- Bump deps: ratatui, crossterm
- Update cargo-dist
- Update changelog

## [0.3.0] - 2024-03-04

### Added

- Format large numbers
- Send user agent header including version
- Accept a profile name as argument

### Other

- Show aleph and followthemoney versions
- Show aleph and ftm versions
- Clear state when switching profiles
- Merge pull request #4 from stchris/feature/show-versions
- Add an icon to show that a fetch is going on
- Create devcontainer.json with rust
- Bump ratatui to 0.26.0
- Merge branch 'main' into feature/number-format
- Merge pull request #6 from stchris/feature/number-format
- Bump version. oops.
- Clear error messages between profile selections
- Merge branch 'bugfix/clear-err'
- Implement custom arg parser so we don't drag in clap (yet)
- Merge branch 'feature/profile-arg'
- Fix help text alignment
- Remove some questionable styling

## [0.1.3] - 2024-01-24

### Other

- Mark collection.{xref,restricted} as optional
- Added latest change to Changelog
- Use git-cliff for generating the CHANGELOG

## [0.1.2] - 2024-01-20

### Other

- No linux-musl targets for now

## [0.1.1] - 2024-01-20

### Other

- Add shell and homebrew installers
- Merge pull request #2 from stchris/feature/cargo-dist-shell-brew

## [0.1.0] - 2024-01-19

### Other

- Initial commit
- Initial commit
- Add build/lint/test workflow to PRs
- Merge pull request #1 from stchris/feature/pr-checks


