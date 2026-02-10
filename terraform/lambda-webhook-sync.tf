data "archive_file" "webhook_sync_lambda_zip" {
  type        = "zip"
  source_file = "../webhook-sync-lambda/bootstrap"
  output_path = "webhook_sync_lambda.zip"
}

resource "aws_lambda_function" "webhook_sync_lambda" {
  filename      = data.archive_file.webhook_sync_lambda_zip.output_path
  function_name = "${local.app_name}-webhook-sync"
  role          = aws_iam_role.webhook_sync_lambda_role.arn
  handler       = "bootstrap"
  runtime       = "provided.al2023"
  timeout       = 30
  memory_size   = 128
  architectures = ["arm64"]

  source_code_hash = data.archive_file.webhook_sync_lambda_zip.output_base64sha256

  environment {
    variables = {
      API_ID             = aws_apigatewayv2_api.webhook_api.id
      API_INTEGRATION_ID = aws_apigatewayv2_integration.webhook_lambda_integration.id
      ROUTE_PREFIX       = "/webhook/"
    }
  }

  depends_on = [
    aws_iam_role_policy_attachment.webhook_sync_lambda_basic_execution,
    aws_cloudwatch_log_group.webhook_sync_lambda_logs
  ]
}

resource "aws_iam_role" "webhook_sync_lambda_role" {
  name = "${local.app_name}-webhook-sync-lambda-role"

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

resource "aws_iam_policy" "webhook_sync_lambda_policy" {
  name = "${local.app_name}-webhook-sync-lambda-policy"

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "dynamodb:GetRecords",
          "dynamodb:GetShardIterator",
          "dynamodb:DescribeStream",
          "dynamodb:ListStreams"
        ]
        Resource = [
          "${aws_dynamodb_table.posting_rules.arn}/stream/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "scheduler:GetSchedule",
          "scheduler:CreateSchedule",
          "scheduler:UpdateSchedule",
          "scheduler:DeleteSchedule"
        ]
        Resource = [
          "arn:aws:scheduler:${data.aws_region.current.id}:${data.aws_caller_identity.current.account_id}:schedule/${aws_scheduler_schedule_group.scheduler_group.name}/*"
        ]
      },
      {
        Effect = "Allow"
        Action = [
          "iam:PassRole"
        ]
        Resource = [
          aws_iam_role.scheduler_role.arn
        ]
      }
    ]
  })
}

resource "aws_iam_role_policy_attachment" "webhook_sync_lambda_policy_attachment" {
  role       = aws_iam_role.webhook_sync_lambda_role.name
  policy_arn = aws_iam_policy.webhook_sync_lambda_policy.arn
}

resource "aws_iam_role_policy_attachment" "webhook_sync_lambda_basic_execution" {
  role       = aws_iam_role.webhook_sync_lambda_role.name
  policy_arn = "arn:aws:iam::aws:policy/service-role/AWSLambdaBasicExecutionRole"
}

resource "aws_cloudwatch_log_group" "webhook_sync_lambda_logs" {
  name              = "/aws/lambda/${local.app_name}-webhook-sync"
  retention_in_days = 14
}

resource "aws_lambda_event_source_mapping" "dynamodb_stream" {
  event_source_arn                   = aws_dynamodb_table.bots.stream_arn
  function_name                      = aws_lambda_function.webhook_sync_lambda.arn
  starting_position                  = "LATEST"
  batch_size                         = 1
  maximum_batching_window_in_seconds = 5
  maximum_retry_attempts             = 1
}
