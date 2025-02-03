use validator::ValidateEmail;
use std::borrow::Cow;

#[derive(Debug)]
pub struct SubscriberEmail(String);

impl ValidateEmail for SubscriberEmail{
    fn as_email_string(&self) -> Option<Cow<str>> {
        Some(Cow::Borrowed(&self.0))
    }
}

impl SubscriberEmail{
    fn parse(s: String)->Result<Self, String>{
        let email = SubscriberEmail(s);
        if email.validate_email(){
            return Ok(email);
        }
        else{
            return Err("invalid email provided".to_string());
        }
    }
}

impl AsRef<str> for SubscriberEmail{
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests{
    use super::SubscriberEmail;
    use claims::assert_err;
    use fake::faker::internet::en::SafeEmail;
    use fake::Fake;

    // kind of hacky
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[derive(Clone, Debug)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut quickcheck::Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }


    #[quickcheck_macros::quickcheck]
    fn test_good_email(email: ValidEmailFixture)->bool{
        dbg!(&email.0);
        SubscriberEmail::parse(email.0).is_ok()
    }
    #[test]
    fn test_empty_string_is_rejected(){
        let email = "".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn test_no_at_symbol_rejected(){
        let email = "nategmail.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn test_missing_subject_rejected(){
        let email = "@gmail.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
}
