use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

pub trait JsTrait {}

pub struct JsT {
    uid: Uuid,
    t: Box<JsTrait>,
}

impl JsT {
    pub fn new(t: Box<JsTrait>) -> JsT {
        JsT {
            uid: Uuid::new_v4(),
            t: t,
        }
    }
}

impl PartialEq for JsT {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for JsT {}

impl Hash for JsT {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.uid.hash(state);
    }

    fn hash_slice<H>(data: &[Self], state: &mut H) where H: Hasher {
        for ref d in data {
            d.uid.hash(state);
        }
    }
}

