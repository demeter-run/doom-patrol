use doom_patrol::custom_resource::HydraDoomPod;
use kube::CustomResourceExt;

fn main() {
    print!("{}", serde_yaml::to_string(&HydraDoomPod::crd()).unwrap())
}
