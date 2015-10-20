use js_types::js_type::{JsT, JsType, Marking};
use std::vec::Vec;
use std::collections::hash_set::HashSet;
use std::collections::hash_map::HashMap;
use std::collections::vec_deque::VecDeque;
use uuid::Uuid;


/// A map from uuids to references to objects. The interface to the garbage
/// collector, this map gets updated whenever a GC cycle is performed. References
/// that are no longer live may be dropped from the map.
pub struct UuidMap<'t> {
    inner: HashMap<Uuid, RefCell<JsT>>,
}

impl<'t> UuidMap<'t> {
    pub fn new() -> UuidMap<'t> {
        UuidMap {
            inner: HashMap::new(),
        }
    }

    pub fn from_jsts(jsts: Vec<&'t JsT>) -> UuidMap<'t> {
        let mut tmp_map = HashMap::new();
        jsts.iter().map(|j| tmp_map.insert(j.uuid, *j));
        UuidMap {
            inner: tmp_map,
        }
    }

    pub fn insert_ref(&mut self, jst: &'t JsT) {
        self.inner.insert(jst.uuid, jst);
    }

    pub fn drop_uuid(&mut self, uuid: Uuid) {
        self.inner.remove(&uuid);
    }

    pub fn drop_ref(&mut self, jst: &'t JsT) {
        self.inner.remove(&jst.uuid);
    }

    pub fn mark_uuid(&mut self, uuid: Uuid, marking: Marking) {
        if let Some(jst) = self.inner.get_mut(&uuid) {
            (*jst).gc_flag = marking;
        }
        //let jst_opt = self.inner.get_mut(&uuid);
        //if let Some(jst) = jst_opt {
        //    let ref mut jst = jst.clone();
        //    jst.gc_flag = marking;
        //    self.inner.insert(uuid, jst);
        //}
    }
}



/// A graph node that stores sets of object uuids, as well as a list of
/// pointers to nodes it can flow to.
/// A variable v is live on edge e if there is:
///     - a node n in the CFG s.t. use[n] contains v
///     - a directed path from e -> n s.t. for every statement s on the path,
///       def[s] does not contain v
#[derive(Clone)]
struct LivenessNode {
    defs: HashSet<Uuid>,
    uses: HashSet<Uuid>,
    ins: HashSet<Uuid>,
    outs: HashSet<Uuid>,
    succ: Vec<Box<LivenessNode>>,
    pred: Vec<Box<LivenessNode>>,
}

struct LivenessGraph {
    nodes: Vec<LivenessNode>,
}

type Statement = i32; // TODO, placeholder

impl LivenessNode {
    fn new() -> LivenessNode {
        LivenessNode {
            defs: HashSet::new(),
            uses: HashSet::new(),
            ins: HashSet::new(),
            outs: HashSet::new(),
            succ: Vec::new(),
            pred: Vec::new(),
        }
    }

    fn node_flow(&mut self) {
        self.ins = self.uses.union(&self.outs.difference(&self.defs).cloned().collect())
                            .cloned().collect();

        self.outs = self.succ.clone().into_iter().flat_map(|s| s.ins).collect();
    }

    // TODO Given a list of AST instructions, parse out what UUIDs are used
    fn compute_uses(&mut self, statements: Vec<Statement>) {

    }

    fn compute_defs(&mut self, statements: Vec<Statement>) {

    }

}

impl LivenessGraph {
    fn new() -> LivenessGraph {
        LivenessGraph {
            nodes: Vec::new(),
        }
    }

    fn graph_flow(&mut self) {
        let mut node_queue: VecDeque<&mut LivenessNode> = self.nodes.iter_mut().collect();
        while let Some(n) = node_queue.pop_front() {
            let old_ins = n.ins.clone();
            let old_outs = n.outs.clone();
            n.node_flow();
            if old_outs != n.outs {
                n.succ.iter_mut().map(|s| node_queue.push_back(s));
            }
        }
    }
}


pub struct StackFrame<'m> {
    members: Vec<&'m JsT>,
}

impl<'m> StackFrame<'m> {
    pub fn new() -> StackFrame<'m> {
        StackFrame { members: Vec::new(), }
    }

    pub fn alloc_ref(&mut self, refce: &'m mut JsT)  {
        self.members.push(refce);
    }
}
