use ce_core::{Env, Generate, ValidationResult, define_env, rand};
use serde::{Deserialize, Serialize};
use stdx::stringify::Stringify;


pub mod types;
pub mod command;
pub mod gradebook;
pub mod generator;


use crate::{
    gradebook::GradeBook,
    command::{parse_command, Command, ParseError},
    types::DANISH_GRADES,
};

define_env!(Challenge2Env);

#[derive(tapi::Tapi, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[tapi(path = "Challenge2")]
pub struct Input {
    pub command: Stringify<String>,
}

#[derive(tapi::Tapi, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[tapi(path = "Challenge2")]
pub struct Output {
    pub result: String,
}

impl Env for Challenge2Env {
    type Input = Input;
    type Output = Output;
    type Meta = ();

    fn run(input: &Self::Input) -> ce_core::Result<Self::Output> {
        let parsed = input.command.try_parse().unwrap();
        let lines = parsed.lines();

        let mut gradebook = GradeBook::new();
        let mut result = String::new();

        for line in lines {
            if line.trim().is_empty() {
                continue;
            }

            match parse_command(line) {
                Ok(Command::Assign { grade, student, subject }) => {
                    gradebook.assign(grade, student, subject);
                    result.push_str("Grade assigned successfully.\n");
                }
                Ok(Command::ShowSubject(subject)) => {
                    result.push_str(&gradebook.show_subject(&subject));
                }
                Ok(Command::ShowStudent(student)) => {
                    result.push_str(&gradebook.show_student(&student));
                }
                Ok(Command::RemoveGrade { grade, student, subject }) => {
                    if gradebook.remove_grade(grade, &student, &subject) {
                        result.push_str("Grade removed successfully.\n");
                    } else {
                        result.push_str("Grade not found.\n");
                    }
                }

                Ok(Command::AverageStudent(student)) => {
                    match gradebook.average_student(&student) {
                        Some(avg) => result.push_str(&format!("Average for {}: {:.2}\n", student, avg)),
                        None => result.push_str(&format!("No grades for student '{}'\n", student)),
                    }
                }

                Ok(Command::AverageSubject(subject)) => {
                    match gradebook.average_subject(&subject) {
                        Some(avg) => result.push_str(&format!("Average for {}: {:.2}\n", subject, avg)),
                        None => result.push_str(&format!("No grades for subject '{}'\n", subject)),
                    }
                }

                Err(err) => match err {
                    ParseError::InvalidFormat(raw) =>
                        result.push_str(&format!("Invalid format: '{}'\n", raw)),
                    ParseError::InvalidGrade(g) =>
                        result.push_str(&format!("Invalid Danish grade '{}'. Valid grades: {:?}\n", g, DANISH_GRADES)),
                    ParseError::UnknownCommand(raw) =>
                        result.push_str(&format!("Unknown command: '{}'\n", raw)),
                }
            }
        }

        Ok(Output { result })
    }

    fn validate(_: &Self::Input, _: &Self::Output) -> ce_core::Result<ValidationResult> {
        Ok(ValidationResult::Correct)
    }
}

impl Generate for Input {
    type Context = ();

    fn gn<R: rand::Rng>(_cx: &mut Self::Context, rng: &mut R) -> Self {
        let script = crate::generator::generate_script(rng);
        
        Self {
            command: Stringify::Unparsed(script),
        }
    }
}

#[cfg(test)]
mod tests;
