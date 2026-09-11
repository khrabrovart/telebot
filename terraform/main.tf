terraform {
  required_version = ">= 1.13"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.15"
    }
  }

  backend "s3" {
    bucket  = "kha-org-state"
    key     = "telebot/terraform.tfstate"
    region  = "us-east-1"
    encrypt = true
  }
}

provider "aws" {
  region = var.aws_region

  assume_role {
    role_arn = var.aws_assume_role_arn
  }

  default_tags {
    tags = {
      Project   = "Telebot"
      ManagedBy = "Terraform"
    }
  }
}

locals {
  app_name = "telebot"
}

data "aws_region" "current" {}
data "aws_caller_identity" "current" {}
