use ce_core::rand;

pub fn generate_grammar_sentence<R: rand::Rng>(rng: &mut R) -> String {
    let subjects = [
        "Rust", "The compiler", "My program", "The developer",
        "Your function", "This input", "The system", "Our code",
    ];

    let verbs = [
        "optimizes", "compiles", "tests", "parses", "transforms",
        "evaluates", "breaks", "improves", "analyzes",
    ];

    let objects = [
        "the expression", "the syntax tree", "the input string",
        "the random generator", "the output", "the module",
    ];

    let adjectives = [
        "unexpected", "beautiful", "chaotic", "complex",
        "elegant", "mysterious", "strange", "fascinating",
    ];

    let adverbs = [
        "quickly", "silently", "surprisingly", "boldly",
        "randomly", "gracefully", "unexpectedly",
    ];

    let clauses = [
        "because the borrow checker said so",
        "while avoiding undefined behavior",
        "even though nobody asked",
        "as part of a mysterious optimization",
        "to satisfy the type system",
    ];

    // Helper macro to pick a random element without `.choose()`
    macro_rules! pick {
        ($arr:expr) => {
            $arr[rng.random_range(0..$arr.len())]
        };
    }

    match rng.random_range(0..5) {
        0 => format!("{} {} {} {}.", pick!(subjects), pick!(verbs), pick!(objects), pick!(adverbs)),
        1 => format!("{} {} the {} {}.", pick!(subjects), pick!(verbs), pick!(adjectives), pick!(objects)),
        2 => format!("{} {} {} {} {}.", pick!(subjects), pick!(adverbs), pick!(verbs), pick!(adjectives), pick!(objects)),
        3 => format!("{} {} {} {}.", pick!(subjects), pick!(verbs), pick!(objects), pick!(clauses)),
        _ => format!("{} {} {} while {} {}.", pick!(subjects), pick!(verbs), pick!(objects), pick!(subjects), pick!(verbs)),
    }
}
