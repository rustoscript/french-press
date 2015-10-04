use std::cmp::Ordering;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

pub struct JsType {
    uid: Uuid,
    t: Box<JsT>,
}

impl JsType {
    fn new(t: Box<JsT>) -> JsType {
        JsType {
            uid: Uuid::new_v4(),
            t: t,
        }
    }
}

impl PartialEq for JsType {
    fn eq(&self, other: &Self) -> bool {
        self.uid == other.uid
    }

    fn ne(&self, other: &Self) -> bool {
        !self.eq(other)
    }
}

impl Eq for JsType{}

impl Hash for JsType {
    fn hash<H>(&self, state: &mut H) where H: Hasher {
        self.uid.hash(state);
    }

    fn hash_slice<H>(data: &[Self], state: &mut H) where H: Hasher {
        for ref d in data {
            d.uid.hash(state);
        }
    }
}

pub trait JsT {}
