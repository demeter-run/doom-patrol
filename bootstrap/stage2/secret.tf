resource "kubernetes_secret" "admin_key" {
  metadata {
    name      = local.secret
    namespace = var.namespace
  }
  data = {
    "admin.sk" = "${file(var.admin_key_path)}"
  }
  type = "Opaque"
}
