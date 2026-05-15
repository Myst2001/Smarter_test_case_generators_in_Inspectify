mod analysis;
pub mod generator;
pub mod generator_old;

pub use generator::{generate_input_new, generate_input_old};

use std::collections::{BTreeMap, BTreeSet};

use analysis::{Security, SecurityLattice};
use ce_core::{Env, Generate, ValidationResult, define_env, rand};
use gcl::ast::{Commands, Target, TargetDef, Variable};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use stdx::stringify::Stringify;

define_env!(SecurityEnv);

#[derive(
    tapi::Tapi, Debug, Default, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize,
)]
#[tapi(path = "SecurityAnalysis")]
pub struct Flow {
    pub from: String,
    pub into: String,
}
pub fn flow(from: impl ToString, to: impl ToString) -> Flow {
    Flow {
        from: from.to_string(),
        into: to.to_string(),
    }
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[tapi(path = "SecurityAnalysis")]
pub struct SecurityLatticeInput {
    pub rules: Vec<Flow>,
}

#[derive(tapi::Tapi, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[tapi(path = "SecurityAnalysis")]
pub struct Input {
    pub commands: Stringify<Commands>,
    pub classification: BTreeMap<String, String>,
    pub lattice: SecurityLatticeInput,
}

#[derive(tapi::Tapi, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[tapi(path = "SecurityAnalysis")]
pub struct Output {
    pub actual: Vec<Flow>,
    pub allowed: Vec<Flow>,
    pub violations: Vec<Flow>,
    pub is_secure: bool,
}

#[derive(tapi::Tapi, Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[tapi(path = "SecurityAnalysis")]
pub struct Meta {
    pub lattice: SecurityLattice,
    pub targets: BTreeSet<TargetDef>,
}

impl Env for SecurityEnv {
    type Input = Input;

    type Output = Output;

    type Meta = Meta;

    fn meta(input: &Self::Input) -> Self::Meta {
        let Ok(commands) =
            input
                .commands
                .try_parse()
                .map_err(ce_core::EnvError::invalid_input_for_program(
                    "failed to parse commands",
                ))
        else {
            return Default::default();
        };

        Meta {
            lattice: SecurityLattice::new(&input.lattice.rules),
            targets: commands.fv().into_iter().map(|t| t.def()).collect(),
        }
    }

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let commands =
            input
                .commands
                .try_parse()
                .map_err(ce_core::EnvError::invalid_input_for_program(
                    "failed to parse commands",
                ))?;

        let lattice = SecurityLattice::new(&input.lattice.rules);

        let actual = commands.flows();
        let allowed = lattice
            .all_allowed(
                &input
                    .classification
                    .iter()
                    .map(|(k, v)| (Target::Variable(Variable(k.clone())), v.clone()))
                    .collect(),
            )
            .collect_vec();
        let violations = actual
            .iter()
            .filter(|f| !allowed.contains(f))
            .cloned()
            .collect_vec();

        let is_secure = violations.is_empty();

        Ok(Output {
            actual: actual.into_iter().collect(),
            allowed: allowed.into_iter().collect(),
            violations,
            is_secure,
        })
    }

    fn validate(input: &Self::Input, output: &Self::Output) -> ce_core::Result<ValidationResult> {
        let refernce = Self::run(input)?;

        let compare_sets = |a: &[Flow], b: &[Flow]| {
            let a: BTreeSet<_> = a.iter().collect();
            let b: BTreeSet<_> = b.iter().collect();
            a == b
        };

        if !compare_sets(&output.actual, &refernce.actual) {
            return Ok(ValidationResult::Mismatch {
                reason: "actual flows does not match reference".to_string(),
            });
        }
        if !compare_sets(&output.allowed, &refernce.allowed) {
            return Ok(ValidationResult::Mismatch {
                reason: "allowed flows does not match reference".to_string(),
            });
        }
        if !compare_sets(&output.violations, &refernce.violations) {
            return Ok(ValidationResult::Mismatch {
                reason: "violations does not match reference".to_string(),
            });
        }
        if output.is_secure != refernce.is_secure {
            if refernce.is_secure {
                return Ok(ValidationResult::Mismatch {
                    reason: "expected secure, but got insecure".to_string(),
                });
            } else {
                return Ok(ValidationResult::Mismatch {
                    reason: "expected insecure, but got secure".to_string(),
                });
            }
        }

        Ok(ValidationResult::Correct)
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, rng: &mut R) -> Self {
        generator::generate_input(rng)
    }
}
