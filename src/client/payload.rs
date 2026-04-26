use crate::models::{Preferences, ProfileUpdate};
use serde_json::json;

/// Convert ProfileUpdate to API format with numeric enum values
pub(super) fn profile_update_to_api_json(update: &ProfileUpdate) -> serde_json::Value {
    use crate::enums::ApiEnum;

    let mut obj = serde_json::Map::new();

    // Convert each field, using the enum's to_api_value() when needed
    if let Some(ref children) = update.children {
        obj.insert(
            "children".to_string(),
            json!({
                "value": children.value.to_api_value(),
                "visible": children.visible
            }),
        );
    }

    if let Some(ref dating) = update.dating_intention {
        obj.insert(
            "datingIntention".to_string(),
            json!({
                "value": dating.value.to_api_value(),
                "visible": dating.visible
            }),
        );
    }

    if let Some(ref drinking) = update.drinking {
        obj.insert(
            "drinking".to_string(),
            json!({
                "value": drinking.value.to_api_value(),
                "visible": drinking.visible
            }),
        );
    }

    if let Some(ref drugs) = update.drugs {
        obj.insert(
            "drugs".to_string(),
            json!({
                "value": drugs.value.to_api_value(),
                "visible": drugs.visible
            }),
        );
    }

    if let Some(ref marijuana) = update.marijuana {
        obj.insert(
            "marijuana".to_string(),
            json!({
                "value": marijuana.value.to_api_value(),
                "visible": marijuana.visible
            }),
        );
    }

    if let Some(ref smoking) = update.smoking {
        obj.insert(
            "smoking".to_string(),
            json!({
                "value": smoking.value.to_api_value(),
                "visible": smoking.visible
            }),
        );
    }

    if let Some(ref politics) = update.politics {
        obj.insert(
            "politics".to_string(),
            json!({
                "value": politics.value.to_api_value(),
                "visible": politics.visible
            }),
        );
    }

    if let Some(ref religions) = update.religions {
        let values: Vec<i8> = religions.value.iter().map(|e| e.to_api_value()).collect();
        obj.insert(
            "religions".to_string(),
            json!({
                "value": values,
                "visible": religions.visible
            }),
        );
    }

    if let Some(ref ethnicities) = update.ethnicities {
        let values: Vec<i8> = ethnicities.value.iter().map(|e| e.to_api_value()).collect();
        obj.insert(
            "ethnicities".to_string(),
            json!({
                "value": values,
                "visible": ethnicities.visible
            }),
        );
    }

    if let Some(ref education) = update.education_attained {
        obj.insert(
            "educationAttained".to_string(),
            json!(education.to_api_value()),
        );
    }

    if let Some(ref relationships) = update.relationship_type_ids {
        let values: Vec<i8> = relationships
            .value
            .iter()
            .map(|e| e.to_api_value())
            .collect();
        obj.insert(
            "relationshipTypeIds".to_string(),
            json!({
                "value": values,
                "visible": relationships.visible
            }),
        );
    }

    if let Some(height) = update.height {
        obj.insert("height".to_string(), json!(height));
    }

    if let Some(ref gender) = update.gender_id {
        obj.insert("genderId".to_string(), json!(gender.to_api_value()));
    }

    if let Some(ref hometown) = update.hometown {
        obj.insert(
            "hometown".to_string(),
            json!({
                "value": hometown.value,
                "visible": hometown.visible
            }),
        );
    }

    if let Some(ref languages) = update.languages_spoken {
        obj.insert(
            "languagesSpoken".to_string(),
            json!({
                "value": languages.value,
                "visible": languages.visible
            }),
        );
    }

    if let Some(ref zodiac) = update.zodiac {
        obj.insert(
            "zodiac".to_string(),
            json!({
                "value": zodiac.value,
                "visible": zodiac.visible
            }),
        );
    }

    serde_json::Value::Object(obj)
}

/// Convert Preferences to API format with numeric enum values
pub(super) fn preferences_to_api_json(prefs: &Preferences) -> serde_json::Value {
    use crate::enums::ApiEnum;

    json!({
        "genderedAgeRanges": prefs.gendered_age_ranges,
        "dealbreakers": prefs.dealbreakers,
        "religions": prefs.religions.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "drinking": prefs.drinking.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "genderedHeightRanges": prefs.gendered_height_ranges,
        "marijuana": prefs.marijuana.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "relationshipTypes": prefs.relationship_types.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "drugs": prefs.drugs.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "maxDistance": prefs.max_distance,
        "children": prefs.children.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "ethnicities": prefs.ethnicities.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "smoking": prefs.smoking.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "educationAttained": prefs.education_attained.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "familyPlans": prefs.family_plans,
        "datingIntentions": prefs.dating_intentions.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "politics": prefs.politics.iter().map(|e| e.to_api_value()).collect::<Vec<_>>(),
        "genderPreferences": prefs.gender_preferences.iter().map(|e| e.to_api_value()).collect::<Vec<_>>()
    })
}
