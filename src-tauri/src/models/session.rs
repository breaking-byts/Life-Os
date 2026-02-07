use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SessionType {
    Study,
    Practice,
}

impl sqlx::Type<sqlx::Sqlite> for SessionType {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for SessionType {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let value = match self {
            SessionType::Study => "study",
            SessionType::Practice => "practice",
        };
        <&str as sqlx::Encode<sqlx::Sqlite>>::encode_by_ref(&value, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for SessionType {
    fn decode(
        value: sqlx::sqlite::SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let raw = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match raw.as_str() {
            "study" => Ok(SessionType::Study),
            "practice" => Ok(SessionType::Practice),
            other => Err(format!("invalid session_type: {}", other).into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct Session {
    pub id: i64,
    pub user_id: i64,
    pub session_type: SessionType,
    pub reference_id: Option<i64>,
    pub reference_type: Option<String>,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub duration_minutes: Option<i64>,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::SessionType;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn session_type_serde_roundtrip() {
        let json = serde_json::to_string(&SessionType::Study).unwrap();
        assert_eq!(json, "\"study\"");

        let parsed: SessionType = serde_json::from_str("\"practice\"").unwrap();
        assert_eq!(parsed, SessionType::Practice);

        assert!(serde_json::from_str::<SessionType>("\"invalid\"").is_err());
    }

    #[tokio::test]
    async fn session_type_sqlx_roundtrip() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let value: SessionType = sqlx::query_scalar("SELECT ?")
            .bind(SessionType::Study)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(value, SessionType::Study);
    }
}
