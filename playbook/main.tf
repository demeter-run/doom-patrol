# Terraform file for local development
#
# To test run this on a local machine run the following:
# 1. kind create cluster --name k8scluster
# 2. docker build -t doom-patrol-operator:local .. 
# 3. kind load docker-image doom-patrol-operator:local --name k8scluster
# 4. cd playbook && tf apply
locals {
  namespace      = "hydra-doom"
  operator_image = "doom-patrol-operator:local"
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
  config_path    = "~/.kube/config"
  config_context = "kind-k8scluster"
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

  namespace                 = local.namespace
  external_domain           = "external.domain"
  image                     = local.operator_image
  hydra_pod_open_head_image = ""
  blockfrost_key            = ""
  external_port             = 4001
}
