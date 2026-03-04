use regex::Regex;
use serde::Deserialize;

pub struct FoldPattern {
    #[allow(dead_code)]
    pub name: String,
    pub start: Regex,
    pub end: Regex,
}

#[derive(Deserialize)]
pub struct FoldConfigRaw {
    #[serde(default = "default_true")]
    pub include_defaults: bool,
    #[serde(default)]
    pub patterns: Vec<FoldPatternRaw>,
}

impl Default for FoldConfigRaw {
    fn default() -> Self {
        Self {
            include_defaults: true,
            patterns: Vec::new(),
        }
    }
}

#[derive(Deserialize)]
pub struct FoldPatternRaw {
    pub name: String,
    pub start: String,
    pub end: String,
}

fn default_true() -> bool {
    true
}

fn default_patterns() -> Vec<FoldPattern> {
    vec![
        FoldPattern {
            name: "plus-minus".to_string(),
            start: Regex::new(r"^\s*(?:#|//)\s*\+\+\+\s*(?P<label>.*?)\s*$").unwrap(),
            end: Regex::new(r"^\s*(?:#|//)\s*---").unwrap(),
        },
        FoldPattern {
            name: "region".to_string(),
            start: Regex::new(r"^\s*(?://|/\*|#)\s*#region\b\s*(?P<label>.*?)\s*$").unwrap(),
            end: Regex::new(r"^\s*(?://|/\*|#)\s*#endregion").unwrap(),
        },
    ]
}

pub struct FoldConfig {
    pub patterns: Vec<FoldPattern>,
}

impl FoldConfig {
    pub fn from_raw(raw: Option<FoldConfigRaw>) -> Self {
        let raw = raw.unwrap_or_default();
        let mut patterns = Vec::new();

        if raw.include_defaults {
            patterns.extend(default_patterns());
        }

        for p in raw.patterns {
            match (Regex::new(&p.start), Regex::new(&p.end)) {
                (Ok(start), Ok(end)) => {
                    patterns.push(FoldPattern {
                        name: p.name,
                        start,
                        end,
                    });
                }
                _ => {
                    eprintln!("Invalid regex in pattern '{}', skipping", p.name);
                }
            }
        }

        FoldConfig { patterns }
    }
}
