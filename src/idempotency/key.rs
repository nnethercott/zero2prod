use anyhow::bail;

#[derive(Debug)]
pub struct IdempotencyKey(String);

impl TryFrom<String> for IdempotencyKey{
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        // check empty
        if value.is_empty(){
            bail!("key cannot be empty");
        }
        const MAX_LENGTH: usize = 50;
        if value.len()>MAX_LENGTH{
            bail!("maximum length for idempotency is 50");
        }
        Ok(Self(value))
    }
}

impl From<IdempotencyKey> for String{
    fn from(value: IdempotencyKey) -> Self {
        value.0
    }
}

impl AsRef<str> for IdempotencyKey{
    fn as_ref(&self) -> &str {
        &self.0
    }
}


// automatic try from 
// impl<T, U> TryInto<U> for T
// where U: TryFrom<T>{
//     type Error = <U as TryFrom<T>>::Error;
//
//     fn try_into(self) -> Result<U, Self::Error> {
//         U::try_from(self)
//     }
// }
