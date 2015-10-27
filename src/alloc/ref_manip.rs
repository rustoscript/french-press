use std::cell::RefCell;
use std::collections::hash_map::HashMap;
use std::collections::hash_set::HashSet;
use std::collections::vec_deque::VecDeque;
use std::vec::Vec;
use js_types::js_type::{JsVar, JsType};
use uuid::Uuid;

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
    members: Vec<&'m JsVar>,
}

impl<'m> StackFrame<'m> {
    pub fn new() -> StackFrame<'m> {
        StackFrame { members: Vec::new(), }
    }

    pub fn alloc_ref(&mut self, refce: &'m mut JsVar)  {
        self.members.push(refce);
    }
}
