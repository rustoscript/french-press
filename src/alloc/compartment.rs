use typed_arena::Arena;
use js_types::js_type::JsT;

pub struct Compartment<'a> {
    source: &'a str,
    arena: Arena<JsT>,
}

impl<'a> Compartment<'a> {
    pub fn new(source: &str) -> Compartment {
        Compartment {
            source: source,
            arena: Arena::new(),
        }
    }

    pub fn alloc_inside(&self, js_t: JsT) {
        self.arena.alloc(js_t);
    }
}
