use regex::Regex;

pub(crate) trait Matcher {
    fn matches(&self, regex: Option<Regex>) -> bool;
}
