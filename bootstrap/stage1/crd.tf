resource "kubernetes_manifest" "customresourcedefinition_hydradoompods_hydra_doom" {
  manifest = {
    "apiVersion" = "apiextensions.k8s.io/v1"
    "kind" = "CustomResourceDefinition"
    "metadata" = {
      "name" = "hydradoompods.hydra.doom"
    }
    "spec" = {
      "group" = "hydra.doom"
      "names" = {
        "categories" = [
          "hydradoom",
        ]
        "kind" = "HydraDoomPod"
        "plural" = "hydradoompods"
        "shortNames" = [
          "hydradoompod",
        ]
        "singular" = "hydradoompod"
      }
      "scope" = "Namespaced"
      "versions" = [
        {
          "additionalPrinterColumns" = [
            {
              "jsonPath" = ".status.localUrl"
              "name" = "Local URI"
              "type" = "string"
            },
            {
              "jsonPath" = ".status.externalUrl"
              "name" = "External URI"
              "type" = "string"
            },
          ]
          "name" = "v1alpha1"
          "schema" = {
            "openAPIV3Schema" = {
              "description" = "Auto-generated derived type for HydraDoomPodSpec via `CustomResource`"
              "properties" = {
                "spec" = {
                  "properties" = {
                    "blockfrostKey" = {
                      "nullable" = true
                      "type" = "string"
                    }
                    "commitInputs" = {
                      "items" = {
                        "type" = "string"
                      }
                      "type" = "array"
                    }
                    "configmap" = {
                      "nullable" = true
                      "type" = "string"
                    }
                    "image" = {
                      "nullable" = true
                      "type" = "string"
                    }
                    "networkId" = {
                      "format" = "uint8"
                      "minimum" = 0
                      "type" = "integer"
                    }
                    "openHeadImage" = {
                      "nullable" = true
                      "type" = "string"
                    }
                    "participant" = {
                      "type" = "string"
                    }
                    "party" = {
                      "type" = "string"
                    }
                    "seedInput" = {
                      "type" = "string"
                    }
                  }
                  "required" = [
                    "commitInputs",
                    "networkId",
                    "participant",
                    "party",
                    "seedInput",
                  ]
                  "type" = "object"
                }
                "status" = {
                  "nullable" = true
                  "properties" = {
                    "externalUrl" = {
                      "type" = "string"
                    }
                    "localUrl" = {
                      "type" = "string"
                    }
                  }
                  "required" = [
                    "externalUrl",
                    "localUrl",
                  ]
                  "type" = "object"
                }
              }
              "required" = [
                "spec",
              ]
              "title" = "HydraDoomPod"
              "type" = "object"
            }
          }
          "served" = true
          "storage" = true
          "subresources" = {
            "status" = {}
          }
        },
      ]
    }
  }
}
