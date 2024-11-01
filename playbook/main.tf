locals {
  namespace      = "hydra-doom"
  operator_image = "ghcr.io/demeter-run/doom-patrol-operator:sha-f06d308"
  # operator_image = "doom-patrol-operator:local"
}

terraform {
  backend "local" {
    path = "local.tfstate"
  }
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.22.0"
    }
  }
}

provider "kubernetes" {
  config_path = "~/.kube/config"
  # config_context = "kind-k8scluster"
  config_context = "felipe@txpipe.io@hydra-doom-dev-cluster.us-east-1.eksctl.io"
}

provider "helm" {
  kubernetes {
    config_path = "~/.kube/config"
    # config_context = "kind-k8scluster"
    config_context = "felipe@txpipe.io@hydra-doom-dev-cluster.us-east-1.eksctl.io"
  }
}



resource "kubernetes_namespace" "namespace" {
  metadata {
    name = local.namespace
  }
}

module "stage1" {
  source = "../bootstrap/stage1/"
}

module "stage2" {
  source     = "../bootstrap/stage2"
  depends_on = [module.stage1]

  namespace       = local.namespace
  external_domain = "external.domain"
  image           = local.operator_image
  open_head_image = ""
  sidecar_image   = "ghcr.io/demeter-run/doom-patrol-metrics-exporter:a5406f8180a77474c06e44f95619cada183bb8fe"
  blockfrost_key  = ""
  external_port   = 80
}
