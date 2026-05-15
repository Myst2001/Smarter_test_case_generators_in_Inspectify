mod sentences;

use ce_core::{Env, Generate, ValidationResult, define_env, rand};
use serde::{Deserialize, Serialize};
use stdx::stringify::Stringify;
use sentences::generate_grammar_sentence;


define_env!(CapitalizeEnv);

#[derive(tapi::Tapi, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[tapi(path = "Capitalize")]
pub struct Input {
    pub text: Stringify<String>,
}

#[derive(tapi::Tapi, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[tapi(path = "Capitalize")]
pub struct Output {
    pub result: String,
    pub error: String,
}

impl Env for CapitalizeEnv {
    type Input = Input;

    type Output = Output;

    type Meta = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let parsed = input.text.try_parse().unwrap();
        let result = capitalize(&parsed);

        Ok(Output { result, error: String::new(), })
    }

    fn validate(_input: &Self::Input, _output: &Self::Output) -> ce_core::Result<ValidationResult> {
        Ok(ValidationResult::Correct)
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, rng: &mut R) -> Self {
        // 1. Start with a grammar-based sentence
        let mut sentence = generate_grammar_sentence(rng);

        // 2. Random chance to add emojis at start or end
        let emojis = ["🌎", "🔥", "✨", "🚀", "💡", "🎉", "😄"];
        if rng.random_bool(0.3) {
            let emoji = emojis[rng.random_range(0..emojis.len())];
            if rng.random_bool(0.5) {
                sentence = format!("{} {}", emoji, sentence);
            } else {
                sentence = format!("{} {}", sentence, emoji);
            }
        }

        // 3. Random chance to add symbols
        let symbols = ["1234", "!@#$", "..", "--", "++", "#tag"];
        if rng.random_bool(0.2) {
            let sym = symbols[rng.random_range(0..symbols.len())];
            sentence = format!("{}{}", sentence, sym);
        }

        // 4. Random chance to apply mixed-case chaos
        if rng.random_bool(0.1) {
            sentence = random_mixed_case(&sentence, rng);
        }

        // 5. Random chance to wrap in whitespace
        if rng.random_bool(0.1) {
            sentence = format!("   {}   ", sentence);
        }

        if rng.random_bool(0.01) {
            sentence = format!("\n{}\n", sentence);
        }
        if rng.random_bool(0.01) {
            sentence = format!("");
        }
        Self {
            text: Stringify::Unparsed(sentence),
        }
    }
}

fn random_mixed_case<R: rand::Rng>(input: &str, rng: &mut R) -> String {
    input
        .chars()
        .map(|c| {
            if rng.random_bool(0.5) {
                c.to_uppercase().to_string()
            } else {
                c.to_lowercase().to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("")
}




pub fn capitalize(input: &str) -> String {
    input
        .split_whitespace()
        .enumerate()
        .map(|(i, w)| {
            if i % 2 == 0 {
                w.to_uppercase()
            } else {
                w.to_string()   
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
