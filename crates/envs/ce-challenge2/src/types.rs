pub type Grade = i8;
pub type Student = String;
pub type Subject = String;

pub const DANISH_GRADES: [Grade; 7] = [-3, 0, 2, 4, 7, 10, 12];

pub fn is_valid_danish_grade(g: Grade) -> bool {
    DANISH_GRADES.contains(&g)
}
