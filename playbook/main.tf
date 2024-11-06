locals {
  namespace      = "hydra-doom"
  operator_image = "ghcr.io/demeter-run/doom-patrol-operator:sha-0b846e7"
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

variable "blockfrost_key" {
  type = string
}

variable "dmtr_project_id" {
  type = string
}

variable "dmtr_api_key" {
  type = string
}

variable "dmtr_port_name" {
  type = string
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
  sidecar_image       = "ghcr.io/demeter-run/doom-patrol-hydra:803df77809e3b5d65ad752603257b31ee05cf481"
  open_head_image     = "ghcr.io/demeter-run/doom-patrol-hydra:803df77809e3b5d65ad752603257b31ee05cf481"
  control_plane_image = "ghcr.io/demeter-run/doom-patrol-hydra:803df77809e3b5d65ad752603257b31ee05cf481"
  blockfrost_key      = var.blockfrost_key
  external_port       = 80
  admin_key_path      = "${path.module}/admin.sk"
  admin_addr          = "addr_test1vpgcjapuwl7gfnzhzg6svtj0ph3gxu8kyuadudmf0kzsksqrfugfc"
  dmtr_project_id     = var.dmtr_project_id
  dmtr_api_key        = var.dmtr_api_key
  dmtr_port_name      = var.dmtr_port_name
  hydra_scripts_tx_id = "31b833c943fc267ee532c772be032183d5842b69492afaf5daa360171168c238"
}
