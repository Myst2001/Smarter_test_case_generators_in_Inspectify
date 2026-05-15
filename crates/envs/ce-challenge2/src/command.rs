use crate::types::*;

#[derive(Debug)]
pub enum Command {
    Assign {
        grade: Grade,
        student: Student,
        subject: Subject,
    },
    
    ShowSubject(Subject),
    ShowStudent(Student),

    RemoveGrade {
        grade: Grade,
        student: Student,
        subject: Subject,
    },

    AverageStudent(Student),
    AverageSubject(Subject),
}


#[derive(Debug)]
pub enum ParseError {
    InvalidFormat(String),
    InvalidGrade(String),
    UnknownCommand(String),
}

pub fn parse_command(line: &str) -> Result<Command, ParseError> {
    let trimmed = line.trim();
    let lower = trimmed.to_lowercase();
    let parts: Vec<&str> = trimmed.split_whitespace().collect();

    if lower.starts_with("grade ") {
        if parts.len() < 6 {
            return Err(ParseError::InvalidFormat(trimmed.to_string()));
        }

        let grade_raw = parts[1];
        let grade: Grade = grade_raw
            .parse()
            .map_err(|_| ParseError::InvalidGrade(grade_raw.to_string()))?;

        if !is_valid_danish_grade(grade) {
            return Err(ParseError::InvalidGrade(grade_raw.to_string()));
        }

        let student = parts[3].to_string();
        let subject = parts[5].to_string();

        return Ok(Command::Assign { grade, student, subject });
    }

    if lower.starts_with("show subject ") {
        let subject = trimmed[13..].trim().to_string();
        if subject.is_empty() {
            return Err(ParseError::InvalidFormat(trimmed.to_string()));
        }
        return Ok(Command::ShowSubject(subject));
    }

    if lower.starts_with("show student ") {
        let student = trimmed[13..].trim().to_string();
        if student.is_empty() {
            return Err(ParseError::InvalidFormat(trimmed.to_string()));
        }
        return Ok(Command::ShowStudent(student));
    }

    // remove grade <grade> from <student> at <subject>
    if lower.starts_with("remove grade ") {
        if parts.len() < 7 {
            return Err(ParseError::InvalidFormat(trimmed.to_string()));
        }

        let grade_raw = parts[2];
        let grade: Grade = grade_raw
        .parse()
        .map_err(|_| ParseError::InvalidGrade(grade_raw.to_string()))?;

        if !is_valid_danish_grade(grade) {
            return Err(ParseError::InvalidGrade(grade_raw.to_string()));
        }

        let student = parts[4].to_string();
        let subject = parts[6].to_string();

        return Ok(Command::RemoveGrade { grade, student, subject });
    }

    // average student <name>
    if lower.starts_with("average student ") {
        let student = trimmed[16..].trim().to_string();
        if student.is_empty() {
            return Err(ParseError::InvalidFormat(trimmed.to_string()));
        }
        return Ok(Command::AverageStudent(student));
    }

    // average subject <subject>
    if lower.starts_with("average subject ") {
        let subject = trimmed[16..].trim().to_string();
        if subject.is_empty() {
            return Err(ParseError::InvalidFormat(trimmed.to_string()));
        }
        return Ok(Command::AverageSubject(subject));
    }

    Err(ParseError::UnknownCommand(trimmed.to_string()))
}
