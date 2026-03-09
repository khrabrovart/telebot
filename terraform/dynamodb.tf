resource "aws_dynamodb_table" "bots" {
  name             = "${local.app_name}-bots"
  billing_mode     = "PAY_PER_REQUEST"
  hash_key         = "Id"
  stream_enabled   = true
  stream_view_type = "NEW_AND_OLD_IMAGES"

  attribute {
    name = "Id"
    type = "S"
  }
  server_side_encryption {
    enabled = true
  }
}

resource "aws_dynamodb_table" "poll_action_log" {
  name         = "${local.app_name}-poll-action-log"
  billing_mode = "PAY_PER_REQUEST"
  hash_key     = "Id"

  attribute {
    name = "Id"
    type = "S"
  }

  attribute {
    name = "ChatId"
    type = "N"
  }

  attribute {
    name = "MessageId"
    type = "N"
  }

  ttl {
    attribute_name = "ExpiresAt"
    enabled        = true
  }

  global_secondary_index {
    name            = "ChatMessageIndex"
    hash_key        = "ChatId"
    range_key       = "MessageId"
    projection_type = "ALL"
  }
}

resource "aws_dynamodb_table" "posting_rules" {
  name             = "${local.app_name}-posting-rules"
  billing_mode     = "PAY_PER_REQUEST"
  hash_key         = "Id"
  stream_enabled   = true
  stream_view_type = "NEW_AND_OLD_IMAGES"

  attribute {
    name = "Id"
    type = "S"
  }
}

resource "aws_dynamodb_table" "posts" {
  name             = "${local.app_name}-posts"
  billing_mode     = "PAY_PER_REQUEST"
  hash_key         = "ChatId"
  range_key        = "MessageId"
  stream_enabled   = true
  stream_view_type = "NEW_AND_OLD_IMAGES"

  attribute {
    name = "ChatId"
    type = "N"
  }

  attribute {
    name = "MessageId"
    type = "N"
  }

  attribute {
    name = "PostingRuleId"
    type = "S"
  }

  attribute {
    name = "Timestamp"
    type = "N"
  }

  ttl {
    attribute_name = "ExpiresAt"
    enabled        = true
  }

  global_secondary_index {
    name            = "PostingRuleIndex"
    hash_key        = "PostingRuleId"
    range_key       = "Timestamp"
    projection_type = "ALL"
  }
}
