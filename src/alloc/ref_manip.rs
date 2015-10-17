use js_types::js_type::{JsT, JsType};
use std::vec::Vec;
use std::collections::hash_set::HashSet;
use uuid::Uuid;

/// A graph node that stores a set of object UIDs, as well as a list of
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
    root: LivenessNode,
    nodes: Vec<LivenessNode>,
}

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

        self.outs = self.succ.clone().into_iter().flat_map(|succ| succ.ins).collect();
    }

    // TODO Given a list of AST instructions, parse out what UUIDs are used

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
