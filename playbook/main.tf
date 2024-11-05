locals {
  namespace      = "hydra-doom"
  operator_image = "ghcr.io/demeter-run/doom-patrol-operator:sha-08c1f0f"
  # operator_image = "doom-patrol-operator:local"
}

terraform {
  backend "s3" {
    bucket = "hydra-doom-tf"
    key    = "clusters/hydra-doom-dev-cluster/tfstate"
    region = "us-east-1"
  }
  required_providers {
    kubernetes = {
      source  = "hashicorp/kubernetes"
      version = "2.22.0"
    }
  }
}

provider "kubernetes" {
  config_path    = "~/.kube/config"
  config_context = "hydra-doom-dev-cluster"
}

provider "helm" {
  kubernetes {
    config_path    = "~/.kube/config"
    config_context = "hydra-doom-dev-cluster"
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

  namespace           = local.namespace
  external_domain     = "us-east-1.hydra-doom.sundae.fi"
  operator_image      = local.operator_image
  hydra_node_image    = "ghcr.io/cardano-scaling/hydra-node:unstable"
  sidecar_image       = "ghcr.io/demeter-run/doom-patrol-hydra:08c1f0f1c58be998a07ab218b0a694785d16bb09"
  open_head_image     = "ghcr.io/demeter-run/doom-patrol-hydra:08c1f0f1c58be998a07ab218b0a694785d16bb09"
  control_plane_image = "ghcr.io/demeter-run/doom-patrol-hydra:08c1f0f1c58be998a07ab218b0a694785d16bb09"
  blockfrost_key      = ""
  external_port       = 80
  admin_key_path      = "${path.module}/admin.sk"
  admin_addr          = "addr_test1vpgcjapuwl7gfnzhzg6svtj0ph3gxu8kyuadudmf0kzsksqrfugfc"
}
