variable "efs_fs_id" {
  type        = string
  description = "ID of EFS resource to use as persistance."
}

variable "efs_ap" {
  type        = string
  description = "Access point of corresponding EFS."
}
