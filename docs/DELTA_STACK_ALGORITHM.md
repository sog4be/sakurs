# Δ-Stack Monoid Algorithm: Core Concepts and Implementation

## Overview

The Δ-Stack Monoid algorithm revolutionizes sentence boundary detection (SBD) by transforming what has traditionally been a sequential process into a parallelizable one. The key innovation lies in formulating SBD as an associative monoid operation, enabling mathematically-sound parallel execution while maintaining perfect accuracy.

## Key Innovation: Monoid Formulation

### Why Monoids Enable Parallelization

A monoid is a mathematical structure with three properties:
1. **Closure**: Combining two states produces another valid state
2. **Associativity**: `(a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)`
3. **Identity element**: There exists an element `e` where `a ⊕ e = e ⊕ a = a`

The associativity property is crucial because it means we can split text into chunks, process them independently, and combine results in any order while getting the same final result.

### State Representation

The algorithm represents the parsing state as a triple:

```
State = ⟨B, Δ, A⟩
```

Where:
- **B (Boundaries)**: Set of detected sentence boundaries with metadata
- **Δ (Delta Stack)**: Tracks enclosure states (quotes, parentheses) across chunks
- **A (Abbreviation State)**: Handles cross-chunk abbreviation patterns

Let's examine each component:

#### 1. Boundary Set (B)

```
B = {(offset, flags) | offset ∈ ℕ, flags ∈ {STRONG, FROM_ABBR}}
```

Each boundary stores:
- `offset`: Position in the text
- `flags`: Metadata about the boundary type

#### 2. Delta Stack (Δ)

The delta stack is the algorithm's most innovative component. For each enclosure type (parentheses, quotes, etc.), it tracks:

```
Δ = [(net₁, min₁), ..., (netₙ, minₙ)]
```

- `netᵢ`: Net count of opening minus closing delimiters
- `minᵢ`: Minimum cumulative sum observed during the chunk scan

This clever representation allows us to determine if we're inside an enclosure at chunk boundaries without knowing the global state.

#### 3. Abbreviation State (A)

```
A = (dangling_dot, head_alpha)
```

- `dangling_dot`: Whether the chunk ends with a potential abbreviation dot
- `head_alpha`: Whether the chunk starts with alphabetic characters

This enables detection of abbreviations that span chunk boundaries (e.g., "U.S." split as "U." | "S.").

## The Monoid Operations

### Identity Element

The identity element represents an empty text segment:

```
e = ⟨∅, 0⃗, (false, false)⟩
```

### Combine Operation (⊕)

The combine operation merges two adjacent text segments' states:

```python
def combine(state1, state2):
    # Merge boundaries with offset adjustment
    merged_boundaries = merge_boundaries(state1.B, state2.B, state1.A, state2.A)
    
    # Merge delta stacks
    merged_deltas = merge_deltas(state1.Δ, state2.Δ)
    
    # Merge abbreviation states
    merged_abbr = merge_abbr(state1.A, state2.A)
    
    return State(merged_boundaries, merged_deltas, merged_abbr)
```

### Delta Merge Algorithm

The delta merge is particularly elegant:

```python
def merge_deltas(Δ1, Δ2):
    result = []
    for i in range(len(Δ1)):
        net1, min1 = Δ1[i]
        net2, min2 = Δ2[i]
        
        # Combined net change
        new_net = net1 + net2
        
        # The minimum in the combined range is either:
        # - The minimum from the left chunk, or
        # - The left's net + minimum from the right chunk
        new_min = min(min1, net1 + min2)
        
        result.append((new_net, new_min))
    
    return result
```

This formula correctly tracks whether we ever go "negative" (more closing than opening delimiters) even when processing chunks in parallel.

## Parallel Algorithm

### Map Phase

Each thread processes a chunk independently:

```rust
fn parse_chunk(chunk: &str, config: &Config) -> PartialState {
    let mut state = PartialState::new();
    let mut depth = vec![0; config.enclosure_count()];
    let mut min_prefix = vec![0; config.enclosure_count()];
    let mut total_depth = 0;
    
    for (i, ch) in chunk.char_indices() {
        // Track enclosure depth
        if let Some(id) = config.open_id(ch) {
            depth[id] += 1;
            total_depth += 1;
        } else if let Some(id) = config.close_id(ch) {
            depth[id] -= 1;
            total_depth -= 1;
            min_prefix[id] = min_prefix[id].min(depth[id]);
        }
        
        // Detect boundaries only at depth 0
        else if config.is_terminator(ch) && total_depth == 0 {
            state.add_boundary(i + ch.len_utf8());
        }
    }
    
    // Convert to delta representation
    state.enclosures = create_deltas(depth, min_prefix);
    state.abbr_state = detect_abbreviation_state(chunk);
    
    return state;
}
```

### Reduce Phase

The reduce phase combines chunk results using tree reduction:

```rust
fn reduce_states(states: Vec<PartialState>) -> PartialState {
    let mut level = states;
    
    // Tree reduction maintains O(log P) depth
    while level.len() > 1 {
        let mut next_level = Vec::new();
        
        for chunk in level.chunks(2) {
            match chunk {
                [left, right] => next_level.push(left.combine(right)),
                [single] => next_level.push(single.clone()),
            }
        }
        
        level = next_level;
    }
    
    level.into_iter().next().unwrap()
}
```

## Handling Complex Cases

### Nested Quotations

Japanese text often contains nested quotations like 「彼は『こんにちは』と言った」. The algorithm handles this through the delta stack mechanism:

1. When encountering 「, increment depth for quote type 1
2. When encountering 『, increment depth for quote type 2
3. Only mark sentence boundaries when total depth = 0
4. The delta representation preserves nesting information across chunks

### Multi-Dot Abbreviations

The algorithm uses a look-ahead approach for abbreviations:

```rust
fn detect_multi_dot(text: &str, start: usize) -> Option<(String, usize)> {
    let mut pos = start;
    let mut pattern = String::new();
    
    // Look for pattern: letter+ dot letter+ dot ...
    while pos < text.len() && pattern.len() < MAX_ABBR_LENGTH {
        // Expect letters
        let letter_start = pos;
        while pos < text.len() && text[pos].is_alphabetic() {
            pos += 1;
        }
        
        if pos == letter_start {
            break; // No letters found
        }
        
        // Expect dot
        if pos < text.len() && text[pos] == '.' {
            pattern.push_str(&text[letter_start..=pos]);
            pos += 1;
        } else {
            break;
        }
    }
    
    if pattern.matches('.').count() > 1 {
        Some((pattern, pos))
    } else {
        None
    }
}
```

### Cross-Chunk Abbreviations

When an abbreviation spans chunks (e.g., "U.S." split as "U." | "S."), the algorithm:

1. Left chunk sets `dangling_dot = true`
2. Right chunk sets `head_alpha = true`
3. During merge, if both conditions are met:
   - Remove the boundary after the dot in the left chunk
   - Suppress boundaries in the right chunk until non-abbreviation text

## Performance Characteristics

### Complexity Analysis

- **Sequential baseline**: O(N) time, O(1) space
- **Δ-Stack parallel**: O(N/P + log P) time, O(P) space

Where:
- N = text length
- P = number of processors

### Why It's Fast

1. **Single pass**: Each byte is read exactly once
2. **Cache-friendly**: Sequential memory access within chunks
3. **SIMD-capable**: Character scanning can use vector instructions
4. **Lock-free**: No synchronization needed during map phase
5. **Work-efficient**: Total work remains O(N)

## Implementation Insights

### Chunk Boundary Handling

The algorithm must carefully handle UTF-8 boundaries:

```rust
fn find_chunk_boundary(text: &[u8], target: usize) -> usize {
    let mut pos = target.min(text.len());
    
    // Backtrack to valid UTF-8 boundary
    while pos > 0 && !is_utf8_char_boundary(text[pos]) {
        pos -= 1;
    }
    
    pos
}
```

### Streaming Architecture

For real-time processing, the algorithm maintains state across chunks:

```rust
impl StreamingSegmenter {
    pub fn push_chunk(&mut self, chunk: &str) -> Vec<String> {
        // Combine carry-over with new chunk
        let combined = self.carry_text.clone() + chunk;
        
        // Parse combined text
        let state = parse_chunk(&combined, &self.config);
        
        // Extract complete sentences
        let safe_boundary = combined.len() - OVERLAP_SIZE;
        let sentences = extract_sentences_up_to(&combined, &state, safe_boundary);
        
        // Update carry-over
        let last_boundary = sentences.last()
            .map(|s| s.end_offset)
            .unwrap_or(0);
        self.carry_text = combined[last_boundary..].to_string();
        
        sentences
    }
}
```

## Mathematical Proof Sketch

The associativity of the combine operation is proven component-wise:

1. **Boundaries**: Shifting offsets and handling abbreviations preserves associativity
2. **Deltas**: Addition and minimum operations are associative
3. **Abbreviation state**: The (head, tail) selection is associative

The formal proof shows that for any three states s₁, s₂, s₃:
```
(s₁ ⊕ s₂) ⊕ s₃ = s₁ ⊕ (s₂ ⊕ s₃)
```

## Conclusion

The Δ-Stack Monoid algorithm achieves parallelization of sentence boundary detection through elegant mathematical formulation. By representing the parsing state as a monoid and carefully handling cross-chunk dependencies, it achieves near-linear speedup while maintaining exact compatibility with complex punctuation rules. The algorithm's success demonstrates the power of applying algebraic structures to traditionally sequential text processing tasks.