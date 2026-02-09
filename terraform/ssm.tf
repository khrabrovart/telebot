resource "aws_ssm_parameter" "bot_token" {
  name  = "/${local.app_name}/bot-token"
  type  = "SecureString"
  value = "placeholder"

  lifecycle {
    ignore_changes = [value]
  }
}
