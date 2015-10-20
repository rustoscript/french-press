use std::cell::RefCell;
use std::mem;
use std::cmp;
use js_types::js_type::JsT;

// Initial Arena size in bytes
const INITIAL_SIZE: usize = 1024;
// Minimum Arena capacity is at least 1 byte
const MIN_CAP: usize = 1;

struct ChunkList<T> {
    curr: Vec<T>,
    rest: Vec<Vec<T>>,
}

impl<T> ChunkList<T> {
    fn grow(&mut self) {
        let new_cap = self.curr.capacity()
                               .checked_mul(2)
                               .expect("ChunkList: In method `grow`, `checked_mul` returned None. Aborting!");
        let chunk = mem::replace(&mut self.curr, Vec::with_capacity(new_cap));
        self.rest.push(chunk);
    }
}

struct GranularArena<T> {
    chunks: RefCell<ChunkList<T>>,
}

impl<T> GranularArena<T> {
    fn new() -> GranularArena<T> {
        let sz = cmp::max(1, mem::size_of::<T>());
        GranularArena::with_capacity(INITIAL_SIZE / sz)
    }

    fn with_capacity(cap: usize) -> GranularArena<T> {
        let cap = cmp::max(MIN_CAP, cap);
        GranularArena {
            chunks: RefCell::new(ChunkList {
                curr: Vec::with_capacity(cap),
                rest: Vec::new(),
            }),
        }
    }

    fn alloc(&self, val: T) -> &mut T {
        let mut chunks = self.chunks.borrow_mut();
        let next_item_idx = chunks.curr.len();
        chunks.curr.push(val);

        let new_item_ref = {
            let new_item_ref = &mut chunks.curr[next_item_idx];

            // According to what I've read online, this extends the lifetime of
            // the returned ref from that of `chunks` as borrowed on line 47 to
            // that of `self`. This is allowable because we never grow the inner
            // `Vec`s beyond their initial cap, and the returned reference is
            // unique since it's &mut, which means the arene never gives away
            // refs to existing items.
            unsafe { mem::transmute::<&mut T, &mut T>(new_item_ref) }
        };

        if chunks.curr.len() == chunks.curr.capacity() {
            chunks.grow();
        }

        new_item_ref
    }

    // TODO Figure out what granular deallocation will mean
    // TODO Is there a better way to allocate? Should I group items that are
    // temporally-local together? They sort of will be anyway, but is it worth
    // it to force such a thing?
}

pub struct Compartment {
    source: String,
    arena: GranularArena<JsT>,
}

impl Compartment {
    pub fn new(source: &str) -> Compartment {
        Compartment {
            source: String::from(source),
            arena: GranularArena::new(),
        }
    }

    pub fn alloc_inside(&self, js_t: JsT) -> &mut JsT {
        self.arena.alloc(js_t)
    }
}
