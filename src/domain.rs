use serde::Serialize;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Serialize, Debug)]
pub struct NewSubscriber {
    pub email: String,
    pub name: SubscriberName,
}

#[derive(Serialize, Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = s.chars().any(|char| forbidden_chars.contains(&char));

        if is_empty_or_whitespace || is_too_long || contains_forbidden_chars {
            panic!("{} is not a valid subscriber name.", s);
        } else {
            Ok(Self(s))
        }
    }

}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}