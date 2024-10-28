use crate::k8s::K8sHelper;

pub struct Context {
    pub k8s: K8sHelper,
}

impl Context {
    pub fn new(k8s: K8sHelper) -> Self {
        Self { k8s }
    }
}
