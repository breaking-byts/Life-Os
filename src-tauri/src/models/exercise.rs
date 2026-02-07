use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExerciseSource {
    Wger,
    Custom,
}

impl sqlx::Type<sqlx::Sqlite> for ExerciseSource {
    fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }

    fn compatible(ty: &sqlx::sqlite::SqliteTypeInfo) -> bool {
        <String as sqlx::Type<sqlx::Sqlite>>::compatible(ty)
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Sqlite> for ExerciseSource {
    fn encode_by_ref(
        &self,
        buf: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
    ) -> Result<sqlx::encode::IsNull, sqlx::error::BoxDynError> {
        let value = match self {
            ExerciseSource::Wger => "wger",
            ExerciseSource::Custom => "custom",
        };
        <&str as sqlx::Encode<sqlx::Sqlite>>::encode_by_ref(&value, buf)
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Sqlite> for ExerciseSource {
    fn decode(
        value: sqlx::sqlite::SqliteValueRef<'r>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let raw = <String as sqlx::Decode<sqlx::Sqlite>>::decode(value)?;
        match raw.as_str() {
            "wger" => Ok(ExerciseSource::Wger),
            "custom" => Ok(ExerciseSource::Custom),
            other => Err(format!("invalid exercise source: {}", other).into()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct ExerciseCache {
    pub id: i64,
    pub wger_id: Option<i64>,
    pub name: String,
    pub category: Option<String>,
    pub muscles: Option<String>,
    pub equipment: Option<String>,
    pub description: Option<String>,
    pub cached_at: Option<String>,
    pub source: String,
    pub created_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::ExerciseSource;
    use sqlx::sqlite::SqlitePoolOptions;

    #[test]
    fn exercise_source_serde_roundtrip() {
        let json = serde_json::to_string(&ExerciseSource::Wger).unwrap();
        assert_eq!(json, "\"wger\"");

        let parsed: ExerciseSource = serde_json::from_str("\"custom\"").unwrap();
        assert_eq!(parsed, ExerciseSource::Custom);

        assert!(serde_json::from_str::<ExerciseSource>("\"invalid\"").is_err());
    }

    #[tokio::test]
    async fn exercise_source_sqlx_roundtrip() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        let value: ExerciseSource = sqlx::query_scalar("SELECT ?")
            .bind(ExerciseSource::Custom)
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(value, ExerciseSource::Custom);
    }

    #[tokio::test]
    async fn exercise_source_decodes_from_text_column() {
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        sqlx::query("CREATE TABLE exercises_cache (source TEXT NOT NULL)")
            .execute(&pool)
            .await
            .unwrap();

        sqlx::query("INSERT INTO exercises_cache (source) VALUES ('wger')")
            .execute(&pool)
            .await
            .unwrap();

        let value: ExerciseSource = sqlx::query_scalar("SELECT source FROM exercises_cache")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert_eq!(value, ExerciseSource::Wger);
    }
}
