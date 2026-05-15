use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use gcl::ast::*;
use indexmap::IndexSet;
use rand::Rng;

use crate::{analysis::{Security, SecurityLattice}, flow, SecurityLatticeInput};

// ─── AST helpers ─────────────────────────────────────────────────────────────

fn var(s: &str) -> Variable {
    Variable(s.to_string())
}
fn tgt(s: &str) -> Target<Box<AExpr>> {
    Target::Variable(var(s))
}
fn arr(s: &str) -> Array {
    Array(s.to_string())
}
fn tgt_arr(name: &str, idx: AExpr) -> Target<Box<AExpr>> {
    Target::Array(arr(name), Box::new(idx))
}
fn aref(s: &str) -> AExpr {
    AExpr::Reference(tgt(s))
}
fn aref_arr(name: &str, idx: AExpr) -> AExpr {
    AExpr::Reference(tgt_arr(name, idx))
}
fn num(n: i32) -> AExpr {
    AExpr::Number(n)
}
fn assign(lhs: &str, rhs: AExpr) -> Command {
    Command::Assignment(tgt(lhs), rhs)
}
fn assign_arr(arr_name: &str, idx: AExpr, rhs: AExpr) -> Command {
    Command::Assignment(tgt_arr(arr_name, idx), rhs)
}
fn guard(b: BExpr, v: Vec<Command>) -> Guard {
    Guard(b, Commands(v))
}
pub(crate) fn skip() -> Command {
    Command::Skip
}
fn brel(lhs: &str, op: RelOp, rhs: i32) -> BExpr {
    BExpr::Rel(aref(lhs), op, num(rhs))
}
fn brel_expr(lhs: AExpr, op: RelOp, rhs: AExpr) -> BExpr {
    BExpr::Rel(lhs, op, rhs)
}

const VARS: [&str; 6] = ["a", "b", "c", "x", "y", "z"];
const ARRAY_NAMES: [&str; 2] = ["arr", "buf"];

fn shuffle<T, R: Rng>(rng: &mut R, s: &mut [T]) {
    for i in 0..s.len() {
        s.swap(i, rng.random_range(i..s.len()));
    }
}

fn pick_vars<R: Rng>(rng: &mut R, min: usize, max: usize) -> Vec<String> {
    let n = rng.random_range(min.max(1).min(max)..=max.min(VARS.len()));
    let mut pool: Vec<String> = VARS.iter().map(|s| s.to_string()).collect();
    shuffle(rng, &mut pool);
    pool.truncate(n);
    pool
}

// ─── Lattices ────────────────────────────────────────────────────────────────

const LATTICE_DEFS: &[&[(&str, &str)]] = &[
    &[("public", "private")],
    &[("public", "internal"), ("internal", "private")],
    &[("public", "confidential"), ("confidential", "private")],
    &[("low", "medium"), ("medium", "high")],
    &[("unclassified", "classified"), ("classified", "secret"), ("secret", "top_secret")],
    &[("public", "financial"), ("financial", "medical"), ("medical", "top_secret")],
    &[("dubious", "trusted")],
    &[("alternative_fact", "conjecture"), ("conjecture", "known_fact")],
    &[("unverified", "verified"), ("verified", "certified")],
    &[("low_trust", "medium_trust"), ("medium_trust", "high_trust")],
    &[("known_facts", "conjecture"), ("conjecture", "alternative_facts")],
    &[("public", "Alice"), ("Alice", "Bob"), ("Bob", "secret")],
    // Diamond: two incomparable paths merge — exercises join behaviour.
    &[("public", "shared"), ("shared", "Alice"), ("shared", "Bob")],
    &[("public", "TeamA"), ("TeamA", "TeamB"), ("TeamB", "shared")],
    &[("public", "Dept1"), ("Dept1", "Dept2"), ("Dept2", "Dept3"), ("Dept3", "shared")],
    &[("clean", "Facebook"), ("clean", "Google"), ("clean", "Microsoft")],
];

fn pick_lattice<R: Rng>(rng: &mut R) -> SecurityLatticeInput {
    let edges = LATTICE_DEFS[rng.random_range(0..LATTICE_DEFS.len())];
    SecurityLatticeInput {
        rules: edges.iter().map(|(a, b)| flow(a, b)).collect(),
    }
}

fn lattice_classes(lattice: &SecurityLatticeInput) -> Vec<String> {
    lattice.rules.iter()
        .flat_map(|f| [f.from.clone(), f.into.clone()])
        .collect::<BTreeSet<_>>().into_iter().collect()
}

/// Returns the bottom (least) element of the lattice — the class that every
/// other class is allowed to flow into.  Falls back to the first class.
fn lattice_bottom(lattice: &SecurityLatticeInput) -> String {
    let classes = lattice_classes(lattice);
    let security_lattice = SecurityLattice::new(&lattice.rules);
    classes.iter()
        .find(|c| classes.iter().all(|other| {
            other == *c || security_lattice.allows(&crate::flow(c, other))
        }))
        .cloned()
        .unwrap_or_else(|| classes[0].clone())
}

/// Returns the top (greatest) element.
fn lattice_top(lattice: &SecurityLatticeInput) -> String {
    let classes = lattice_classes(lattice);
    let security_lattice = SecurityLattice::new(&lattice.rules);
    classes.iter()
        .find(|c| classes.iter().all(|other| {
            other == *c || security_lattice.allows(&crate::flow(other, c))
        }))
        .cloned()
        .unwrap_or_else(|| classes[classes.len() - 1].clone())
}

// ─── Random command builders ─────────────────────────────────────────────────

fn rand_idx_expr<R: Rng>(vars: &[String], rng: &mut R) -> AExpr {
    aref(&vars[rng.random_range(0..vars.len())])
}

fn rand_bool<R: Rng>(vars: &[String], arrays: &[String], depth: usize, rng: &mut R) -> BExpr {
    if depth == 0 {
        return brel(&vars[rng.random_range(0..vars.len())], RelOp::Gt, rng.random_range(-5..=5));
    }

    // Array index comparisons — exercises implicit flows through array reads in guards.
    if !arrays.is_empty() && rng.random_range(0.0f32..1.0) < 0.15 {
        let arr_name = &arrays[rng.random_range(0..arrays.len())];
        return brel_expr(
            aref_arr(arr_name, rand_idx_expr(vars, rng)),
            if rng.random() { RelOp::Gt } else { RelOp::Le },
            if rng.random() { aref(&vars[rng.random_range(0..vars.len())]) } else { num(0) },
        );
    }

    match rng.random_range(0..4) {
        0 => brel(&vars[rng.random_range(0..vars.len())], RelOp::Gt, rng.random_range(-5..=5)),
        1 => brel_expr(
            aref(&vars[rng.random_range(0..vars.len())]),
            RelOp::Gt,
            aref(&vars[rng.random_range(0..vars.len())])
        ),
        2 => BExpr::Not(Box::new(rand_bool(vars, arrays, depth - 1, rng))),
        _ => BExpr::Logic(
            Box::new(rand_bool(vars, arrays, depth - 1, rng)),
            LogicOp::And,
            Box::new(rand_bool(vars, arrays, depth - 1, rng))
        ),
    }
}

fn rand_assign<R: Rng>(vars: &[String], arrays: &[String], rng: &mut R) -> Command {
    // Increased array probability: 25 % (was 10 %).
    if !arrays.is_empty() && rng.random_range(0.0f32..1.0) < 0.25 {
        let arr_name = &arrays[rng.random_range(0..arrays.len())];
        // Occasionally use a *variable* as the index so the index value itself
        // becomes a security-relevant implicit channel.
        let idx = if rng.random_range(0.0f32..1.0) < 0.40 {
            rand_idx_expr(vars, rng)
        } else {
            num(rng.random_range(0..=3))
        };
        let rhs = match rng.random_range(0..4) {
            0 => aref(&vars[rng.random_range(0..vars.len())]),
            1 => AExpr::Binary(
                Box::new(aref(&vars[rng.random_range(0..vars.len())])),
                AOp::Plus,
                Box::new(aref(&vars[rng.random_range(0..vars.len())])),
            ),
            // Read from array at a variable index — captures arr[secret] as a source.
            2 => aref_arr(arr_name, rand_idx_expr(vars, rng)),
            _ => num(rng.random_range(-5..=5)),
        };
        return assign_arr(arr_name, idx, rhs);
    }

    let lhs = &vars[rng.random_range(0..vars.len())];
    let rhs = match rng.random_range(0..5) {
        0 => aref(&vars[rng.random_range(0..vars.len())]),
        1 => num(rng.random_range(-5..=5)),
        2 => AExpr::Binary(
            Box::new(aref(&vars[rng.random_range(0..vars.len())])),
            if rng.random() { AOp::Plus } else { AOp::Minus },
            Box::new(num(rng.random_range(1..=3)))
        ),
        3 if !arrays.is_empty() => {
            // Variable index read: implicit flow from the index variable into lhs.
            aref_arr(&arrays[rng.random_range(0..arrays.len())], rand_idx_expr(vars, rng))
        }
        _ => AExpr::Binary(
            Box::new(aref(&vars[rng.random_range(0..vars.len())])),
            AOp::Plus,
            Box::new(aref(&vars[rng.random_range(0..vars.len())]))
        ),
    };
    assign(lhs, rhs)
}

fn rand_guards<R: Rng>(
    vars: &[String],
    arrays: &[String],
    depth: usize,
    count: std::ops::RangeInclusive<usize>,
    rng: &mut R
) -> Vec<Guard> {
    (0..rng.random_range(count))
        .map(|i| {
            let b = if i > 0 && rng.random() {
                BExpr::Bool(false)
            } else {
                rand_bool(vars, arrays, 1.min(depth), rng)
            };
            guard(b, vec![rand_cmd(vars, arrays, depth.saturating_sub(1), rng)])
        })
        .collect()
}

fn rand_cmd<R: Rng>(vars: &[String], arrays: &[String], depth: usize, rng: &mut R) -> Command {
    if depth == 0 {
        return rand_assign(vars, arrays, rng);
    }
    match rng.random_range(0..5) {
        0 => rand_cmd(vars, arrays, depth - 1, rng),
        1 => Command::If(rand_guards(vars, arrays, depth, 2..=3, rng)),
        2 => Command::Loop(rand_guards(vars, arrays, depth, 1..=2, rng)),
        3 => Command::Loop(vec![guard(
            rand_bool(vars, arrays, 1, rng),
            vec![Command::If(vec![
                guard(rand_bool(vars, arrays, 1, rng), vec![rand_cmd(vars, arrays, depth.saturating_sub(1), rng)]),
                guard(BExpr::Bool(false), vec![skip()])
            ])]
        )]),
        _ => {
            let body: Vec<Command> = (0..rng.random_range(1..=2))
                .map(|_| rand_cmd(vars, arrays, depth.saturating_sub(1), rng))
                .collect();
            Command::If(vec![
                guard(rand_bool(vars, arrays, 1, rng), body),
                guard(BExpr::Not(Box::new(rand_bool(vars, arrays, 1, rng))), vec![skip()])
            ])
        }
    }
}

// ─── Seeded patterns ─────────────────────────────────────────────────────────
//
// These build structurally interesting programs that are guaranteed to contain
// a specific kind of flow so the classifier always sees non-trivial cases.

/// high_var → if-guard → low_var assigned in both branches (explicit implicit flow).
fn seeded_implicit_guard(high_var: &str, low_var: &str, _vars: &[String]) -> Command {
        Command::If(vec![
        guard(brel(high_var, RelOp::Gt, 0), vec![assign(low_var, num(1))]),
        guard(brel(high_var, RelOp::Le, 0), vec![assign(low_var, num(0))]),
    ])
}

/// Loop where the guard reads high_var and the body writes low_var.
fn seeded_loop_guard(high_var: &str, low_var: &str) -> Command {
    Command::Loop(vec![guard(
        brel(high_var, RelOp::Gt, 0),
        vec![
            assign(low_var, aref(high_var)),
            assign(
                high_var,
                AExpr::Binary(Box::new(aref(high_var)), AOp::Minus, Box::new(num(1)))
            ),
        ],
    )])
}

/// arr[high_idx] = low_val — index is the security-relevant channel.
fn seeded_array_idx_write(arr_name: &str, high_idx: &str, low_val: &str) -> Command {
    assign_arr(arr_name, aref(high_idx), aref(low_val))
}

/// low_var = arr[high_idx] — reading through a secret index.
fn seeded_array_idx_read(target: &str, arr_name: &str, high_idx: &str) -> Command {
    assign(target, aref_arr(arr_name, aref(high_idx)))
}

// ─── Program generators ──────────────────────────────────────────────────────

fn pick_vars_3_5_or_5_6<R: Rng>(rng: &mut R) -> Vec<String> {
    if rng.random_range(0.0f32..1.0) < 0.80 {
        pick_vars(rng, 3, 5)
    } else {
        pick_vars(rng, 5, 6)
    }
}

fn pick_arrays<R: Rng>(rng: &mut R) -> Vec<String> {
    // Increased probability: 25 % (was 12 %).
    if rng.random_range(0.0f32..1.0) < 0.25 {
        ARRAY_NAMES.iter().map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}

fn generate_medium<R: Rng>(rng: &mut R) -> Commands {
    let vars = pick_vars_3_5_or_5_6(rng);
    let arrays = pick_arrays(rng);
    let nv = vars.len();
    let (a, b, c) = (&vars[0], &vars[1 % nv], &vars[2 % nv]);
    let x = &vars[if nv >= 4 { 3 } else { nv - 1 }];
    let y = &vars[if nv >= 5 { 4 } else { 0 }];

    let mut cmds: Vec<Command> = vec![
        assign(x, aref(a)),
        assign(y, aref(b)),
        Command::If(vec![
            guard(brel(a, RelOp::Gt, 0), vec![Command::If(vec![
                guard(brel(b, RelOp::Gt, 0), vec![assign(c, num(1))]),
                guard(brel(b, RelOp::Le, 0), vec![assign(c, num(0))])
            ])]),
            guard(brel(a, RelOp::Le, 0), vec![assign(c, num(-1))])
        ]),
        Command::Loop(vec![guard(
            brel(a, RelOp::Gt, 0),
            vec![
                assign(x, aref(y)),
                assign(a, AExpr::Binary(Box::new(aref(a)), AOp::Minus, Box::new(num(1))))
            ]
        )])
    ];

    while cmds.len() < rng.random_range(8..=15) {
        match rng.random_range(0..5) {
            0 => {
                let (l, r) = (
                    &vars[rng.random_range(0..vars.len())],
                    &vars[rng.random_range(0..vars.len())]
                );
                cmds.push(if l != r {
                    assign(l, aref(r))
                } else {
                    assign(l, num(rng.random_range(-3..=3)))
                });
            }
            1 => cmds.push(rand_assign(&vars, &arrays, rng)),
            2 => {
                let (g, t) = (
                    &vars[rng.random_range(0..vars.len())],
                    &vars[rng.random_range(0..vars.len())]
                );
                cmds.push(Command::If(vec![
                    guard(brel(g, RelOp::Gt, 0), vec![assign(t, num(1))]),
                    guard(brel(g, RelOp::Le, 0), vec![skip()])
                ]));
            }
            // Seeded array patterns so they appear in medium programs too.
            3 if !arrays.is_empty() => {
                let arr_name = &arrays[0];
                let idx_var = &vars[rng.random_range(0..vars.len())];
                let val_var = &vars[rng.random_range(0..vars.len())];
                cmds.push(seeded_array_idx_read(val_var, arr_name, idx_var));
            }
            _ => break,
        }
    }
    Commands(cmds)
}

fn generate_big<R: Rng>(rng: &mut R) -> Commands {
    let vars = if rng.random_range(0.0f32..1.0) < 0.80 {
        pick_vars(rng, 3, 5)
    } else {
        pick_vars(rng, 4, 6)
    };
    let arrays = pick_arrays(rng);
    let depth = rng.random_range(2..=3);
    Commands(
        (0..rng.random_range(2..=3))
            .map(|_| rand_cmd(&vars, &arrays, depth, rng))
            .collect()
    )
}

// ─── Classification ──────────────────────────────────────────────────────────

fn var_names(commands: &Commands) -> Vec<String> {
    commands
        .fv()
        .iter()
        .map(|t| t.name().to_string())
        .collect::<IndexSet<_>>()
        .into_iter()
        .collect()
}

fn usage_counts(commands: &Commands) -> (BTreeMap<String, usize>, BTreeMap<String, usize>) {
    let (mut lhs, mut guards) = (BTreeMap::new(), BTreeMap::new());

    fn walk(
        cmds: &Commands,
        lhs: &mut BTreeMap<String, usize>,
        guards: &mut BTreeMap<String, usize>
    ) {
        for cmd in &cmds.0 {
            match cmd {
                Command::Assignment(t, _) => {
                    *lhs.entry(t.name().to_string()).or_default() += 1
                }
                Command::Skip => {}
                Command::If(gs) | Command::Loop(gs) => {
                    for Guard(b, inner) in gs {
                        for v in b.fv() {
                            *guards.entry(v.name().to_string()).or_default() += 1;
                        }
                        walk(inner, lhs, guards);
                    }
                }
            }
        }
    }

    walk(commands, &mut lhs, &mut guards);
    (lhs, guards)
}

/// Result of `build_classification` — carries whether a security violation was
/// actually injected so callers can assert on it rather than silently getting a
/// clean classification when `force_violation` is true but the search failed.
#[derive(Debug)]
pub struct Classification {
    pub map: BTreeMap<String, String>,
    /// True iff a violation was successfully injected when `force_violation` was set.
    pub violation_injected: bool,
}

pub fn build_classification<R: Rng>(
    commands: &Commands,
    lattice: &SecurityLatticeInput,
    rng: &mut R,
    force_violation: bool
) -> Classification {
    let mut classes = lattice_classes(lattice);
    if classes.len() < 2 {
        classes = vec!["low".into(), "high".into()];
    }

    let program_vars = var_names(commands);
    let (lhs_counts, guard_counts) = usage_counts(commands);
    let k = rng.random_range(3..=6).min(program_vars.len()).max(1);

    let mut selected: Vec<String> = program_vars.iter().cloned().collect();
    shuffle(rng, &mut selected);
    selected.truncate(k);

    selected.sort_by(|a, b| {
        let (la, lb) = (
            lhs_counts.get(a).copied().unwrap_or(0),
            lhs_counts.get(b).copied().unwrap_or(0)
        );
        let (ga, gb) = (
            guard_counts.get(a).copied().unwrap_or(0),
            guard_counts.get(b).copied().unwrap_or(0)
        );
        lb.cmp(&la).then(ga.cmp(&gb))
    });

    let mut levels = classes.clone();
    shuffle(rng, &mut levels);

    let n_levels = levels.len();
    let distinct = n_levels.min(k);
    let chosen: Vec<String> = levels.into_iter().take(distinct).collect();

    let mut counts: BTreeMap<String, usize> =
        chosen.iter().cloned().map(|l| (l, 0)).collect();
    let mut map = BTreeMap::new();

    for (i, v) in selected.iter().enumerate().take(distinct) {
        let lvl = chosen[i].clone();
        *counts.get_mut(&lvl).unwrap() += 1;
        map.insert(v.clone(), lvl);
    }

    let remaining: Vec<String> = selected
        .iter()
        .filter(|v| !map.contains_key(*v))
        .cloned()
        .collect();

    for v in remaining {
        let min_c = counts.values().copied().min().unwrap_or(0);
        let candidates: Vec<String> = counts
            .iter()
            .filter(|(_, c)| **c == min_c)
            .map(|(l, _)| l.clone())
            .collect();
        let lvl = candidates[rng.random_range(0..candidates.len().max(1))].clone();
        *counts.get_mut(&lvl).unwrap() += 1;
        map.insert(v, lvl);
    }

    if force_violation {
        let security_lattice = SecurityLattice::new(&lattice.rules);
        let classified: BTreeSet<_> = map.keys().cloned().collect();

        let mut candidates: Vec<_> = commands
            .flows()
            .into_iter()
            .filter(|f| {
                f.from != f.into
                    && classified.contains(&f.from)
                    && classified.contains(&f.into)
            })
            .collect();

        shuffle(rng, &mut candidates);

        for f in &candidates {
            for _ in 0..30 {
                let (c_from, c_into) = (
                    &classes[rng.random_range(0..classes.len())],
                    &classes[rng.random_range(0..classes.len())]
                );
                if !security_lattice.allows(&crate::flow(c_from, c_into)) {
                    map.insert(f.from.clone(), c_from.clone());
                    map.insert(f.into.clone(), c_into.clone());
                    return Classification { map, violation_injected: true };
                }
            }
        }

        // Fallback: directly assign top to a source and bottom to a sink.
        // This is guaranteed to produce a violation as long as top ≠ bottom.
        let top = lattice_top(lattice);
        let bottom = lattice_bottom(lattice);
        if top != bottom && !candidates.is_empty() {
            let f = &candidates[0];
            map.insert(f.from.clone(), top);
            map.insert(f.into.clone(), bottom);
            return Classification { map, violation_injected: true };
        }

        // Could not inject: log a warning and return clean.
        eprintln!(
            "[generator] WARNING: force_violation=true but no violating assignment could be \
             found for this program/lattice combination. Returning clean classification. \
             Lattice has {} classes, program has {} flows.",
            classes.len(),
            commands.flows().len()
        );
        return Classification { map, violation_injected: false };
    }

    Classification { map, violation_injected: false }
}

// ─── Template loading ────────────────────────────────────────────────────────

fn templates_dir() -> PathBuf {
    std::env::var("CE_SECURITY_TEMPLATES_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("src")
                .join("templates")
        })
}

fn load_template_commands() -> Vec<Commands> {
    let dir = templates_dir();
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Vec::new();
    };

    let mut result = Vec::new();

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("txt") {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(&path) else {
            continue;
        };
        if let Ok(cmds) = gcl::parse::parse_commands(&src) {
            result.push(cmds);
        }
    }

    result
}

// ─── Public API ──────────────────────────────────────────────────────────────

pub fn generate_commands<R: Rng>(rng: &mut R) -> Commands {
    if rng.random_range(0.0f32..1.0) < 0.30 {
        let templates = load_template_commands();
        if !templates.is_empty() {
            return templates[rng.random_range(0..templates.len())].clone();
        }
    }
    if rng.random() {
        generate_medium(rng)
    } else {
        generate_big(rng)
    }
}

pub fn generate_input<R: Rng>(rng: &mut R) -> crate::Input {
    let commands = generate_commands(rng);
    let lattice = pick_lattice(rng);
    let force_violation = rng.random_range(0.0f32..1.0) < 0.40;
    let classification = build_classification(&commands, &lattice, rng, force_violation);

    if force_violation && !classification.violation_injected {
        // Surface this prominently so test harnesses can track the miss rate.
        eprintln!("[generator] generate_input: violation requested but not achieved.");
    }

    crate::Input {
        commands: stdx::stringify::Stringify::new(commands),
        classification: classification.map,
        lattice,
    }
}

/// Generates a program that is *guaranteed* to contain a specific, named kind
/// of security violation — useful for targeted regression tests.
pub fn generate_input_seeded_violation<R: Rng>(rng: &mut R) -> crate::Input {
    let vars = pick_vars(rng, 3, 5);
    let arrays: Vec<String> = ARRAY_NAMES.iter().map(|s| s.to_string()).collect();
    let nv = vars.len();

    // Pick two distinct variables: one will be classified high, one low.
    let high_var = &vars[0];
    let low_var  = &vars[1 % nv];
    let mid_var  = &vars[2 % nv];

    // Choose the seeding pattern at random so the suite covers several shapes.
    let cmds = match rng.random_range(0..4) {
        // Pattern A: high leaks into low via if-guard (classic implicit flow).
        0 => vec![
            assign(high_var, num(rng.random_range(1..=5))),
            seeded_implicit_guard(high_var, low_var, &vars),
        ],
        // Pattern B: high leaks into low via loop guard.
        1 => vec![
            assign(high_var, num(rng.random_range(1..=5))),
            seeded_loop_guard(high_var, low_var),
        ],
        // Pattern C: arr[high_idx] = low_val — secret index channel.
        2 => vec![
            assign(high_var, num(rng.random_range(0..=3))),
            assign(low_var, num(rng.random_range(-5..=5))),
            seeded_array_idx_write(&arrays[0], high_var, low_var),
        ],
        // Pattern D: low_var = arr[high_idx] then low_var is used visibly.
        _ => vec![
            assign(high_var, num(rng.random_range(0..=3))),
            seeded_array_idx_read(mid_var, &arrays[0], high_var),
            assign(low_var, aref(mid_var)),
        ],
    };

    let commands = Commands(cmds);
    let lattice = pick_lattice(rng);
    let top = lattice_top(&lattice);
    let bottom = lattice_bottom(&lattice);

    // Classify high_var as top, low_var as bottom — guaranteed violation.
    let mut map = BTreeMap::new();
    for v in var_names(&commands) {
        map.insert(v.clone(), bottom.clone());
    }
    map.insert(high_var.to_string(), top);
    // low_var stays at bottom.

    crate::Input {
        commands: stdx::stringify::Stringify::new(commands),
        classification: map,
        lattice,
    }
}

/// Generates a simple program where the lattice structure is respected:
/// variables written first get lower labels, variables read from them get
/// higher labels — producing a *clean* (non-violating) classification.
pub fn generate_input_simple<R: Rng>(rng: &mut R) -> crate::Input {
    let vars = pick_vars(rng, 2, 4);
    let lattice = pick_lattice(rng);
    let classes = lattice_classes(&lattice);
    let security_lattice = SecurityLattice::new(&lattice.rules);

    // Assign each var a random class, then build a program that only flows
    // from lower-or-equal classes to higher-or-equal classes (clean program).
    let mut var_class: BTreeMap<String, String> = BTreeMap::new();
    for v in &vars {
        var_class.insert(v.clone(), classes[rng.random_range(0..classes.len())].clone());
    }

    let mut cmds: Vec<Command> = Vec::new();

    for _ in 0..rng.random_range(3..=6) {
        match rng.random_range(0..3) {
            0 => {
                // Safe assignment: rhs class ≤ lhs class.
                let lhs = &vars[rng.random_range(0..vars.len())];
                let lhs_class = &var_class[lhs].clone();
                // Find a rhs whose class allows flowing into lhs_class.
                let safe_rhs: Vec<&String> = vars.iter().filter(|rhs| {
                    let rc = &var_class[*rhs];
                    security_lattice.allows(&crate::flow(rc, lhs_class))
                }).collect();
                if safe_rhs.is_empty() {
                    cmds.push(assign(lhs, num(rng.random_range(-5..=5))));
                } else {
                    let rhs = safe_rhs[rng.random_range(0..safe_rhs.len())];
                    cmds.push(assign(lhs, aref(rhs)));
                }
            }
            1 => {
                let cond = &vars[rng.random_range(0..vars.len())];
                let tgt  = &vars[rng.random_range(0..vars.len())];
                let thresh = rng.random_range(-3..=3);
                // Only emit the if when cond's class ≤ tgt's class (no implicit flow upward).
                let cond_class = &var_class[cond].clone();
                let tgt_class  = &var_class[tgt].clone();
                if security_lattice.allows(&crate::flow(cond_class, tgt_class)) {
                    cmds.push(Command::If(vec![
                        guard(brel(cond, RelOp::Gt, thresh), vec![assign(tgt, num(rng.random_range(-5..=5)))]),
                        guard(brel(cond, RelOp::Le, thresh), vec![assign(tgt, num(rng.random_range(-5..=5)))]),
                    ]));
                } else {
                    cmds.push(assign(tgt, num(rng.random_range(-5..=5))));
                }
            }
            _ => {
                let cond      = &vars[rng.random_range(0..vars.len())];
                let body_var  = &vars[rng.random_range(0..vars.len())];
                let cond_class = &var_class[cond].clone();
                let bv_class   = &var_class[body_var].clone();
                if security_lattice.allows(&crate::flow(cond_class, bv_class)) {
                    cmds.push(Command::Loop(vec![guard(
                        brel(cond, RelOp::Gt, 0),
                        vec![
                            assign(body_var, aref(cond)),
                            assign(
                                cond,
                                AExpr::Binary(Box::new(aref(cond)), AOp::Minus, Box::new(num(1)))
                            ),
                        ],
                    )]));
                } else {
                    cmds.push(assign(body_var, num(0)));
                }
            }
        }
    }

    let commands = Commands(cmds);

    crate::Input {
        commands: stdx::stringify::Stringify::new(commands),
        classification: var_class,
        lattice,
    }
}

pub fn generate_input_new<R: Rng>(rng: &mut R) -> crate::Input {
    generate_input(rng)
}

pub fn generate_input_old<R: Rng>(rng: &mut R) -> crate::Input {
    generate_input_simple(rng)
}