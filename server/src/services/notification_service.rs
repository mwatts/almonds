use std::sync::Arc;

use lunar::entities::notifications;
use sea_orm::DatabaseConnection;
use uuid::Uuid;

use crate::{
    adapters::{
        jwt::Claims,
        pagination::{PaginatedResponse, PaginationParams},
    },
    dto::common::RowCount,
    errors::service_error::ServiceError,
    repositories::{
        base::Repository,
        notification::{NotificationRepository, NotificationRepositoryExt},
    },
};

#[derive(Clone)]
pub struct NotificationService {
    repository: NotificationRepository,
}

impl NotificationService {
    pub fn new(repository: NotificationRepository) -> Self {
        Self { repository }
    }

    pub fn init(db_conn: &Arc<DatabaseConnection>) -> Self {
        Self {
            repository: NotificationRepository::init(db_conn),
        }
    }
}

#[allow(dead_code)]
pub(crate) trait NotificationServiceExt {
    async fn fetch_one(&self, notification_identifier: &Uuid) -> Option<notifications::Model>;

    async fn fetch_notifications(
        &self,
        claims: &Claims,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResponse<Vec<notifications::Model>>, ServiceError>;

    async fn count_unread(&self, claims: &Claims) -> Result<RowCount, ServiceError>;

    async fn mark_read(
        &self,
        claims: &Claims,
        notification_identifier: &Uuid,
    ) -> Result<(), ServiceError>;
}

impl NotificationServiceExt for NotificationService {
    async fn fetch_one(&self, notification_identifier: &Uuid) -> Option<notifications::Model> {
        self.repository.fetch_one(notification_identifier).await
    }

    async fn fetch_notifications(
        &self,
        _claims: &Claims,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResponse<Vec<notifications::Model>>, ServiceError> {
        let records = self.repository.fetch_all(pagination).await?;

        let paginated_result = PaginatedResponse {
            records: records.notifications,
            page: pagination.page(),
            per_page: pagination.per_page(),
            total_count: records.total as u64,
            total_pages: records.total as u32 / pagination.per_page(),
        };
        Ok(paginated_result)
    }

    async fn count_unread(&self, _claims: &Claims) -> Result<RowCount, ServiceError> {
        let result = self.repository.count_unread().await?;

        Ok(result)
    }

    async fn mark_read(
        &self,
        _claims: &Claims,
        notification_identifier: &Uuid,
    ) -> Result<(), ServiceError> {
        self.repository.mark_read(notification_identifier).await?;
        Ok(())
    }
}
