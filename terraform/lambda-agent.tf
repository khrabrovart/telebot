data "archive_file" "agent_lambda_zip" {
  type        = "zip"
  source_file = "../agent-lambda/bootstrap"
  output_path = "agent_lambda.zip"
}

resource "aws_lambda_function" "agent_lambda" {
  filename      = data.archive_file.agent_lambda_zip.output_path
  function_name = "${local.app_name}-agent"
  role          = aws_iam_role.agent_lambda_role.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  timeout       = 30
  memory_size   = 128
  architectures = ["arm64"]

  source_code_hash = data.archive_file.agent_lambda_zip.output_base64sha256

  environment {
    variables = {
      POSTING_RULES_TABLE    = aws_dynamodb_table.posting_rules.name
      BOTS_TABLE             = aws_dynamodb_table.bots.name
      POLLS_ACTION_LOG_TABLE = aws_dynamodb_table.polls_action_log.name
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.agent_lambda_basic_execution,
    aws_cloudwatch_log_group.agent_lambda_logs
  ]
}

resource "aws_iam_role" "agent_lambda_role" {
  name = "${local.app_name}-agent-lambda-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })
}


resource "aws_iam_policy" "agent_lambda_policy" {
  name = "${local.app_name}-agent-lambda-policy"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "dynamodb:GetItem",
          "dynamodb:PutItem",
          "dynamodb:UpdateItem",
          "dynamodb:DeleteItem",
          "dynamodb:Scan",
          "dynamodb:Query"
        ]
        Resource = [
          aws_dynamodb_table.posting_rules.arn,
          aws_dynamodb_table.bots.arn,
          aws_dynamodb_table.polls_action_log.arn
        ]
      },
    ]
  })
}

resource "aws_iam_role_policy_attachment" "agent_lambda_basic_execution" {
  role       = aws_iam_role.agent_lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_iam_role_policy_attachment" "agent_lambda_policy_attachment" {
  role       = aws_iam_role.agent_lambda_role.name
  policy_arn = aws_iam_policy.agent_lambda_policy.arn
}

resource "aws_cloudwatch_log_group" "agent_lambda_logs" {
  name              = "/aws/lambda/${local.app_name}-agent"
  retention_in_days = 14
}
