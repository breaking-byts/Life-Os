use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
struct WgerExerciseResponse {
    results: Vec<WgerExercise>,
}

#[derive(Debug, Deserialize)]
struct WgerExercise {
    id: i64,
    #[serde(default)]
    category: Option<WgerCategory>,
    #[serde(default)]
    muscles: Vec<WgerMuscle>,
    #[serde(default)]
    equipment: Vec<WgerEquipment>,
    #[serde(default)]
    translations: Vec<WgerTranslation>,
}

#[derive(Debug, Deserialize)]
struct WgerTranslation {
    language: i64,
    name: String,
    #[serde(default)]
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WgerCategory {
    name: String,
}

#[derive(Debug, Deserialize)]
struct WgerMuscle {
    name: String,
}

#[derive(Debug, Deserialize)]
struct WgerEquipment {
    name: String,
}

#[derive(Debug, Serialize)]
pub struct ExerciseData {
    pub wger_id: i64,
    pub name: String,
    pub category: Option<String>,
    pub muscles: Option<String>,
    pub equipment: Option<String>,
    pub description: Option<String>,
}

/// Fetch exercises from wger.de API
/// Returns up to 1000 exercises filtered by English language
pub async fn fetch_exercises() -> Result<Vec<ExerciseData>, String> {
    let client = reqwest::Client::new();
    
    // Fetch exercise info from wger API (language=2 is English)
    let response = client
        .get("https://wger.de/api/v2/exerciseinfo/")
        .query(&[
            ("language", "2"),
            ("limit", "1000"),
        ])
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let data: WgerExerciseResponse = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    let exercises: Vec<ExerciseData> = data.results
        .into_iter()
        .filter_map(|ex| {
            // Find English translation (language == 2)
            let english = ex.translations.into_iter().find(|t| t.language == 2)?;
            
            Some(ExerciseData {
                wger_id: ex.id,
                name: english.name,
                category: ex.category.map(|c| c.name),
                muscles: if ex.muscles.is_empty() {
                    None
                } else {
                    Some(ex.muscles.iter().map(|m| m.name.clone()).collect::<Vec<_>>().join(", "))
                },
                equipment: if ex.equipment.is_empty() {
                    None
                } else {
                    Some(ex.equipment.iter().map(|e| e.name.clone()).collect::<Vec<_>>().join(", "))
                },
                description: english.description.filter(|d| !d.is_empty()),
            })
        })
        .collect();

    Ok(exercises)
}
