# Architecture Refactoring Notes

## Problem: Trait Object vs Concrete Types

The current issue is that:
1. `SentenceProcessor` stores `Arc<dyn LanguageRules>` (trait object)
2. `DeltaScanner` expects a concrete type implementing `LanguageRules`
3. The `Executor` trait tries to be generic over `LanguageRules`

This creates a mismatch because you can't pass `&dyn LanguageRules` where a generic `R: LanguageRules` is expected.

## Solutions Considered

### 1. Make everything use trait objects (❌ Rejected)
- Would require `DeltaScanner` to work with `&dyn LanguageRules`
- Performance implications due to dynamic dispatch
- Breaks the zero-cost abstraction principle

### 2. Make SentenceProcessor generic (❌ Rejected)
- Would make `SentenceProcessor<R: LanguageRules>` 
- Forces all users to specify the language rules type
- Breaks the clean API design

### 3. Use enum dispatch (✅ Current approach)
- Create a concrete enum of all supported language rules
- Avoids trait objects in the hot path
- Maintains clean API while preserving performance

## Implementation Plan

1. Create `LanguageRulesImpl` enum in engine that wraps all concrete implementations
2. Update `SentenceProcessor` to use this enum instead of trait object
3. Keep the `LanguageRules` trait for extensibility
4. Executors work with concrete types

This maintains the benefits of the new architecture while avoiding the trait object performance penalty.