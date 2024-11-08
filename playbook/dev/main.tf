locals {
  namespace      = "hydra-doom-dev"
  operator_image = "ghcr.io/demeter-run/doom-patrol-operator:sha-6c44eb2"
}

terraform {
  backend "s3" {
    bucket = "hydra-doom-tf"
    key    = "clusters/hydra-doom-dev-cluster/tfstate.dev"
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

variable "init_aws_access_key_id" {
  type = string
}

variable "init_aws_secret_access_key" {
  type = string
}

module "stage2" {
  source = "../../bootstrap/stage2"

  namespace                  = local.namespace
  control_plane_prefix       = "api-dev"
  external_domain            = "us-east-1.hydra-doom.sundae.fi"
  operator_image             = local.operator_image
  hydra_node_image           = "ghcr.io/cardano-scaling/hydra-node:latest"
  sidecar_image              = "ghcr.io/demeter-run/doom-patrol-hydra:477d1ca9081693999580b2b7827df2fc87bb4cb8"
  open_head_image            = "ghcr.io/demeter-run/doom-patrol-hydra:477d1ca9081693999580b2b7827df2fc87bb4cb8"
  control_plane_image        = "ghcr.io/demeter-run/doom-patrol-hydra:477d1ca9081693999580b2b7827df2fc87bb4cb8"
  blockfrost_key             = var.blockfrost_key
  external_port              = 80
  admin_key_path             = "${path.module}/admin.sk"
  admin_addr                 = "addr_test1vpgcjapuwl7gfnzhzg6svtj0ph3gxu8kyuadudmf0kzsksqrfugfc"
  dmtr_project_id            = var.dmtr_project_id
  dmtr_api_key               = var.dmtr_api_key
  dmtr_port_name             = var.dmtr_port_name
  hydra_scripts_tx_id        = "03f8deb122fbbd98af8eb58ef56feda37728ec957d39586b78198a0cf624412a"
  init_image                 = "ghcr.io/demeter-run/doom-patrol-init:b7b4fc499b5274cd71b6b72f93ab4ba8199437fe"
  init_aws_access_key_id     = var.init_aws_access_key_id
  init_aws_secret_access_key = var.init_aws_secret_access_key
}
