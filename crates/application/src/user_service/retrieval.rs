use super::*;

impl UserService {
    /// Returns a user record by ID, if it exists.
    pub async fn find_by_id(&self, user_id: UserId) -> AppResult<Option<UserRecord>> {
        self.user_repository.find_by_id(user_id).await
    }

    /// Returns a user record by email, if it exists.
    pub async fn find_by_email(&self, email: &str) -> AppResult<Option<UserRecord>> {
        self.user_repository.find_by_email(email).await
    }
}
