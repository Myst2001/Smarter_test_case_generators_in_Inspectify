use crate::types::*;
use std::collections::HashMap;

pub struct GradeBook {
    pub by_subject: HashMap<Subject, HashMap<Student, Vec<Grade>>>,
    pub by_student: HashMap<Student, HashMap<Subject, Vec<Grade>>>,
}

impl GradeBook {
    pub fn new() -> Self {
        Self {
            by_subject: HashMap::new(),
            by_student: HashMap::new(),
        }
    }

    pub fn assign(&mut self, grade: Grade, student: Student, subject: Subject) {
        self.by_subject
            .entry(subject.clone())
            .or_insert_with(HashMap::new)
            .entry(student.clone())
            .or_insert_with(Vec::new)
            .push(grade);

        self.by_student
            .entry(student)
            .or_insert_with(HashMap::new)
            .entry(subject)
            .or_insert_with(Vec::new)
            .push(grade);
    }

    pub fn show_subject(&self, subject: &str) -> String {
        match self.by_subject.get(subject) {
            Some(students) => {
                let mut result = format!("Grades for subject '{}':\n", subject);
                for (student, grades) in students {
                    result.push_str(&format!("  {}: {:?}\n", student, grades));
                }
                result
            }
            None => format!("No grades recorded for subject '{}'.\n", subject),
        }
    }

    pub fn show_student(&self, student: &str) -> String {
        match self.by_student.get(student) {
            Some(subjects) => {
                let mut result = format!("Grades for student '{}':\n", student);
                for (subject, grades) in subjects {
                    result.push_str(&format!("  {}: {:?}\n", subject, grades));
                }
                result
            }
            None => format!("No grades recorded for student '{}'.\n", student),
        }
    }

    pub fn remove_grade(&mut self, grade: Grade, student: &str, subject: &str) -> bool {
    let mut removed = false;

    if let Some(students) = self.by_subject.get_mut(subject) {
        if let Some(grades) = students.get_mut(student) {
            if let Some(pos) = grades.iter().position(|g| *g == grade) {
                grades.remove(pos);
                removed = true;

                if grades.is_empty() {
                    students.remove(student);
                }
            }
        }

        if students.is_empty() {
            self.by_subject.remove(subject);
        }
    }

    if let Some(subjects) = self.by_student.get_mut(student) {
        if let Some(grades) = subjects.get_mut(subject) {
            if let Some(pos) = grades.iter().position(|g| *g == grade) {
                grades.remove(pos);

                // Remove empty grade list
                if grades.is_empty() {
                    subjects.remove(subject);
                }
            }
        }

        // Remove empty student entry
        if subjects.is_empty() {
            self.by_student.remove(student);
        }
    }

    removed
    }


    pub fn average_subject(&self, subject: &str) -> Option<f32> {
        let students = self.by_subject.get(subject)?;
        let mut all: Vec<i8> = Vec::new(); 

        for grades in students.values() {
            all.extend(grades);
        }

        if all.is_empty() {
            None
        } else {
            Some(all.iter().copied().map(|g| g as f32).sum::<f32>() / all.len() as f32)
        }
    }

    pub fn average_student(&self, student: &str) -> Option<f32> {
        let subjects = self.by_student.get(student)?;
        let mut all: Vec<i8> = Vec::new(); 

        for grades in subjects.values() {
            all.extend(grades);
        }

        if all.is_empty() {
            None
        } else {
            Some(all.iter().copied().map(|g| g as f32).sum::<f32>() / all.len() as f32)
        }
    }
}
