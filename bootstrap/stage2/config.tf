resource "kubernetes_config_map" "node-config" {
  metadata {
    namespace = var.namespace
    name      = local.configmap
  }

  data = {
    "admin.sk"                 = "${file("${path.module}/admin.sk")}"
    "protocol-parameters.json" = "${file("${path.module}/protocol-parameters.json")}"
  }
}
