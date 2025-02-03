use unicode_segmentation::UnicodeSegmentation;
use std::fmt::Display;

#[derive(Debug)]
pub struct SubscriberName(String);

// impl Display for SubscriberName{
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.write_str(&self.0);
//         Ok(())
//     }
// }

impl SubscriberName{
    pub fn parse(s: String)->Result<Self, String>{
        let is_empty = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;

        let bad_chars = [
            '/', '\\', '(', ')', '_', '<', '"', '<', '>', '[', ']', '{', '{',
        ];
        let contains_bad_chars = s.chars().any(|c| bad_chars.contains(&c));

        if (is_empty || is_too_long || contains_bad_chars){
            return Err("invalid subscriber name !".to_string());
        }
        Ok(Self(s))
    }
}

impl AsRef<str> for SubscriberName{
    fn as_ref(&self) -> &str {
        &self.0
    }
}


#[cfg(test)]
mod test{
    use super::*;
    use unicode_segmentation::UnicodeSegmentation;
    use claims::{assert_err, assert_ok};

    #[test]
    fn a_256_graphene_is_ok(){
        let name = "n".repeat(256);
        let sub = SubscriberName::parse(name);
        assert_ok!(sub);
    }

    #[test]
    fn a_longer_than_256_graphene_is_err(){
        let name = "n".repeat(257);
        let sub = SubscriberName::parse(name);
        assert_err!(sub);
    }

    #[test]
    fn whitespace_only_name_rejected(){
        let name = "    ".to_string();
        let sub = SubscriberName::parse(name);
        assert_err!(sub);
    }

    #[test]
    fn empty_string_name_rejected(){
        let name = "".to_string();
        let sub = SubscriberName::parse(name);
        assert_err!(sub);
    }

    #[test]
    fn invalid_character_names_rejected(){
        let name = "nate<nethercott)".to_string();
        let sub = SubscriberName::parse(name);
        assert_err!(sub);
    }

}
