use crate::{
    gradebook::GradeBook,
    command::{parse_command, Command, ParseError},
};


#[test]
fn test_parse_assign() {
    let cmd = parse_command("Grade 12 to Alice at Math").unwrap();

    match cmd {
        Command::Assign { grade, student, subject } => {
            assert_eq!(grade, 12);
            assert_eq!(student, "Alice");
            assert_eq!(subject, "Math");
        }
        _ => panic!("Parsed wrong command"),
    }
}

#[test]
fn test_parse_remove_grade() {
    let cmd = parse_command("remove grade 4 from Bob at Science").unwrap();

    match cmd {
        Command::RemoveGrade { grade, student, subject } => {
            assert_eq!(grade, 4);
            assert_eq!(student, "Bob");
            assert_eq!(subject, "Science");
        }
        _ => panic!("Parsed wrong command"),
    }
}

#[test]
fn test_parse_average_student() {
    let cmd = parse_command("average student Alice").unwrap();

    match cmd {
        Command::AverageStudent(student) => {
            assert_eq!(student, "Alice");
        }
        _ => panic!("Wrong command parsed"),
    }
}

#[test]
fn test_parse_average_subject() {
    let cmd = parse_command("average subject Math").unwrap();

    match cmd {
        Command::AverageSubject(subject) => {
            assert_eq!(subject, "Math");
        }
        _ => panic!("Wrong command parsed"),
    }
}

#[test]
fn test_parse_invalid_grade() {
    let err = parse_command("Grade 99 to Alice at Math").unwrap_err();

    match err {
        ParseError::InvalidGrade(g) => assert_eq!(g, "99"),
        _ => panic!("Expected InvalidGrade"),
    }
}

#[test]
fn test_parse_unknown_command() {
    let err = parse_command("dance around the room").unwrap_err();

    match err {
        ParseError::UnknownCommand(raw) => assert_eq!(raw, "dance around the room"),
        _ => panic!("Expected UnknownCommand"),
    }
}


#[test]
fn test_assign_and_show_student() {
    let mut gb = GradeBook::new();

    gb.assign(12, "Alice".into(), "Math".into());
    gb.assign(4, "Alice".into(), "Science".into());

    let output = gb.show_student("Alice");

    assert!(output.contains("Math"));
    assert!(output.contains("Science"));
    assert!(output.contains("12"));
    assert!(output.contains("4"));
}

#[test]
fn test_assign_and_show_subject() {
    let mut gb = GradeBook::new();

    gb.assign(12, "Alice".into(), "Math".into());
    gb.assign(4, "Bob".into(), "Math".into());

    let output = gb.show_subject("Math");

    assert!(output.contains("Alice"));
    assert!(output.contains("Bob"));
    assert!(output.contains("12"));
    assert!(output.contains("4"));
}


#[test]
fn test_remove_grade_success() {
    let mut gb = GradeBook::new();

    gb.assign(12, "Bob".into(), "Math".into());
    gb.assign(4, "Bob".into(), "Math".into());

    assert!(gb.remove_grade(12, "Bob", "Math"));
}

#[test]
fn test_remove_grade_nonexistent() {
    let mut gb = GradeBook::new();

    gb.assign(4, "Bob".into(), "Math".into());

    assert!(!gb.remove_grade(12, "Bob", "Math"));
}


#[test]
fn test_average_student() {
    let mut gb = GradeBook::new();

    gb.assign(12, "Alice".into(), "Math".into());
    gb.assign(4, "Alice".into(), "Science".into());

    let avg = gb.average_student("Alice").unwrap();
    assert!((avg - 8.0).abs() < f32::EPSILON);
}

#[test]
fn test_average_subject() {
    let mut gb = GradeBook::new();

    gb.assign(12, "Alice".into(), "Math".into());
    gb.assign(4, "Bob".into(), "Math".into());

    let avg = gb.average_subject("Math").unwrap();
    assert!((avg - 8.0).abs() < f32::EPSILON);
}

#[test]
fn test_average_student_no_grades() {
    let gb = GradeBook::new();

    assert!(gb.average_student("Alice").is_none());
}

#[test]
fn test_average_subject_no_grades() {
    let gb = GradeBook::new();

    assert!(gb.average_subject("Math").is_none());
}
