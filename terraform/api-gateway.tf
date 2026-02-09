resource "aws_api_gateway_rest_api" "agent_api" {
  name = "${local.app_name}-agent-api"

  endpoint_configuration {
    types = ["REGIONAL"]
  }
}

resource "aws_api_gateway_resource" "agent_receive_resource" {
  rest_api_id = aws_api_gateway_rest_api.agent_api.id
  parent_id   = aws_api_gateway_rest_api.agent_api.root_resource_id
  path_part   = "receive"
}

resource "aws_api_gateway_method" "agent_receive_post_method" {
  rest_api_id   = aws_api_gateway_rest_api.agent_api.id
  resource_id   = aws_api_gateway_resource.agent_receive_resource.id
  http_method   = "POST"
  authorization = "NONE"
}

resource "aws_api_gateway_integration" "agent_receive_integration" {
  rest_api_id = aws_api_gateway_rest_api.agent_api.id
  resource_id = aws_api_gateway_resource.agent_receive_resource.id
  http_method = aws_api_gateway_method.agent_receive_post_method.http_method

  integration_http_method = "POST"
  type                    = "AWS_PROXY"
  uri                     = aws_lambda_function.agent_lambda.invoke_arn
}

resource "aws_lambda_permission" "agent_api_gateway_invoke" {
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.agent_lambda.function_name
  principal     = "apigateway.amazonaws.com"
  source_arn    = "${aws_api_gateway_rest_api.agent_api.execution_arn}/*/POST/receive"
}

resource "aws_api_gateway_deployment" "agent_api_deployment" {
  rest_api_id = aws_api_gateway_rest_api.agent_api.id

  lifecycle {
    create_before_destroy = true
  }

  depends_on = [
    aws_api_gateway_integration.agent_receive_integration,
  ]
}

resource "aws_api_gateway_stage" "agent_api_stage_hook" {
  deployment_id = aws_api_gateway_deployment.agent_api_deployment.id
  rest_api_id   = aws_api_gateway_rest_api.agent_api.id
  stage_name    = "hook"
}
