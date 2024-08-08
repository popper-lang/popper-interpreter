use std::fmt::Debug;
use std::ops::Deref;

#[derive(Debug, Clone)]
pub struct Tagged<T>
where
    T: Debug + Clone,
{
    pub tag: String,
    pub value: T,
}

pub trait Tag: Debug + Clone {
    fn tag(&self, s: String) -> Tagged<Self>;
}

impl<T: Debug + Clone> Tag for T {
    fn tag(&self, s: String) -> Tagged<Self> {
        Tagged {
            tag: s,
            value: self.clone(),
        }
    }
}

impl<T: Debug + Clone> Tagged<T> {
    pub fn void(t: T) -> Self {
        Tagged {
            tag: "".to_string(),
            value: t,
        }
    }
}

impl<T: Debug + Clone> Deref for Tagged<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.value
    }
}
