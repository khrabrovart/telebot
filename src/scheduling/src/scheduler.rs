use anyhow::{anyhow, Error};
use aws_sdk_scheduler::{
    types::{FlexibleTimeWindow, FlexibleTimeWindowMode, RetryPolicy, ScheduleState, Target},
    Client,
};
use telebot_shared::{
    aws::errors::map_aws_error,
    data::{PostingRule, SchedulerEvent},
};

pub struct SchedulerClient {
    client: Client,
    group_name: String,
    target_lambda_arn: String,
    scheduler_role_arn: String,
    schedule_prefix: String,
}

impl SchedulerClient {
    pub async fn new() -> Result<Self, Error> {
        let target_lambda_arn = std::env::var("TARGET_LAMBDA_ARN")
            .map_err(|_| anyhow!("TARGET_LAMBDA_ARN environment variable not set"))?;

        let scheduler_role_arn = std::env::var("SCHEDULER_ROLE_ARN")
            .map_err(|_| anyhow!("SCHEDULER_ROLE_ARN environment variable not set"))?;

        let group_name = std::env::var("SCHEDULER_GROUP_NAME")
            .map_err(|_| anyhow!("SCHEDULER_GROUP_NAME environment variable not set"))?;

        let schedule_prefix = std::env::var("SCHEDULE_PREFIX")
            .map_err(|_| anyhow!("SCHEDULE_PREFIX environment variable not set"))?;

        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        Ok(Self {
            client,
            group_name,
            target_lambda_arn,
            scheduler_role_arn,
            schedule_prefix,
        })
    }

    fn schedule_name(&self, posting_rule_id: &str) -> String {
        format!("{}{}", self.schedule_prefix, posting_rule_id)
    }

    pub async fn create_or_update_schedule(
        &self,
        posting_rule: &PostingRule,
    ) -> Result<(), anyhow::Error> {
        let schedule_name = self.schedule_name(&posting_rule.id);
        let payload = SchedulerEvent {
            posting_rule_id: posting_rule.id.clone(),
        };
        let payload_json = serde_json::to_string(&payload)
            .map_err(|_| anyhow!("Failed to serialize scheduler payload"))?;

        let retry_policy = RetryPolicy::builder()
            .maximum_event_age_in_seconds(60)
            .maximum_retry_attempts(0)
            .build();

        let target = Target::builder()
            .arn(&self.target_lambda_arn)
            .role_arn(&self.scheduler_role_arn)
            .retry_policy(retry_policy)
            .input(payload_json)
            .build()
            .map_err(|e| anyhow!("Failed to build target: {e}"))?;

        let flexible_time_window = FlexibleTimeWindow::builder()
            .mode(FlexibleTimeWindowMode::Off)
            .build()
            .map_err(|_| anyhow!("Failed to build flexible time window"))?;

        let schedule_expression = format!("cron({})", posting_rule.schedule.trim());

        let state = if posting_rule.is_active {
            ScheduleState::Enabled
        } else {
            ScheduleState::Disabled
        };

        let schedule_exists = self.schedule_exists(&schedule_name).await?;

        if schedule_exists {
            self.client
                .update_schedule()
                .group_name(&self.group_name)
                .name(&schedule_name)
                .state(state)
                .schedule_expression(&schedule_expression)
                .schedule_expression_timezone(&posting_rule.timezone)
                .target(target)
                .flexible_time_window(flexible_time_window)
                .send()
                .await
                .map_err(map_aws_error)?;
        } else {
            self.client
                .create_schedule()
                .group_name(&self.group_name)
                .name(&schedule_name)
                .state(state)
                .schedule_expression(&schedule_expression)
                .schedule_expression_timezone(&posting_rule.timezone)
                .target(target)
                .flexible_time_window(flexible_time_window)
                .send()
                .await
                .map_err(map_aws_error)?;
        }

        Ok(())
    }

    pub async fn delete_schedule(&self, posting_rule_id: &str) -> Result<(), Error> {
        let schedule_name = self.schedule_name(posting_rule_id);

        if !self.schedule_exists(&schedule_name).await? {
            return Ok(());
        }

        self.client
            .delete_schedule()
            .group_name(&self.group_name)
            .name(&schedule_name)
            .send()
            .await
            .map_err(map_aws_error)?;

        Ok(())
    }

    async fn schedule_exists(&self, schedule_name: &str) -> Result<bool, Error> {
        match self
            .client
            .get_schedule()
            .group_name(&self.group_name)
            .name(schedule_name)
            .send()
            .await
        {
            Ok(_) => Ok(true),
            Err(e) => {
                let service_error = e.into_service_error();
                if service_error.is_resource_not_found_exception() {
                    Ok(false)
                } else {
                    Err(anyhow!(service_error.to_string()))
                }
            }
        }
    }
}
