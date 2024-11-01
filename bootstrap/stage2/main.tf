locals {
  operator_component      = "operator"
  configmap               = "hydra-pod-config"
  control_plane_component = "control-plane"
}

variable "namespace" {
  type    = string
  default = "hydra-doom"
}

variable "operator_image" {
  type = string
}

variable "hydra_node_image" {
  type    = string
  default = "ghcr.io/cardano-scaling/hydra-node"
}

variable "open_head_image" {
  type = string
}

variable "sidecar_image" {
  type = string
}

variable "control_plane_image" {
  type = string
}

variable "blockfrost_key" {
  type = string
}

variable "external_domain" {
  type = string
}

variable "external_port" {
  type = number
}

variable "tolerations" {
  type = list(object({
    effect   = string
    key      = string
    operator = string
    value    = optional(string)
  }))
  default = []
}

variable "resources" {
  type = object({
    limits = object({
      cpu    = optional(string)
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    requests = {
      cpu    = "500m"
      memory = "512Mi"
    }
    limits = {
      cpu    = "2"
      memory = "512Mi"
    }
  }
}

variable "control_plane_resources" {
  type = object({
    limits = object({
      cpu    = optional(string)
      memory = string
    })
    requests = object({
      cpu    = string
      memory = string
    })
  })
  default = {
    requests = {
      cpu    = "500m"
      memory = "512Mi"
    }
    limits = {
      cpu    = "2"
      memory = "512Mi"
    }
  }
}
