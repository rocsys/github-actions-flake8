use regex::{Regex, RegexBuilder};

pub struct LineMatch<'a> {
    pub path: &'a str,
    pub line: &'a str,
    pub column: &'a str,
    pub message: &'a str,
}

pub struct LineParser {
    regex: Regex,
}

impl Default for LineParser {
    fn default() -> Self {
        Self {
            regex: RegexBuilder::new(r"^([^:]+):(\d+):(\d+): (.*)$")
                .build()
                .expect("Failed to build regex!"),
        }
    }
}

impl LineParser {
    pub fn parse<'a>(&self, line: &'a str) -> Option<LineMatch<'a>> {
        self.regex.captures(line).map(move |captures| LineMatch {
            path: captures.get(1).unwrap().as_str(),
            line: captures.get(2).unwrap().as_str(),
            column: captures.get(3).unwrap().as_str(),
            message: captures.get(4).unwrap().as_str(),
        })
    }
}
