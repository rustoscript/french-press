pub mod compartment;
pub mod ref_manip;

#[cfg(test)]
mod tests {
    use super::*;
    use js_types::js_type::{JsT, JsType};
    use js_types::js_obj::JsObjStruct;
    use js_types::js_str::JsStrStruct;
    use alloc::compartment::Compartment;
    use alloc::ref_manip::StackFrame;

    #[test]
    fn test_alloc() {
        let comp = Compartment::new("test");
        let mut frame = StackFrame::new();
        let test_obj = JsT::new(
                        JsType::JsObj(
                        JsObjStruct::new(None, "test_obj",
                                         vec![(JsT::new(JsType::JsStr(JsStrStruct::new("a"))),
                                               JsT::new(JsType::JsNum(1.0f64)))])));
        let obj_ref = comp.alloc_inside(test_obj);
        frame.alloc_ref(obj_ref);
    }
}
