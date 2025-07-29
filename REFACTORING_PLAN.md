# 3-Crate Architecture Refactoring Plan

## Overview
Refactor `sakurs-core` into three separate crates (core, engine, api) while maintaining backward compatibility for `sakurs-py` and `sakurs-cli`.

## Phases

### Phase 1: Extract Core Domain Layer (sakurs-delta-core)
1. Create new `sakurs-delta-core/` crate with no_std support
2. Move pure domain types:
   - `types::Boundary`, `BoundaryKind`, `Class`
   - `error::CoreError` (deterministic errors only)
   - `LanguageRules` trait (simplified)
   - `DeltaVec`, `PartialState`
3. Implement new `DeltaScanner` with closure-based emission
4. Create utility functions: `scan_chunk`, `reduce_deltas`, `run`
5. Remove all std/external dependencies

### Phase 2: Create Engine Application Layer (sakurs-engine)
1. Create new `sakurs-engine/` crate
2. Implement `Executor` trait and strategies:
   - `SequentialExecutor`
   - `ParallelExecutor`
   - `StreamingExecutor`
3. Move and refactor:
   - `ChunkManager` and related types
   - `prefix_sum` implementation
   - `EngineConfig` and `ExecMode`
4. Create `SentenceProcessor` and builder
5. Implement `auto_select` logic

### Phase 3: Restructure API Layer (sakurs-api)
1. Create new `sakurs-api/` crate
2. Move DTOs:
   - `Input` enum
   - `Output`, `BoundaryDTO`, `Metadata`
3. Create high-level `ConfigBuilder`
4. Re-export `SentenceProcessor` from engine
5. Implement convenience functions
6. Map errors appropriately

### Phase 4: Update External Crates
1. Update `sakurs-py/Cargo.toml` to depend on `sakurs-api`
2. Update `sakurs-cli/Cargo.toml` to depend on `sakurs-api`
3. Fix all import paths
4. Ensure backward compatibility

### Phase 5: Update Tests and Benchmarks
1. Move unit tests to appropriate crates
2. Update integration tests
3. Fix benchmark imports
4. Add cross-crate integration tests

### Phase 6: Update Documentation
1. Update workspace Cargo.toml
2. Update README files
3. Update ARCHITECTURE.md
4. Add migration guide

### Phase 7: Final Verification
1. Run full test suite
2. Run benchmarks and compare performance
3. Verify Python bindings work
4. Verify CLI works
5. Clean up old sakurs-core

## Key Implementation Details

### DeltaScanner Design (Core)
```rust
pub struct DeltaScanner<'r, R: LanguageRules> {
    rules: &'r R,
    state: PartialState,
    offset: usize,
}

impl<'r, R: LanguageRules> DeltaScanner<'r, R> {
    pub fn step(
        &mut self, 
        ch: char, 
        emit: &mut impl FnMut(Boundary)
    ) {
        // Process character and emit boundaries
    }
}
```

### Executor Pattern (Engine)
```rust
pub trait Executor: Send + Sync {
    fn process(
        &self,
        text: &str,
        rules: &dyn LanguageRules,
    ) -> Result<Vec<Boundary>, EngineError>;
}
```

### Clean API (API Layer)
```rust
// Simple public API
pub fn process_text(text: &str) -> Result<Output, ApiError> {
    SentenceProcessor::new()
        .process(Input::Text(text.to_string()))
}
```

## Success Criteria
1. All tests pass
2. No performance regression (within 5%)
3. sakurs-py and sakurs-cli work unchanged
4. Clean dependency graph (core → engine → api)
5. No_std support in core

## Risk Mitigation
1. Keep old sakurs-core until migration complete
2. Run benchmarks after each phase
3. Test Python/CLI after each major change
4. Use feature flags for gradual rollout if needed