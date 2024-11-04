resource "kubernetes_secret" "postgres" {
  metadata {
    name      = local.secret
    namespace = var.namespace
  }
  data = {
    "admin.sk" = "${file("${path.module}/admin.sk")}"
  }
  type = "Opaque"
}
