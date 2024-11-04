resource "kubernetes_secret" "admin_key" {
  metadata {
    name      = local.secret
    namespace = var.namespace
  }
  data = {
    "admin.sk" = "${file("${path.module}/admin.sk")}"
  }
  type = "Opaque"
}
