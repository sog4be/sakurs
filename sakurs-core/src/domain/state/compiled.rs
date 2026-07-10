//! Compiled language rules: the concrete [`Judge`] built from a TOML language
//! configuration, plus the character classification the scanner consumes.
//!
//! Both oracles are pure functions of the candidate's window, reproducing the
//! configurable-rules semantics on window-relative slices. Each sub-rule
//! consumes exactly the context reach it declares (standard 10-character
//! contexts, 21-character abbreviation lookback, ±20-byte ellipsis exception
//! window, ≤3-character suppression patterns, 11-character line-start
//! decision), all of which fit inside the ±[`WINDOW_CHARS`] judgment window —
//! [`CompiledRules::from_config`] rejects configurations that would not.

use super::candidate::{EnclosureSlot, Judge, Judgment, TerminatorKind};
use super::context::{fwd_chars, WINDOW_CHARS};
use crate::domain::error::DomainError;
use crate::domain::language::config::LanguageConfig;
use crate::domain::types::BoundaryFlags;
use regex::{Regex, RegexSet};
use std::collections::{HashMap, HashSet};

/// Standard context reach of the boundary sub-rules, in characters.
const CONTEXT_REACH: usize = 10;

/// Backward reach of the abbreviation lookup, in characters.
const ABBREVIATION_REACH: usize = 21;

/// Reach of the ellipsis exception regex window, in bytes (snapped to
/// character boundaries when sliced).
const ELLIPSIS_REGEX_REACH: usize = 20;

/// A line-start condition is decidable once this many characters precede the
/// position without a newline (the threshold compared against is 10).
const LINE_START_REACH: usize = 11;

/// Longest chain of closing enclosure characters a boundary-after-closers
/// candidate walks back through to reach its terminator (`."`, `.")`, …).
const CLOSER_CHAIN_MAX: usize = 3;

/// Classification of one character for the scanner.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct CharClass {
    /// Evaluate this character as a potential sentence terminator.
    pub terminator: bool,
    /// This character opens/closes/toggles an enclosure.
    pub enclosure: Option<EnclosureInfo>,
}

/// Enclosure role of a character.
#[derive(Debug, Clone, Copy)]
pub(crate) struct EnclosureInfo {
    /// Depth/parity effect when the character counts as a real enclosure.
    pub slot: EnclosureSlot,
    /// Whether a suppression rule can exclude this character, which forces
    /// occurrences near chunk edges to become pending enclosures.
    pub suppressible: bool,
}

/// One fast suppression pattern (see the TOML `[suppression]` section).
#[derive(Debug)]
struct SuppressionPattern {
    ch: char,
    line_start: bool,
    before: Option<String>,
    after: Option<String>,
}

/// Language rules compiled for the deferred-judgment pipeline.
#[derive(Debug)]
pub(crate) struct CompiledRules {
    /// ASCII classification table; non-ASCII characters fall back to the map.
    ascii: [CharClass; 128],
    other: HashMap<char, CharClass>,
    /// Number of asymmetric enclosure types (delta slots).
    asym_count: usize,

    // Terminator rules
    terminator_chars: HashSet<char>,
    terminator_patterns: Vec<String>,
    boundary_after_closers: bool,

    // Ellipsis rules
    ellipsis_treat_as_boundary: bool,
    ellipsis_patterns: Vec<String>,
    ellipsis_context_rules: Vec<(ContextCondition, bool)>,
    terminator_context_rules: Vec<(ContextCondition, bool, Vec<char>)>,
    ellipsis_exceptions: Vec<(Regex, bool)>,

    // Abbreviation rules
    abbreviations: ReverseTrie,

    // Sentence starter rules
    starter_set: HashSet<String>,
    starter_require_space: bool,
    starter_min_len: usize,

    // Suppression rules
    suppression_patterns: Vec<SuppressionPattern>,
    suppression_regexes: RegexSet,
}

/// Abbreviation matcher: a trie over *reversed* abbreviation strings
/// (case-sensitive — `a.m.` and `A.M.` are distinct entries), walked
/// backward from the period. One backward walk
/// replaces a forward walk from every candidate start position; the accepted
/// language is identical, because a forward match from `start` is exactly a
/// backward walk reaching depth `end − start` on an accepting node.
#[derive(Debug)]
struct ReverseTrie {
    /// Node arena; index 0 is the root.
    nodes: Vec<TrieNode>,
}

#[derive(Debug, Default)]
struct TrieNode {
    /// Sorted `(char, node index)` pairs; abbreviation alphabets are tiny,
    /// so binary search on a compact vector beats hashing.
    children: Vec<(char, u32)>,
    is_end: bool,
}

impl ReverseTrie {
    fn new() -> Self {
        Self {
            nodes: vec![TrieNode::default()],
        }
    }

    fn insert(&mut self, abbr: &str) {
        let mut node = 0usize;
        for ch in abbr.chars().rev() {
            node = match self.nodes[node]
                .children
                .binary_search_by_key(&ch, |&(c, _)| c)
            {
                Ok(i) => self.nodes[node].children[i].1 as usize,
                Err(i) => {
                    let idx = self.nodes.len();
                    self.nodes.push(TrieNode::default());
                    self.nodes[node].children.insert(i, (ch, idx as u32));
                    idx
                }
            };
        }
        self.nodes[node].is_end = true;
    }

    /// Byte length of the longest abbreviation ending exactly at the
    /// exclusive byte offset `end`, scanning back at most
    /// [`ABBREVIATION_REACH`] characters.
    fn longest_match_ending_at(&self, text: &str, end: usize) -> Option<usize> {
        if self.nodes.len() == 1 {
            return None;
        }
        let mut node = 0usize;
        let mut best = None;
        for (count, (start, ch)) in text[..end].char_indices().rev().enumerate() {
            if count >= ABBREVIATION_REACH {
                break;
            }
            match self.nodes[node]
                .children
                .binary_search_by_key(&ch, |&(c, _)| c)
            {
                Ok(i) => node = self.nodes[node].children[i].1 as usize,
                Err(_) => break,
            }
            if self.nodes[node].is_end {
                best = Some(end - start);
            }
        }
        best
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextCondition {
    FollowedByCapital,
    FollowedByLowercase,
    /// Unimplemented custom condition: never matches.
    Custom,
}

impl CompiledRules {
    /// Compiles the embedded configuration for a language code.
    #[cfg(test)]
    pub(crate) fn from_code(code: &str) -> Result<Self, DomainError> {
        Self::from_config(crate::domain::language::config::get_language_config(code)?)
    }

    /// Compiles a language configuration, rejecting it if any rule would need
    /// context beyond the ±[`WINDOW_CHARS`] judgment window.
    pub(crate) fn from_config(config: &LanguageConfig) -> Result<Self, DomainError> {
        let required = required_window(config);
        if required > WINDOW_CHARS {
            return Err(DomainError::ConfigurationError(format!(
                "language '{}' needs a ±{} character judgment window, but the \
                 algorithm's window is ±{} (WINDOW_CHARS)",
                config.metadata.code, required, WINDOW_CHARS
            )));
        }

        let mut ascii = [CharClass::default(); 128];
        let mut other: HashMap<char, CharClass> = HashMap::new();
        let mut classify = |ch: char, f: &mut dyn FnMut(&mut CharClass)| {
            if (ch as u32) < 128 {
                f(&mut ascii[ch as usize]);
            } else {
                f(other.entry(ch).or_default());
            }
        };

        // Every character that can begin a boundary decision is evaluated:
        // terminator chars, terminator-pattern chars, ellipsis-pattern chars.
        // Only `terminators.chars` members pass the terminator gate in
        // `judge`, mirroring the legacy rules.
        let terminator_chars: HashSet<char> = config.terminators.chars.iter().copied().collect();
        let mut potential: Vec<char> = config.terminators.chars.clone();
        for p in &config.terminators.patterns {
            potential.extend(p.pattern.chars());
        }
        for p in &config.ellipsis.patterns {
            potential.extend(p.chars());
        }
        for ch in potential {
            classify(ch, &mut |c| c.terminator = true);
        }

        // Suppressible characters: named by a fast pattern, or any enclosure
        // character when regex suppression patterns exist (the legacy
        // suppressor runs regexes for every enclosure character).
        let fast_chars: HashSet<char> = config
            .suppression
            .fast_patterns
            .iter()
            .map(|p| p.char)
            .collect();
        let regexes_present = !config.suppression.regex_patterns.is_empty();

        // Enclosure slots: asymmetric pairs take delta indices, symmetric
        // pairs take parity bits, both in configuration order.
        let mut asym_count = 0usize;
        let mut sym_count = 0usize;
        for pair in &config.enclosures.pairs {
            if pair.symmetric {
                let bit = u8::try_from(sym_count).map_err(|_| {
                    DomainError::ConfigurationError("too many symmetric enclosure types".into())
                })?;
                if sym_count >= 32 {
                    return Err(DomainError::ConfigurationError(
                        "at most 32 symmetric enclosure types are supported".into(),
                    ));
                }
                sym_count += 1;
                let slot = EnclosureSlot::Sym { bit };
                for ch in [pair.open, pair.close] {
                    let suppressible = fast_chars.contains(&ch) || regexes_present;
                    classify(ch, &mut |c| {
                        c.enclosure = Some(EnclosureInfo { slot, suppressible })
                    });
                }
            } else {
                let index = u8::try_from(asym_count).map_err(|_| {
                    DomainError::ConfigurationError("too many asymmetric enclosure types".into())
                })?;
                asym_count += 1;
                for (ch, delta) in [(pair.open, 1i8), (pair.close, -1i8)] {
                    let suppressible = fast_chars.contains(&ch) || regexes_present;
                    let slot = EnclosureSlot::Asym { index, delta };
                    classify(ch, &mut |c| {
                        c.enclosure = Some(EnclosureInfo { slot, suppressible })
                    });
                }
            }
        }

        let parse_condition = |s: &str| match s {
            "followed_by_capital" => ContextCondition::FollowedByCapital,
            "followed_by_lowercase" => ContextCondition::FollowedByLowercase,
            _ => ContextCondition::Custom,
        };
        let ellipsis_context_rules = config
            .ellipsis
            .context_rules
            .iter()
            .map(|r| (parse_condition(&r.condition), r.boundary))
            .collect();
        let terminator_context_rules = config
            .terminators
            .context_rules
            .iter()
            .map(|r| (parse_condition(&r.condition), r.boundary, r.chars.clone()))
            .collect();

        let ellipsis_exceptions = config
            .ellipsis
            .exceptions
            .iter()
            .map(|e| Ok((Regex::new(&e.regex)?, e.boundary)))
            .collect::<Result<Vec<_>, regex::Error>>()
            .map_err(|e| {
                DomainError::InvalidLanguageRules(format!("invalid ellipsis regex: {e}"))
            })?;

        // A single RegexSet pass replaces one is_match call per pattern on
        // every suppressible enclosure character.
        let suppression_regexes =
            RegexSet::new(config.suppression.regex_patterns.iter().map(|p| &p.pattern)).map_err(
                |e| DomainError::InvalidLanguageRules(format!("invalid suppression regex: {e}")),
            )?;

        let (starter_set, starter_require_space, starter_min_len) =
            if let Some(ref starters) = config.sentence_starters {
                let mut set = HashSet::new();
                for words in starters.categories.values() {
                    for w in words {
                        if w.len() >= starters.min_word_length {
                            set.insert(w.clone());
                        }
                    }
                }
                (
                    set,
                    starters.require_following_space,
                    starters.min_word_length,
                )
            } else {
                (HashSet::new(), true, 1)
            };

        Ok(Self {
            ascii,
            other,
            asym_count,
            terminator_chars,
            terminator_patterns: config
                .terminators
                .patterns
                .iter()
                .map(|p| p.pattern.clone())
                .collect(),
            boundary_after_closers: config.terminators.boundary_after_closers,
            terminator_context_rules,
            ellipsis_treat_as_boundary: config.ellipsis.treat_as_boundary,
            ellipsis_patterns: config.ellipsis.patterns.clone(),
            ellipsis_context_rules,
            ellipsis_exceptions,
            abbreviations: {
                // Case-sensitive: the configuration lists each casing it
                // accepts (No/no, Vol/vol), and casing can carry meaning —
                // lowercase a.m. is an abbreviation while uppercase P.M.
                // before a capitalized word ends the sentence.
                let mut trie = ReverseTrie::new();
                for words in config.abbreviations.categories.values() {
                    for word in words {
                        trie.insert(word);
                    }
                }
                trie
            },
            starter_set,
            starter_require_space,
            starter_min_len,
            suppression_patterns: config
                .suppression
                .fast_patterns
                .iter()
                .map(|p| SuppressionPattern {
                    ch: p.char,
                    line_start: p.line_start,
                    before: p.before.clone(),
                    after: p.after.clone(),
                })
                .collect(),
            suppression_regexes,
        })
    }

    /// Number of asymmetric enclosure types (size of the delta vector).
    pub(crate) fn asym_type_count(&self) -> usize {
        self.asym_count
    }

    /// Classifies one character for the scanner.
    #[inline]
    pub(crate) fn classify(&self, ch: char) -> CharClass {
        if (ch as u32) < 128 {
            self.ascii[ch as usize]
        } else {
            self.other.get(&ch).copied().unwrap_or_default()
        }
    }

    /// True when an ellipsis pattern ends exactly at `pos` (the byte offset
    /// just after the pattern's last character) *and* the run stops there: a
    /// completion inside a longer run (the third dot of `....`, the first
    /// `…` of `……`) does not fire — only the run's final position does, so
    /// each maximal run yields at most one boundary.
    fn ellipsis_completes_at(&self, w: &str, pos: usize) -> bool {
        let ends_here = self.ellipsis_patterns.iter().any(|p| {
            pos >= p.len() && w.is_char_boundary(pos - p.len()) && &w[pos - p.len()..pos] == p
        });
        if !ends_here {
            return false;
        }
        // The run continues if any pattern occurrence overlaps or starts at
        // `pos` and ends beyond it.
        !self.ellipsis_patterns.iter().any(|q| {
            let qlen = q.len();
            (pos.saturating_sub(qlen - 1)..=pos).any(|s| {
                w.is_char_boundary(s)
                    && w.len() >= s + qlen
                    && w.is_char_boundary(s + qlen)
                    && &w[s..s + qlen] == q
            })
        })
    }

    /// Ellipsis boundary evaluation: exception regexes on a ±20-byte window
    /// around the terminator, then context rules, then the default.
    fn evaluate_ellipsis(&self, w: &str, term_pos: usize, following10: &str) -> Judgment {
        let mut start = term_pos.saturating_sub(ELLIPSIS_REGEX_REACH);
        while start > 0 && !w.is_char_boundary(start) {
            start -= 1;
        }
        let mut end = (term_pos + ELLIPSIS_REGEX_REACH).min(w.len());
        while end < w.len() && !w.is_char_boundary(end) {
            end += 1;
        }
        let regex_window = &w[start..end];
        for (regex, is_boundary) in &self.ellipsis_exceptions {
            if regex.is_match(regex_window) {
                return if *is_boundary {
                    Judgment::Boundary(BoundaryFlags::WEAK)
                } else {
                    Judgment::NotBoundary
                };
            }
        }

        for (cond, is_boundary) in &self.ellipsis_context_rules {
            let first_alpha = following10.chars().find(|c| c.is_alphabetic());
            let matches = match cond {
                ContextCondition::FollowedByCapital => {
                    first_alpha.map(char::is_uppercase).unwrap_or(false)
                }
                ContextCondition::FollowedByLowercase => {
                    first_alpha.map(char::is_lowercase).unwrap_or(false)
                }
                ContextCondition::Custom => false,
            };
            if matches {
                return if *is_boundary {
                    Judgment::Boundary(BoundaryFlags::WEAK)
                } else {
                    Judgment::NotBoundary
                };
            }
        }

        if self.ellipsis_treat_as_boundary {
            Judgment::Boundary(BoundaryFlags::WEAK)
        } else {
            Judgment::NotBoundary
        }
    }

    /// Multi-period abbreviation context (`U.S.A.`, `Ph.D.`): 1–2 letters
    /// before the period and 1–2 letters immediately after it (same token,
    /// no whitespace) followed by another period. Whitespace after the
    /// period means the next word is a separate token — `P.M. Mr.` is a
    /// sentence break plus a title, not one spaced abbreviation.
    fn is_multi_period_context(&self, preceding10: &str, following10: &str) -> bool {
        let mut letters_before = 0usize;
        let mut before_run_start: Option<char> = None;
        for ch in preceding10.chars().rev() {
            if ch.is_alphabetic() && letters_before < 3 {
                letters_before += 1;
            } else {
                before_run_start = Some(ch);
                break;
            }
        }
        if letters_before == 0 || letters_before > 2 {
            return false;
        }
        if before_run_start.map(char::is_alphabetic).unwrap_or(false) {
            return false;
        }

        let mut it = following10.chars().peekable();
        let mut letters_after = 0usize;
        while it.peek().is_some_and(|c| c.is_alphabetic()) && letters_after < 3 {
            it.next();
            letters_after += 1;
        }
        letters_after > 0 && letters_after <= 2 && it.peek() == Some(&'.')
    }

    /// Abbreviation ending at `term_pos` (the period's byte offset) with a
    /// word boundary before it.
    fn abbreviation_ends_at(&self, w: &str, term_pos: usize) -> Option<usize> {
        if term_pos == 0 {
            return None;
        }
        let length = self.abbreviations.longest_match_ending_at(w, term_pos)?;
        let abbr_start = term_pos - length;
        let has_word_boundary = abbr_start == 0
            || w[..abbr_start]
                .chars()
                .next_back()
                .map(|c| !c.is_alphanumeric())
                .unwrap_or(true);
        has_word_boundary.then_some(length)
    }

    /// Extracts the next word from the following context: skip whitespace,
    /// take alphabetic characters. Returns the word and the rest.
    fn extract_next_word(following: &str) -> Option<(&str, &str)> {
        let trimmed_start = following.len() - following.trim_start().len();
        let rest = &following[trimmed_start..];
        let word_len = rest
            .char_indices()
            .find(|(_, c)| !c.is_alphabetic())
            .map(|(i, _)| i)
            .unwrap_or(rest.len());
        if word_len == 0 {
            None
        } else {
            Some((&rest[..word_len], &rest[word_len..]))
        }
    }

    fn is_sentence_starter(&self, word: &str, remaining: &str) -> bool {
        if word.len() < self.starter_min_len || !self.starter_set.contains(word) {
            return false;
        }
        if self.starter_require_space {
            remaining.chars().next().is_some_and(char::is_whitespace)
        } else {
            true
        }
    }

    /// Whether the language places boundaries after closers that immediately
    /// follow a terminator (the scanner consults this to emit candidates at
    /// closing-capable enclosure characters).
    pub(crate) fn boundary_after_closers(&self) -> bool {
        self.boundary_after_closers
    }

    /// Core terminator judgment — a window-relative port of the legacy
    /// `detect_sentence_boundary`. `after_term` is the byte offset just after
    /// the terminator. Forward-context rules read from `follow_from`: equal to
    /// `after_term` for a plain terminator, or the offset after the closer
    /// chain for a boundary-after-closers candidate, so rules like sentence
    /// starters see the text after the closing quote.
    fn judge_terminator(
        &self,
        w: &str,
        after_term: usize,
        ch: char,
        follow_from: usize,
    ) -> Judgment {
        let term_pos = after_term - ch.len_utf8();
        let following = &w[follow_from..];
        let following10 = &following[..fwd_chars(following, 0, CONTEXT_REACH)];
        let preceding = &w[..term_pos];
        let preceding10 =
            &preceding[super::context::back_chars(preceding, preceding.len(), CONTEXT_REACH)..];

        // 1. A completed ellipsis run gets the ellipsis evaluation.
        if self.ellipsis_completes_at(w, after_term) {
            return self.evaluate_ellipsis(w, term_pos, following10);
        }

        if ch == '.' {
            // 2. Inside an unfinished dot run (first/second dot of "..."):
            //    the completed pattern is judged at its last dot.
            if following10.starts_with('.') {
                return Judgment::NotBoundary;
            }
            // 3. Multi-period abbreviation pattern (U.S.A., Ph.D.).
            if self.is_multi_period_context(preceding10, following10) {
                return Judgment::NotBoundary;
            }
        }

        if !self.terminator_chars.contains(&ch) {
            return Judgment::NotBoundary;
        }

        // 4. Multi-character terminator patterns ("!?"): strong boundary at
        //    the pattern's last character, no boundary before it completes.
        for pattern in &self.terminator_patterns {
            if after_term >= pattern.len()
                && w.is_char_boundary(after_term - pattern.len())
                && &w[after_term - pattern.len()..after_term] == pattern.as_str()
            {
                return Judgment::Boundary(BoundaryFlags::STRONG);
            }
        }
        if let Some(next) = following.chars().next() {
            let starts_two_char_pattern = self.terminator_patterns.iter().any(|p| {
                let mut pc = p.chars();
                pc.next() == Some(ch) && pc.next() == Some(next) && pc.next().is_none()
            });
            if starts_two_char_pattern {
                return Judgment::NotBoundary;
            }
        }

        // 5. Abbreviations: no boundary, unless followed by a configured
        //    sentence starter (weak boundary) or the end of text. A following
        //    word that isn't a starter, and any non-letter continuation
        //    (comma, apostrophe, digit, closing delimiter — "U.S., drafted",
        //    "Jr.'s", "p. 55"), keep the sentence open. Only a period can be
        //    an abbreviation dot; "no!" must not match the abbreviation "no".
        if ch == '.' && self.abbreviation_ends_at(w, term_pos).is_some() {
            return match Self::extract_next_word(following10) {
                Some((word, remaining)) => {
                    if self.is_sentence_starter(word, remaining) {
                        Judgment::Boundary(BoundaryFlags::WEAK)
                    } else {
                        Judgment::NotBoundary
                    }
                }
                None if following10.trim_start().is_empty() => {
                    Judgment::Boundary(BoundaryFlags::WEAK)
                }
                None => Judgment::NotBoundary,
            };
        }

        // 6. Default single-terminator evaluation.
        let default_judgment = match ch {
            '!' | '?' | '！' | '？' => Judgment::Boundary(BoundaryFlags::STRONG),
            '.' | '。' => {
                let digit_before = preceding
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_ascii_digit());
                let digit_after = following.chars().next().is_some_and(|c| c.is_ascii_digit());
                if digit_before && digit_after {
                    Judgment::NotBoundary
                } else {
                    Judgment::Boundary(BoundaryFlags::WEAK)
                }
            }
            _ => Judgment::NotBoundary,
        };

        // 7. Terminator context rules refine a positive default verdict
        //    ("Yahoo! in the department" — next word lowercase, keep the
        //    sentence open). They never resurrect a negative one: a decimal
        //    point stays a decimal point.
        if let Judgment::Boundary(_) = default_judgment {
            for (cond, boundary, chars) in &self.terminator_context_rules {
                if !chars.is_empty() && !chars.contains(&ch) {
                    continue;
                }
                let first_alpha = following10.chars().find(|c| c.is_alphabetic());
                let matches = match cond {
                    ContextCondition::FollowedByCapital => {
                        first_alpha.map(char::is_uppercase).unwrap_or(false)
                    }
                    ContextCondition::FollowedByLowercase => {
                        first_alpha.map(char::is_lowercase).unwrap_or(false)
                    }
                    ContextCondition::Custom => false,
                };
                if matches {
                    return if *boundary {
                        default_judgment
                    } else {
                        Judgment::NotBoundary
                    };
                }
            }
        }
        default_judgment
    }

    /// Boundary-after-closers judgment: the candidate at `pos` sits just
    /// after a closing-capable enclosure character. Walk back through at most
    /// [`CLOSER_CHAIN_MAX`] such characters (each must be a real enclosure,
    /// not a suppressed use) to the terminator, re-judge it with forward
    /// context read from after the chain, and require the follow condition —
    /// an uppercase letter or the end of text. Structural
    /// containment is not decided here: a candidate emitted after an *opening*
    /// toggle simply records a depth/parity inside the enclosure and is
    /// dropped by the reduce predicate.
    fn judge_after_closers(&self, w: &str, pos: usize, _closer: char) -> Judgment {
        if !self.boundary_after_closers {
            return Judgment::NotBoundary;
        }

        let mut cur = pos;
        let mut closers = 0usize;
        loop {
            let Some(ch_at) = w[..cur].chars().next_back() else {
                return Judgment::NotBoundary;
            };
            let start = cur - ch_at.len_utf8();
            let class = self.classify(ch_at);
            if let Some(enc) = class.enclosure {
                if !enc.slot.closing_capable() || closers >= CLOSER_CHAIN_MAX {
                    return Judgment::NotBoundary;
                }
                if enc.suppressible && Judge::suppress_enclosure(self, w, start, ch_at) {
                    return Judgment::NotBoundary;
                }
                closers += 1;
                cur = start;
                continue;
            }
            if closers == 0 || !class.terminator {
                return Judgment::NotBoundary;
            }

            let Judgment::Boundary(flags) = self.judge_terminator(w, cur, ch_at, pos) else {
                return Judgment::NotBoundary;
            };
            return match w[pos..].chars().find(|c| !c.is_whitespace()) {
                None => Judgment::Boundary(flags),
                Some(next) if next.is_uppercase() => Judgment::Boundary(flags),
                Some(_) => Judgment::NotBoundary,
            };
        }
    }
}

/// The judgment-window requirement of a configuration, in characters.
fn required_window(config: &LanguageConfig) -> usize {
    let longest_terminator_pattern = config
        .terminators
        .patterns
        .iter()
        .map(|p| p.pattern.chars().count())
        .max()
        .unwrap_or(0);
    let longest_ellipsis_pattern = config
        .ellipsis
        .patterns
        .iter()
        .map(|p| p.chars().count())
        .max()
        .unwrap_or(0);
    // A boundary-after-closers candidate sits up to CLOSER_CHAIN_MAX
    // characters past its terminator, extending every backward reach by the
    // chain length (plus the terminator character itself).
    let closer_chain = if config.terminators.boundary_after_closers {
        CLOSER_CHAIN_MAX + 1
    } else {
        0
    };
    [
        CONTEXT_REACH + 1,
        ABBREVIATION_REACH + 1 + closer_chain,
        ELLIPSIS_REGEX_REACH + 1 + closer_chain,
        LINE_START_REACH,
        longest_terminator_pattern + 1 + closer_chain,
        longest_ellipsis_pattern + 1 + closer_chain,
    ]
    .into_iter()
    .max()
    .unwrap_or(0)
}

impl Judge for CompiledRules {
    fn judge(&self, w: &str, pos_in_window: usize, kind: TerminatorKind) -> Judgment {
        match kind {
            TerminatorKind::Char(ch) => self.judge_terminator(w, pos_in_window, ch, pos_in_window),
            TerminatorKind::AfterClosers(ch) => self.judge_after_closers(w, pos_in_window, ch),
        }
    }

    /// Window-relative port of the legacy `Suppressor`.
    fn suppress_enclosure(&self, w: &str, pos_in_window: usize, ch: char) -> bool {
        let preceding = &w[..pos_in_window];
        let following_after_ch = {
            let mut it = w[pos_in_window..].chars();
            it.next(); // the enclosure character itself
            it.as_str()
        };

        for pattern in &self.suppression_patterns {
            if pattern.ch != ch {
                continue;
            }
            if pattern.line_start {
                // Decidable within 11 characters: a newline within that reach
                // gives the offset, its absence means offset > threshold.
                let mut offset = 0usize;
                let mut found = false;
                for c in preceding.chars().rev().take(LINE_START_REACH) {
                    if c == '\n' {
                        found = true;
                        break;
                    }
                    offset += 1;
                }
                let at_line_start = (found || offset < LINE_START_REACH) && offset <= 10;
                if !at_line_start {
                    continue;
                }
            }
            if let Some(ref class) = pattern.before {
                if !char_matches_class(preceding.chars().next_back(), class) {
                    continue;
                }
            }
            if let Some(ref class) = pattern.after {
                if !char_matches_class(following_after_ch.chars().next(), class) {
                    continue;
                }
            }
            return true;
        }

        if !self.suppression_regexes.is_empty() {
            // The legacy regex window is the 3 characters on each side:
            // at most 7 characters, so it fits a small stack buffer.
            let start = super::context::back_chars(preceding, preceding.len(), 3);
            let end = fwd_chars(following_after_ch, 0, 3);
            let mut buf = [0u8; 32];
            let mut len = 0;
            for &b in &preceding.as_bytes()[start..] {
                buf[len] = b;
                len += 1;
            }
            len += ch.encode_utf8(&mut buf[len..]).len();
            for &b in &following_after_ch.as_bytes()[..end] {
                buf[len] = b;
                len += 1;
            }
            let window = std::str::from_utf8(&buf[..len]).expect("window bytes are valid UTF-8");
            if self.suppression_regexes.is_match(window) {
                return true;
            }
        }

        false
    }
}

fn char_matches_class(ch: Option<char>, class: &str) -> bool {
    match (ch, class) {
        (Some(c), "alpha") => c.is_alphabetic(),
        (Some(c), "alnum") => c.is_alphanumeric(),
        (Some(c), "digit") => c.is_ascii_digit(),
        (Some(c), "whitespace") => c.is_whitespace(),
        // Unknown classes (and missing context) never match, matching the
        // legacy suppressor.
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_configs_fit_the_window() {
        for code in crate::domain::language::config::list_available_languages() {
            CompiledRules::from_code(code)
                .unwrap_or_else(|e| panic!("config '{code}' must compile: {e}"));
        }
    }

    #[test]
    fn oversized_pattern_is_rejected() {
        let toml = format!(
            r#"
            [metadata]
            code = "xx"
            name = "Test"
            [terminators]
            chars = ["."]
            [ellipsis]
            patterns = ["{}"]
            [enclosures]
            pairs = []
            [suppression]
            fast_patterns = []
            "#,
            ".".repeat(WINDOW_CHARS + 1)
        );
        let config: LanguageConfig = toml::from_str(&toml).unwrap();
        let err = CompiledRules::from_config(&config).unwrap_err();
        assert!(
            err.to_string().contains("judgment window"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn en_classification_covers_terminators_and_enclosures() {
        let rules = CompiledRules::from_code("en").unwrap();
        assert!(rules.classify('.').terminator);
        assert!(rules.classify('?').terminator);
        assert!(
            rules.classify('…').terminator,
            "ellipsis chars are evaluated"
        );
        assert!(!rules.classify('a').terminator);

        let paren = rules.classify('(').enclosure.unwrap();
        assert!(matches!(paren.slot, EnclosureSlot::Asym { delta: 1, .. }));
        assert!(
            paren.suppressible,
            "en has suppression regexes, which the legacy rules evaluate for every enclosure char"
        );

        let apostrophe = rules.classify('\'').enclosure.unwrap();
        assert!(matches!(apostrophe.slot, EnclosureSlot::Sym { .. }));
        assert!(apostrophe.suppressible, "apostrophe has suppression rules");

        assert!(rules.asym_type_count() >= 3);
        let quote = rules.classify('"').enclosure.unwrap();
        match (apostrophe.slot, quote.slot) {
            (EnclosureSlot::Sym { bit: a }, EnclosureSlot::Sym { bit: q }) => assert_ne!(a, q),
            other => panic!("both quotes must be symmetric, got {other:?}"),
        }
    }
}
