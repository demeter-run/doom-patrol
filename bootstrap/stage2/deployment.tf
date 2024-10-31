resource "kubernetes_deployment_v1" "operator" {
  wait_for_rollout = false

  metadata {
    namespace = var.namespace
    name      = local.component
    labels = {
      role     = local.component
      "run-on" = "fargate"
    }
  }

  spec {
    replicas = 1

    // No 2 replicas simultaneously
    strategy {
      type = "Recreate"
    }

    selector {
      match_labels = {
        role     = local.component
        "run-on" = "fargate"
      }
    }

    template {
      metadata {
        labels = {
          role     = local.component
          "run-on" = "fargate"
        }
      }

      spec {
        container {
          image = var.image
          name  = "main"

          env {
            name  = "K8S_IN_CLUSTER"
            value = "true"
          }

          env {
            name  = "IMAGE"
            value = var.hydra_pod_image
          }

          env {
            name  = "OPEN_HEAD_IMAGE"
            value = var.hydra_pod_open_head_image
          }

          env {
            name  = "CONFIGMAP"
            value = local.configmap
          }

          env {
            name  = "BLOCKFROST_KEY"
            value = var.blockfrost_key
          }

          env {
            name  = "EXTERNAL_DOMAIN"
            value = var.external_domain
          }

          env {
            name  = "EXTERNAL_PORT"
            value = var.external_port
          }

          resources {
            limits = {
              cpu    = var.resources.limits.cpu
              memory = var.resources.limits.memory
            }
            requests = {
              cpu    = var.resources.requests.cpu
              memory = var.resources.requests.memory
            }
          }

          port {
            name           = "api"
            container_port = 8000
            protocol       = "TCP"
          }
        }

        volume {
          name = "config"
          config_map {
            name = local.configmap
          }
        }

        dynamic "toleration" {
          for_each = var.tolerations

          content {
            effect   = toleration.value.effect
            key      = toleration.value.key
            operator = toleration.value.operator
            value    = toleration.value.value
          }
        }
      }
    }
  }
}
