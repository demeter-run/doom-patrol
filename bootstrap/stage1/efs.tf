resource "kubernetes_storage_class" "efs_storage_class" {
  metadata {
    name = "efs-sc"
  }
  storage_provisioner = "efs.csi.aws.com"
  parameters = {
    provisioningMode      = "efs-ap"
    fileSystemId          = var.efs_fs_id
    directoryPerms        = "777"
    basePath              = "/hydra-node-persistance"
    subPathPattern        = "$${.PVC.name}"
    ensureUniqueDirectory = "true"
  }
}

resource "kubernetes_persistent_volume" "efs_pv" {
  metadata {
    name = "hydra-doom-persistence"
  }

  spec {
    capacity = {
      storage = "100Gi"
    }
    volume_mode                      = "Filesystem"
    access_modes                     = ["ReadWriteMany"]
    persistent_volume_reclaim_policy = "Retain"
    storage_class_name               = "efs-cs"
    persistent_volume_source {
      csi {
        driver        = "efs.csi.aws.com"
        volume_handle = var.efs_fs_id
      }
    }
  }
}
