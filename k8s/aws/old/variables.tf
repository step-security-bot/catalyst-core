variable "access_key" {
  description = "AWS ACCESS_KEY"
}

variable "secret_key" {
  description = "AWS SECRET_KEY"
}

variable "aws_region" {
  description = "AWS region to launch servers."
  default     = "eu-west-2"
}

variable "cidr_block" {
  description = "CIDR for the whole VPC"

  default = {
    prod    = "10.10.0.0/16"
    testnet = "10.20.0.0/16"
    dev     = "10.30.0.0/16"
  }
}

variable "eks_cluster_name" {
  description = "cluster name"
  default     = "catalyst"
}

variable "identifier" {
  description = "Identifier for DB"
  default     = "catalyst-db"
}

variable "storage_type" {
  description = "Type of the storage ssd or magnetic"
  default     = "gp2"
}

variable "allocated_storage" {
  description = "amount of storage allocated in GB"

  default = {
    prod = "100"
    testnet = "100"
    dev  = "10"
  }
}

variable "db_engine" {
  description = " DB engine"
  default     = "postgres"
}

variable "engine_version" {
  description = "DB engine version"
  default     = "14.4-R1"
}

variable "instance_class" {
  description = "machine type to be used"

  default = {
    prod = "db.t2.large"
    testnet  = "db.t2.micro"
    dev  = "db.t2.micro"
  }
}

variable "db_username" {
  description = "db admin user"
  default     = "root"
}

variable "db_password" {
  description = "password, provide through your tfvars file"
}
