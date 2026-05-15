use quick_xml::Reader;
use quick_xml::events::Event;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct CoverageReport {
    pub line_rate: f64,
    pub branch_rate: f64,
    pub lines_valid: u64,
    pub lines_covered: u64,
    pub branches_valid: u64,
    pub branches_covered: u64,
}

pub fn parse_file(path: &std::path::Path) -> Result<CoverageReport, Box<dyn std::error::Error>> {
    let xml = std::fs::read_to_string(path)?;
    parse_xml(&xml)
}

pub fn parse_xml(xml: &str) -> Result<CoverageReport, Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut report = CoverageReport::default();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                if e.name().as_ref() == b"coverage" {
                    for attr_result in e.attributes() {
                        let attr = attr_result?;
                        let value = attr.unescape_value()?.to_string();
                        match attr.key.as_ref() {
                            b"line-rate" => {
                                report.line_rate = value.parse().unwrap_or(0.0);
                            }
                            b"branch-rate" => {
                                report.branch_rate = value.parse().unwrap_or(0.0);
                            }
                            b"lines-valid" => {
                                report.lines_valid = value.parse().unwrap_or(0);
                            }
                            b"lines-covered" => {
                                report.lines_covered = value.parse().unwrap_or(0);
                            }
                            b"branches-valid" => {
                                report.branches_valid = value.parse().unwrap_or(0);
                            }
                            b"branches-covered" => {
                                report.branches_covered = value.parse().unwrap_or(0);
                            }
                            _ => {}
                        }
                    }
                    // Found the coverage element; we have what we need.
                    break;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }

    Ok(report)
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

pub fn filter_analysis_file(
    input: &Path,
    output: &Path,
    target_suffix: &str,
) -> Result<CoverageReport, Box<dyn std::error::Error>> {
    let xml = std::fs::read_to_string(input)?;
    let (filtered, report) = filter_analysis_xml(&xml, target_suffix)?;
    std::fs::write(output, filtered)?;
    Ok(report)
}

pub fn filter_analysis_xml(
    xml: &str,
    target_suffix: &str,
) -> Result<(String, CoverageReport), Box<dyn std::error::Error>> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    let mut source = String::new();
    let mut in_source = false;

    let mut timestamp = String::new();
    let mut version = String::new();
    let mut complexity = String::from("0");

    let mut in_target_class = false;
    let mut in_methods = false;
    let mut in_class_lines = false;
    let mut class_name = String::from("analysis.rs");
    let mut class_filename = String::from("crates/envs/ce-security/src/analysis.rs");
    let mut class_complexity = String::from("0");
    let mut class_branch_rate = 0.0f64;
    let mut class_branches_valid = 0u64;
    let mut class_branches_covered = 0u64;

    let mut lines: Vec<(u64, u64, String)> = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                if e.name().as_ref() == b"coverage" {
                    for attr_result in e.attributes() {
                        let attr = attr_result?;
                        let value = attr.unescape_value()?.to_string();
                        match attr.key.as_ref() {
                            b"timestamp" => timestamp = value,
                            b"version" => version = value,
                            b"complexity" => complexity = value,
                            _ => {}
                        }
                    }
                } else if e.name().as_ref() == b"source" {
                    in_source = true;
                } else if e.name().as_ref() == b"class" {
                    let mut filename = String::new();
                    let mut name = String::new();
                    let mut cplx = String::from("0");
                    let mut branch_rate = 0.0f64;
                    let mut branches_valid = 0u64;
                    let mut branches_covered = 0u64;
                    for attr_result in e.attributes() {
                        let attr = attr_result?;
                        let value = attr.unescape_value()?.to_string();
                        match attr.key.as_ref() {
                            b"filename" => filename = value.replace('\\', "/"),
                            b"name" => name = value,
                            b"complexity" => cplx = value,
                            b"branch-rate" => branch_rate = value.parse().unwrap_or(0.0),
                            b"branches-valid" => branches_valid = value.parse().unwrap_or(0),
                            b"branches-covered" => branches_covered = value.parse().unwrap_or(0),
                            _ => {}
                        }
                    }
                    if filename.ends_with(target_suffix) || filename.ends_with("analysis.rs") {
                        in_target_class = true;
                        class_name = name;
                        class_filename = filename;
                        class_complexity = cplx;
                        class_branch_rate = branch_rate;
                        class_branches_valid = branches_valid;
                        class_branches_covered = branches_covered;
                    }
                } else if in_target_class && e.name().as_ref() == b"methods" {
                    in_methods = true;
                } else if in_target_class && e.name().as_ref() == b"lines" && !in_methods {
                    in_class_lines = true;
                }
            }
            Ok(Event::Empty(ref e)) => {
                if in_target_class && in_class_lines && e.name().as_ref() == b"line" {
                    let mut number = 0u64;
                    let mut hits = 0u64;
                    let mut branch = String::from("false");
                    for attr_result in e.attributes() {
                        let attr = attr_result?;
                        let value = attr.unescape_value()?.to_string();
                        match attr.key.as_ref() {
                            b"number" => number = value.parse().unwrap_or(0),
                            b"hits" => hits = value.parse().unwrap_or(0),
                            b"branch" => branch = value,
                            _ => {}
                        }
                    }
                    if number > 0 {
                        lines.push((number, hits, branch));
                    }
                }
            }
            Ok(Event::Text(t)) => {
                if in_source {
                    source = t.unescape()?.to_string();
                }
            }
            Ok(Event::End(ref e)) => {
                if e.name().as_ref() == b"source" {
                    in_source = false;
                } else if in_target_class && e.name().as_ref() == b"methods" {
                    in_methods = false;
                } else if in_target_class && e.name().as_ref() == b"lines" && !in_methods {
                    in_class_lines = false;
                } else if e.name().as_ref() == b"class" {
                    in_target_class = false;
                    in_methods = false;
                    in_class_lines = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(Box::new(e)),
            _ => {}
        }
        buf.clear();
    }

    let lines_valid = lines.len() as u64;
    let lines_covered = lines.iter().filter(|(_, hits, _)| *hits > 0).count() as u64;
    let line_rate = if lines_valid > 0 {
        lines_covered as f64 / lines_valid as f64
    } else {
        0.0
    };

    let report = CoverageReport {
        line_rate,
        branch_rate: class_branch_rate,
        lines_valid,
        lines_covered,
        branches_valid: class_branches_valid,
        branches_covered: class_branches_covered,
    };

    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str(&format!(
        "<coverage line-rate=\"{:.12}\" branch-rate=\"{:.12}\" lines-covered=\"{}\" lines-valid=\"{}\" branches-covered=\"{}\" branches-valid=\"{}\" complexity=\"{}\" timestamp=\"{}\" version=\"{}\">\n",
        report.line_rate,
        report.branch_rate,
        report.lines_covered,
        report.lines_valid,
        report.branches_covered,
        report.branches_valid,
        esc(&complexity),
        esc(&timestamp),
        esc(&version)
    ));
    out.push_str("  <sources>\n");
    out.push_str(&format!("    <source>{}</source>\n", esc(&source)));
    out.push_str("  </sources>\n");
    out.push_str("  <packages>\n");
    out.push_str(&format!(
        "    <package name=\"crates.envs.ce-security.src\" line-rate=\"{:.12}\" branch-rate=\"{:.12}\" complexity=\"{}\">\n",
        report.line_rate,
        report.branch_rate,
        esc(&class_complexity)
    ));
    out.push_str("      <classes>\n");
    out.push_str(&format!(
        "        <class name=\"{}\" filename=\"{}\" line-rate=\"{:.12}\" branch-rate=\"{:.12}\" complexity=\"{}\">\n",
        esc(&class_name),
        esc(&class_filename.replace('/', "\\")),
        report.line_rate,
        report.branch_rate,
        esc(&class_complexity)
    ));
    out.push_str("          <methods />\n");
    out.push_str("          <lines>\n");
    for (number, hits, branch) in &lines {
        out.push_str(&format!(
            "            <line number=\"{}\" hits=\"{}\" branch=\"{}\" />\n",
            number, hits, esc(branch)
        ));
    }
    out.push_str("          </lines>\n");
    out.push_str("        </class>\n");
    out.push_str("      </classes>\n");
    out.push_str("    </package>\n");
    out.push_str("  </packages>\n");
    out.push_str("</coverage>\n");

    Ok((out, report))
}
