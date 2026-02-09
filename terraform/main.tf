terraform {
  required_version = ">= 1.13"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = "~> 6.15"
    }
  }

  backend "s3" {
    bucket  = "arturkhrabrov-tfstate"
    key     = "telebot/terraform.tfstate"
    region  = "us-east-1"
    encrypt = true
  }
}

provider "aws" {
  region = var.aws_region

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
