//! Legacy generator for coverage testing.
//! Uses Commands::gn and Memory::from_targets_with (original old generator logic).

use std::collections::BTreeMap;

use ce_core::Generate;
use gcl::ast::Commands;
use gcl::memory::Memory;
use itertools::Itertools;
use rand::prelude::IndexedRandom;
use rand::Rng;

use crate::{flow, Input, SecurityLatticeInput};
use stdx::stringify::Stringify;

/// Generate input using the original old generator: Commands::gn + lattice options + Memory::from_targets_with.
pub fn generate_input<R: Rng>(rng: &mut R) -> Input {
    let commands = Commands::gn(&mut Default::default(), rng);

    let lattice_options: [Vec<_>; 6] = [
        vec![flow("public", "private")],
        vec![
            flow("unclassified", "classified"),
            flow("classified", "secret"),
            flow("secret", "top_secret"),
        ],
        vec![flow("trusted", "dubious")],
        vec![
            flow("known_facts", "conjecture"),
            flow("conjecture", "alternative_facts"),
        ],
        vec![flow("low", "high")],
        vec![
            flow("clean", "Facebook"),
            flow("clean", "Google"),
            flow("clean", "Microsoft"),
        ],
    ];

    let lattice = SecurityLatticeInput {
        rules: lattice_options.choose(rng).unwrap().clone(),
    };
    let classes: Vec<String> = lattice
        .rules
        .iter()
        .flat_map(|f| [f.from.clone(), f.into.clone()])
        .sorted()
        .dedup()
        .collect_vec();

    let classification: BTreeMap<String, String> = Memory::from_targets_with(
        commands.fv(),
        rng,
        |rng, _| classes.choose(rng).unwrap().clone(),
        |rng, _| classes.choose(rng).unwrap().clone(),
    )
    .iter()
    .map(|r| (r.target().name().to_string(), r.value().clone()))
    .collect();

    Input {
        commands: Stringify::new(commands),
        classification,
        lattice,
    }
}
