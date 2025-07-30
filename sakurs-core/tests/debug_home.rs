//! Debug home. detection

use sakurs_core::language::get_rules;

#[test]
fn debug_home_decision() {
    let rules = get_rules("en").unwrap();

    // Test various texts that end with "e."
    let tests = vec![
        ("e.", 2),
        ("home.", 5),
        ("went home.", 10),
        ("E.", 2),
        ("HOME.", 5),
    ];

    for (text, pos) in tests {
        println!("\nTesting: '{}' at pos {}", text, pos);

        // Check if it's a terminator
        let term_char = text.chars().nth(text[..pos].chars().count() - 1).unwrap();
        println!("  Terminator char: '{}'", term_char);
        println!("  Is terminator: {}", rules.is_terminator_char(term_char));

        // Check dot role
        if term_char == '.' {
            let chars: Vec<char> = text.chars().collect();
            let char_pos = text[..pos].chars().count();
            let prev = if char_pos > 1 {
                chars.get(char_pos - 2).copied()
            } else {
                None
            };
            let next = chars.get(char_pos).copied();
            let dot_role = rules.dot_role(prev, next);
            println!(
                "  Dot role: {:?} (prev={:?}, next={:?})",
                dot_role, prev, next
            );
        }

        // Check abbreviation
        let is_abbrev = rules.is_abbreviation(text, pos - 1);
        println!("  Is abbreviation: {}", is_abbrev);

        // Get boundary decision
        let prev_char = if pos > 1 {
            text.chars().nth(pos - 2)
        } else {
            None
        };
        let next_char = text.chars().nth(pos);
        let decision = rules.boundary_decision(text, pos, '.', prev_char, next_char);
        println!("  Decision: {:?}", decision);
    }
}
