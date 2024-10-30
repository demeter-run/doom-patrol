use doom_patrol::custom_resource::HydraDoomNode;
use kube::CustomResourceExt;

fn main() {
    print!("{}", serde_yaml::to_string(&HydraDoomNode::crd()).unwrap())
}
