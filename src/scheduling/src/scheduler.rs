use crate::Post;
use aws_sdk_scheduler::{
    types::{FlexibleTimeWindow, FlexibleTimeWindowMode, Target},
    Client,
};
use serde::Serialize;
use thiserror::Error;
use tracing::info;

#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("Failed to create schedule: {0}")]
    CreateScheduleFailed(String),

    #[error("Failed to update schedule: {0}")]
    UpdateScheduleFailed(String),

    #[error("Failed to delete schedule: {0}")]
    DeleteScheduleFailed(String),

    #[error("Failed to get schedule: {0}")]
    GetScheduleFailed(String),
}

#[derive(Debug, Serialize)]
pub struct SchedulerPayload {
    pub posting_data_id: String,
}

pub struct SchedulerClient {
    client: Client,
    group_name: String,
    target_lambda_arn: String,
    scheduler_role_arn: String,
    schedule_prefix: String,
}

impl SchedulerClient {
    pub async fn new() -> Self {
        let target_lambda_arn = std::env::var("TARGET_LAMBDA_ARN")
            .expect("TARGET_LAMBDA_ARN environment variable not set");

        let scheduler_role_arn = std::env::var("SCHEDULER_ROLE_ARN")
            .expect("SCHEDULER_ROLE_ARN environment variable not set");

        let group_name = std::env::var("SCHEDULER_GROUP_NAME")
            .expect("SCHEDULER_GROUP_NAME environment variable not set");

        let schedule_prefix =
            std::env::var("SCHEDULE_PREFIX").expect("SCHEDULE_PREFIX environment variable not set");

        let config = aws_config::load_from_env().await;
        let client = Client::new(&config);

        Self {
            client,
            group_name,
            target_lambda_arn,
            scheduler_role_arn,
            schedule_prefix,
        }
    }

    fn schedule_name(&self, posting_id: &str) -> String {
        format!("{}-posting-{}", self.schedule_prefix, posting_id)
    }

    pub async fn create_or_update_schedule(&self, post: &Post) -> Result<(), SchedulerError> {
        let schedule_name = self.schedule_name(&post.id);
        let payload = SchedulerPayload {
            posting_data_id: post.id.clone(),
        };
        let payload_json =
            serde_json::to_string(&payload).expect("Failed to serialize scheduler payload");

        let target = Target::builder()
            .arn(&self.target_lambda_arn)
            .role_arn(&self.scheduler_role_arn)
            .input(payload_json)
            .build()
            .expect("Failed to build target");

        let flexible_time_window = FlexibleTimeWindow::builder()
            .mode(FlexibleTimeWindowMode::Off)
            .build()
            .expect("Failed to build flexible time window");

        let schedule_expression = format!("cron({})", post.schedule.trim());

        let state = if post.is_active {
            ScheduleState::Active
        } else {
            ScheduleState::Inactive
        };

        let schedule_exists = self.schedule_exists(&schedule_name).await?;

        if schedule_exists {
            info!(schedule_name = %schedule_name, "Updating existing schedule");
            self.client
                .update_schedule()
                .group_name(&self.group_name)
                .name(&schedule_name)
                .state(state)
                .schedule_expression(&schedule_expression)
                .schedule_expression_timezone(&post.timezone)
                .target(target)
                .flexible_time_window(flexible_time_window)
                .send()
                .await
                .map_err(|e| SchedulerError::UpdateScheduleFailed(e.to_string()))?;
        } else {
            info!(schedule_name = %schedule_name, "Creating new schedule");
            self.client
                .create_schedule()
                .group_name(&self.group_name)
                .name(&schedule_name)
                .state(state)
                .schedule_expression(&schedule_expression)
                .schedule_expression_timezone(&post.timezone)
                .target(target)
                .flexible_time_window(flexible_time_window)
                .send()
                .await
                .map_err(|e| SchedulerError::CreateScheduleFailed(e.to_string()))?;
        }

        info!(schedule_name = %schedule_name, schedule_expression = %schedule_expression, "Schedule configured successfully");
        Ok(())
    }

    pub async fn delete_schedule(&self, posting_id: &str) -> Result<(), SchedulerError> {
        let schedule_name = self.schedule_name(posting_id);

        if !self.schedule_exists(&schedule_name).await? {
            info!(schedule_name = %schedule_name, "Schedule does not exist, nothing to delete");
            return Ok(());
        }

        info!(schedule_name = %schedule_name, "Deleting schedule");
        self.client
            .delete_schedule()
            .group_name(&self.group_name)
            .name(&schedule_name)
            .send()
            .await
            .map_err(|e| SchedulerError::DeleteScheduleFailed(e.to_string()))?;

        info!(schedule_name = %schedule_name, "Schedule deleted successfully");
        Ok(())
    }

    async fn schedule_exists(&self, schedule_name: &str) -> Result<bool, SchedulerError> {
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
                    Err(SchedulerError::GetScheduleFailed(service_error.to_string()))
                }
            }
        }
    }
}
