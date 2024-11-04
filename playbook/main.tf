locals {
  namespace      = "hydra-doom"
  operator_image = "ghcr.io/demeter-run/doom-patrol-operator:sha-aa3fdda"
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
  sidecar_image       = "ghcr.io/demeter-run/doom-patrol-metrics-exporter:a5406f8180a77474c06e44f95619cada183bb8fe"
  open_head_image     = "ghcr.io/demeter-run/doom-patrol-hydra:0ee2f6b6d38e500097d992820e0089ead7cb10bc"
  control_plane_image = "ghcr.io/demeter-run/doom-patrol-hydra:c4d9d80ea42a202408cf062193d2a675a42f432f"
  blockfrost_key      = ""
  external_port       = 80
}
