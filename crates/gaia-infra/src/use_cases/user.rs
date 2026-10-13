use anyhow::Result;
use sqlx::PgPool;
use crate::Infrastructure;

pub struct CreateUserCommand {
    pub username: String,
    pub email: String,
}

pub struct CreateUserDto {
    pub id: i32,
    pub username: String,
    pub email: String,
}

pub struct CreateUserUseCase;

impl CreateUserUseCase {
    pub async fn execute(
        infra: &Infrastructure,
        cmd: CreateUserCommand,
    ) -> Result<CreateUserDto> {
        let pool = infra.database();
        let id = 1;

        let dto = sqlx::query_as!(
            CreateUserDto,
            "INSERT INTO users (id, username, email) VALUES ($1, $2, $3) RETURNING id, username, email",
            id,
            cmd.username,
            cmd.email
        )
        .fetch_one(pool)
        .await
        .map(|r| CreateUserDto {
            id: r.id,
            username: r.username,
            email: r.email,
        })
        .context("Failed to create user")?;

        Ok(dto)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_user_command_has_valid_fields() {
        let cmd = CreateUserCommand {
            username: "testuser".to_string(),
            email: "test@example.com".to_string(),
        };
        assert_eq!(cmd.username, "testuser");
        assert_eq!(cmd.email, "test@example.com");
    }
}