use crate::types::DANISH_GRADES;
use ce_core::rand;
use std::collections::HashMap;

pub fn generate_script<R: rand::Rng>(rng: &mut R) -> String {
    let names = ["Alice", "Bob", "Emma"];
    let subjects = ["Math", "Science", "English"];

    let mut script = String::new();
    let mut assigned: HashMap<(&'static str, &'static str), Vec<i8>> = HashMap::new();

    let assign_count = rng.random_range(6..=10);

    for _ in 0..assign_count {
        let grade = DANISH_GRADES[rng.random_range(0..DANISH_GRADES.len())];
        let student = names[rng.random_range(0..names.len())];
        let subject = subjects[rng.random_range(0..subjects.len())];

        assigned.entry((student, subject)).or_default().push(grade);

        script.push_str(&format!("Grade {} to {} at {}\n", grade, student, subject));
    }

    fn pick_student<'a, R: rand::Rng>(rng: &mut R, names: &'a [&'a str]) -> &'a str {
        names[rng.random_range(0..names.len())]
    }

    fn pick_subject<'a, R: rand::Rng>(rng: &mut R, subjects: &'a [&'a str]) -> &'a str {
        subjects[rng.random_range(0..subjects.len())]
    }

    fn pick_existing_pair<'a, R: rand::Rng>(
        rng: &mut R,
        assigned: &'a HashMap<(&'a str, &'a str), Vec<i8>>,
    ) -> Option<(&'a str, &'a str)> {
        if assigned.is_empty() {
            None
        } else {
            let keys: Vec<_> = assigned.keys().copied().collect();
            Some(keys[rng.random_range(0..keys.len())])
        }
    }

    let mixed_count = rng.random_range(5..=8);

    for _ in 0..mixed_count {
        let choice = rng.random_range(0..100);

        let line = match choice {
            // show student
            0..=19 => {
                let student = pick_student(rng, &names);
                format!("Show student {}", student)
            }

            // show subject
            20..=39 => {
                let subject = pick_subject(rng, &subjects);
                format!("Show subject {}", subject)
            }

            // remove grade
            40..=49 => {
                if let Some((student, subject)) = pick_existing_pair(rng, &assigned) {
                    let grades = &assigned[&(student, subject)];
                    let grade = grades[rng.random_range(0..grades.len())];
                    format!("Remove grade {} from {} at {}", grade, student, subject)
                } else {
                    // fallback
                    let grade = DANISH_GRADES[rng.random_range(0..DANISH_GRADES.len())];
                    let student = pick_student(rng, &names);
                    let subject = pick_subject(rng, &subjects);
                    format!("Remove grade {} from {} at {}", grade, student, subject)
                }
            }

            // average student
            50..=74 => {
                let student = pick_student(rng, &names);
                format!("Average student {}", student)
            }

            // average subject
            75..=94 => {
                let subject = pick_subject(rng, &subjects);
                format!("Average subject {}", subject)
            }

            // malformed
            _ => {
                let malformed = [
                    "Grade to Alice at Math",
                    "remove 12 from Bob",
                    "average",
                    "show",
                    "???",
                ];
                malformed[rng.random_range(0..malformed.len())].to_string()
            }
        };

        script.push_str(&line);
        script.push('\n');
    }

    script
}
