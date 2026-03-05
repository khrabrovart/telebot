# Telebot

A serverless Telegram bot management platform built on AWS Lambda, enabling multi-bot administration, scheduled content distribution, and poll interaction tracking.

## Overview

Telebot is a production-grade system that manages multiple Telegram bots with automated scheduling, templated content posting, and comprehensive poll tracking. It leverages AWS serverless services for cost-effective, scalable operation.

**Key Capabilities:**
- **Multi-Bot Management**: Support for unlimited independent Telegram bots
- **Scheduled Posting**: CRON-based text and poll scheduling with timezone support
- **Content Templating**: Dynamic variable replacement (e.g., `{next_monday}`, `{next_friday}` in post content)
- **Poll Tracking**: Automatic logging of poll participant votes and interactions
- **Message Pinning**: Auto-pin important posts to channels
- **Dynamic Webhook Routing**: Automatic API Gateway routes per bot

## Architecture

Telebot is composed of 5 AWS Lambda functions plus a shared library, orchestrated via DynamoDB Streams and EventBridge Scheduler.

| Lambda Function | Purpose | Trigger | Key Responsibility |
|---|---|---|---|
| **Agent** | Webhook handler | HTTP API (Telegram updates) | Process messages, callback queries, poll answers; route to handlers; validate bot config |
| **Post-Create** | Scheduled posting | EventBridge Scheduler (CRON) | Create text posts/polls in channels; perform variable replacement; pin messages |
| **Scheduling** | Schedule sync | DynamoDB Streams (posting_rules table) | Create/update/delete EventBridge schedules when posting rules change |
| **Webhook-Sync** | Webhook management | DynamoDB Streams (bots table) | Register bots with Telegram; create/delete API Gateway routes |
| **Shared** | Common library | N/A | Data types, repositories, AWS utilities shared by all functions |

**Data Flow:**
1. Admin registers a bot in DynamoDB → Webhook-Sync creates API Gateway route and registers Telegram webhook
2. Admin creates a PostingRule → Scheduling Lambda creates EventBridge schedule
3. At scheduled time, EventBridge invokes Post-Create → sends content to Telegram
4. User interacts with content → Telegram sends update via webhook to Agent → Agent logs interaction in DynamoDB

## DynamoDB Tables

| Table Name | Hash Key | Range Key | Purpose | Streams |
|---|---|---|---|---|
| `telebot-bots` | `Id` | — | Bot configuration and tokens | ✓ (INSERT/DELETE) |
| `telebot-posting-rules` | `Id` | — | Scheduled posting configurations | ✓ (INSERT/UPDATE/DELETE) |
| `telebot-posts` | `ChatId` | `MessageId` | Records of sent messages | ✓ |
| `telebot-poll-action-log` | `Id` | — | Poll participation tracking | — |

**Environment Variables** (set by Terraform):
- `BOTS_TABLE`: DynamoDB bots table name
- `POSTING_RULES_TABLE`: DynamoDB posting rules table name
- `POSTS_TABLE`: DynamoDB posts table name
- `POLL_ACTION_LOG_TABLE`: DynamoDB poll action log table name
- `TARGET_LAMBDA_ARN`: Post-Create Lambda ARN (used by Scheduler)
- `SCHEDULER_ROLE_ARN`: EventBridge Scheduler execution role ARN
- `SCHEDULER_GROUP_NAME`: EventBridge Scheduler group name
- `SCHEDULE_PREFIX`: Prefix for generated schedule names

## License

See [LICENSE](LICENSE) file for details.
