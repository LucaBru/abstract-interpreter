use std::collections::{BTreeMap, HashMap};

use crate::{abstract_domains::abstract_domain::AbstractDomain, interpreter::ProgramInvariants};

pub fn decorate_code_with_analysis<'a, D: AbstractDomain>(
    source_code: String,
    mut invariants: ProgramInvariants<'a, D>,
) -> String {
    // Extract last invariant safely
    let program_inv = invariants
        .pop_last()
        .map(|(_, inv)| format!("\n# {}", inv))
        .unwrap_or_else(|| String::from("\n# No invariant found"));

    let mut code_analysis: Vec<_> = source_code.lines().collect();
    code_analysis.push(&program_inv);

    let invariants: BTreeMap<_, _> = invariants
        .into_iter()
        .map(|(pos, inv)| {
            let tabs = " ".repeat(pos.clm);
            (pos, format!("{tabs}# LOOP INVARIANT: {inv}"))
        })
        .collect();

    // Insert invariants in reverse order to preserve correct line positions
    for (pos, inv) in invariants.iter().rev() {
        code_analysis.insert(pos.line, inv);
    }
    code_analysis.join("\n")
}

pub fn extract_vars_init(source_code: &String) -> HashMap<&str, &str> {
    let assume_line = source_code.lines().next().unwrap_or_default();
    if !assume_line.contains("assume") || assume_line.contains("#") {
        return HashMap::new();
    }
    source_code.lines().next().unwrap_or("assume").trim()[6..]
        .split(';')
        .map(|assignment| {
            let mut parts = assignment.split(":=");
            (
                parts.next().unwrap().trim(),
                parts.next().unwrap_or_default().trim(),
            )
        })
        .collect::<HashMap<&str, &str>>()
}
