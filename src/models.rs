use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, de::Deserializer};
use serde_json::Value as JsonValue;

use crate::enums::{
    ChildrenStatusPreference, ChildrenStatusProfile, ContentType, DatingIntentionPreference,
    DatingIntentionProfile, DrinkingStatusPreference, DrinkingStatusProfile, DrugStatusPreference,
    DrugStatusProfile, EducationAttainedPreference, EducationAttainedProfile, EthnicityPreference,
    EthnicityProfile, GenderEnum, GenderPreferences, MarijuanaStatusPreference,
    MarijuanaStatusProfile, PoliticsPreference, PoliticsProfile, RelationshipTypePreference,
    RelationshipTypeProfile, ReligionPreference, ReligionProfile, SmokingStatusPreference,
    SmokingStatusProfile,
};

// Wrapper structs for status fields that include visibility
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChildrenStatus {
    pub value: ChildrenStatusProfile,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DatingIntention {
    pub value: DatingIntentionProfile,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrinkingStatus {
    pub value: DrinkingStatusProfile,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DrugStatus {
    pub value: DrugStatusProfile,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MarijuanaStatus {
    pub value: MarijuanaStatusProfile,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmokingStatus {
    pub value: SmokingStatusProfile,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Religion {
    pub value: Vec<ReligionProfile>,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Politics {
    pub value: PoliticsProfile,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Pronouns {
    pub value: Vec<i32>, // TODO: Define PronounsEnum once we know the values
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationshipTypeIds {
    pub value: Vec<RelationshipTypeProfile>,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Ethnicities {
    pub value: Vec<EthnicityProfile>,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Educations {
    pub value: Vec<String>,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SexualOrientations {
    pub value: Vec<i32>, // TODO: Define proper enum
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Works {
    pub value: String,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FamilyPlans {
    pub value: i32,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenderIdentityId {
    pub value: i32,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Hometown {
    pub value: String,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobTitle {
    pub value: String,
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LanguagesSpoken {
    pub value: Vec<i32>, // Language enum IDs
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Zodiac {
    pub value: i32, // Zodiac sign enum ID
    pub visible: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Location {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub longitude: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "metroArea")]
    pub metro_area: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "metroAreaV2"
    )]
    pub metro_area_v2: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "countryShort"
    )]
    pub country_short: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "adminArea1Long"
    )]
    pub admin_area1_long: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "adminArea1Short"
    )]
    pub admin_area1_short: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "adminArea2"
    )]
    pub admin_area2: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HingeAuthToken {
    pub identity_id: String,
    pub token: String,
    pub expires: DateTime<Utc>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendbirdAuthToken {
    pub token: String,
    pub expires: DateTime<Utc>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginTokens {
    #[serde(default)]
    pub hinge_auth_token: Option<HingeAuthToken>,
    #[serde(default)]
    pub sendbird_auth_token: Option<SendbirdAuthToken>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeLimit {
    pub likes: i32,
    #[serde(default)]
    pub superlikes: Option<i32>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationSubject {
    pub subject_id: String,
    pub rating_token: String,
    #[serde(default)]
    pub origin: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsFeed {
    pub id: i32,
    pub origin: String,
    pub subjects: Vec<RecommendationSubject>,
    #[serde(default)]
    pub permission: Option<String>,
    #[serde(default)]
    pub preview: Option<RecommendationsPreview>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsResponse {
    pub feeds: Vec<RecommendationsFeed>,
    #[serde(default)]
    pub active_pills: Option<Vec<ActivePill>>,
    #[serde(default)]
    pub cache_control: Option<serde_json::Value>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileName {
    pub first_name: String,
    #[serde(default)]
    pub last_name: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Coordinate {
    #[serde(default)]
    pub x: f64,
    #[serde(default)]
    pub y: f64,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoundingBox {
    #[serde(default)]
    pub top_left: Coordinate,
    #[serde(default)]
    pub bottom_right: Coordinate,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Feedback {
    pub evaluation: String,
    pub detail: String,
    pub feedback_token: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoAsset {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdn_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfie_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub p_hash: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAnswer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub question_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "type")]
    pub answer_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feedback: Option<Feedback>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription_metadata: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdn_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub waveform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transcription: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptPoll {
    pub content_id: String,
    pub question_id: String,
    pub options: Vec<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VideoPrompt {
    pub content_id: String,
    pub question_id: String,
    pub thumbnail_url: String,
    pub video_url: String,
    pub cdn_id: String,
    pub bounding_box: BoundingBox,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileContent {
    pub name: ProfileName,
    #[serde(default)]
    pub photos: Vec<PhotoAsset>,
    #[serde(default)]
    pub answers: Vec<ProfileAnswer>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Profile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub covid_vax: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<ChildrenStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dating_intention: Option<DatingIntention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dating_intention_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drinking: Option<DrinkingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drugs: Option<DrugStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education_attained: Option<EducationAttainedProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educations: Option<Educations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethnicities: Option<Ethnicities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethnicities_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_plans: Option<FamilyPlans>,
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_completed_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_id: Option<GenderEnum>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_identity_id: Option<GenderIdentityId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hometown: Option<Hometown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_title: Option<JobTitle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages_spoken: Option<LanguagesSpoken>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    pub location: Location,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marijuana: Option<MarijuanaStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pets: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub politics: Option<Politics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<Pronouns>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_type_ids: Option<RelationshipTypeIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_types_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub religions: Option<Religion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfie_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sexual_orientations: Option<SexualOrientations>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smoking: Option<SmokingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub works: Option<Works>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zodiac: Option<Zodiac>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_status_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub did_just_join: Option<bool>,
    // Content fields that might not be in the /user/v3 response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<ProfileName>,
    #[serde(default)]
    pub photos: Vec<PhotoAsset>,
    #[serde(default)]
    pub answers: Vec<ProfileAnswer>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfProfileResponse {
    pub user_id: String,
    pub created: String,
    pub registered: String,
    pub modified: String,
    pub last_active_opt_in: bool,
    pub profile: Profile,
    pub paused: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContentData {
    #[serde(default)]
    pub photos: Vec<PhotoAsset>,
    #[serde(default)]
    pub answers: Vec<ProfileAnswer>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_poll: Option<PromptPoll>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video_prompt: Option<VideoPrompt>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SelfContentResponse {
    pub content: ContentData,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PreferencesResponse {
    pub gendered_age_ranges: GenderedRange,
    pub dealbreakers: Dealbreakers,
    pub religions: Vec<ReligionPreference>,
    pub drinking: Vec<DrinkingStatusPreference>,
    pub gendered_height_ranges: GenderedRange,
    pub marijuana: Vec<MarijuanaStatusPreference>,
    pub relationship_types: Vec<RelationshipTypePreference>,
    pub drugs: Vec<DrugStatusPreference>,
    pub max_distance: i32,
    pub children: Vec<ChildrenStatusPreference>,
    pub ethnicities: Vec<EthnicityPreference>,
    pub smoking: Vec<SmokingStatusPreference>,
    pub education_attained: Vec<EducationAttainedPreference>,
    pub family_plans: Vec<i32>,
    pub dating_intentions: Vec<DatingIntentionPreference>,
    pub politics: Vec<PoliticsPreference>,
    pub gender_preferences: Vec<GenderPreferences>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeResponse {
    pub limit: LikeLimit,
}

// Rate respond (POST /rate/v2/respond)
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateRespondRequest {
    pub sort_type: String,
    pub subject_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rating_id: Option<String>,
    pub origin: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiated_with: Option<String>,
    pub rating: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateRespondResponse {
    #[serde(default)]
    pub limit: Option<LikeLimit>,
}

// Likes v2 (recent likes feed)
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeRatingContentItem {
    #[serde(default)]
    pub photo: Option<PhotoAsset>,
    #[serde(default)]
    pub prompt: Option<CreateRateContentPrompt>,
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    pub prompt_poll: Option<LikePromptPoll>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikePromptPoll {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    pub options: Vec<String>,
    pub question_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_option: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_option_index: Option<i32>,
}

// Match note response
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MatchNoteResponse {
    pub note: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeRating {
    #[serde(default)]
    pub content: Vec<LikeRatingContentItem>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeItemV2 {
    pub player_id: String,
    pub subject_id: String,
    pub created: String,
    pub source: String,
    pub initiated_with: String,
    pub rating: LikeRating,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikeSortV2 {
    pub id: String,
    pub title: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortedLikeIdV2 {
    pub id: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SortedLikesGroupV2 {
    #[serde(rename = "sortID")]
    pub sort_id: String,
    pub data: Vec<SortedLikeIdV2>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LikesV2Response {
    pub likes: Vec<LikeItemV2>,
    #[serde(default)]
    pub hidden_likes: Vec<JsonValue>,
    #[serde(default)]
    pub sorts: Vec<LikeSortV2>,
    #[serde(default)]
    pub sorted_likes: Vec<SortedLikesGroupV2>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub age: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub birthday: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub covid_vax: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<i32>, // Direct value, not wrapper
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dating_intention: Option<i32>, // Direct value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dating_intention_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drinking: Option<i32>, // Direct value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drugs: Option<i32>, // Direct value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education_attained: Option<EducationAttainedProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub educations: Option<Vec<String>>, // Direct array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethnicities: Option<Vec<i32>>, // Direct array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethnicities_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub family_plans: Option<i32>, // Direct value
    #[serde(rename = "firstName")]
    pub first_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_completed_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_id: Option<GenderEnum>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_identity_id: Option<i32>, // Direct value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hometown: Option<String>, // Direct string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub job_title: Option<String>, // Direct string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages_spoken: Option<Vec<i32>>, // Direct array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    pub location: Location, // Same structure
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marijuana: Option<i32>, // Direct value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pets: Option<Vec<i32>>, // Direct array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub politics: Option<i32>, // Direct value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pronouns: Option<Vec<i32>>, // Direct array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_type_ids: Option<Vec<i32>>, // Direct array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_types_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub religions: Option<Vec<i32>>, // Direct array
    #[serde(rename = "selfieVerified")]
    pub selfie_verified: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sexual_orientations: Option<Vec<i32>>, // Direct array
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smoking: Option<i32>, // Direct value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub works: Option<String>, // Direct string
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zodiac: Option<i32>, // Direct value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_active_status_id: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none", alias = "didjustJoin")]
    pub did_just_join: Option<bool>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublicUserProfile {
    pub user_id: String,
    pub profile: PublicProfile,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: String,
    pub profile: JsonValue,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileContentContent {
    pub photos: Vec<PhotoAsset>,
    pub answers: Vec<ProfileAnswer>,
    #[serde(default)]
    pub prompt_poll: Option<PromptPoll>,
    #[serde(default)]
    pub video_prompt: Option<VideoPrompt>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileContentFull {
    pub user_id: String,
    pub content: ProfileContentContent,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRateContentPrompt {
    pub answer: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    pub question: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRateContent {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<PhotoAsset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<CreateRateContentPrompt>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateRate {
    pub rating_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hcm_run_id: Option<String>,
    pub session_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<CreateRateContent>,
    // RFC3339 without subseconds, ending with Z
    pub created: String,
    pub rating_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub initiated_with: Option<String>,
    // "like" | "note" | "skip"
    pub rating: String,
    #[serde(default)]
    pub has_pairing: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
    pub subject_id: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceAnswerPayload {
    pub cdn_url: String,
    pub waveform: String,
    #[serde(rename = "type")]
    pub answer_type: String,
    pub cdn_id: String,
    pub url: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerContentPayload {
    pub position: i32,
    pub question_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_answer: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub voice_answer: Option<VoiceAnswerPayload>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AnswerEvaluateRequest {
    pub answer: String,
    pub prompt_id: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePromptPollRequest {
    pub options: Vec<String>,
    pub question_id: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreatePromptPollResponse {
    pub content_id: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVideoPromptRequest {
    pub question_id: String,
    pub thumbnail_url: String,
    pub cdn_id: String,
    pub bounding_box: BoundingBox,
    pub video_url: String,
    pub source: String,
    pub length_seconds: i32,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVideoPromptResponse {
    pub content_id: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    pub id: String,
    pub prompt: String,
    pub is_selectable: bool,
    #[serde(default)]
    pub placeholder: String,
    pub is_new: bool,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub content_types: Vec<ContentType>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptCategory {
    pub name: String,
    pub slug: String,
    pub is_visible: bool,
    pub is_new: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PromptsResponse {
    pub prompts: Vec<Prompt>,
    pub categories: Vec<PromptCategory>,
}

// Preferences DTOs
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RangeDetails {
    pub max: Option<i32>,
    pub min: Option<i32>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenderedRange {
    #[serde(rename = "0")]
    pub men: Option<RangeDetails>,
    #[serde(rename = "1")]
    pub women: Option<RangeDetails>,
    #[serde(rename = "3")]
    pub non_binary_people: Option<RangeDetails>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecsV2Params {
    #[serde(default)]
    pub new_here: bool,
    #[serde(default)]
    pub active_today: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivePill {
    pub pill_type: String,
    pub permission: String,
    pub id: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RecommendationsPreview {
    pub subjects: Vec<RecommendationSubject>,
    pub permission: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GenderedDealbreaker {
    #[serde(rename = "0")]
    pub men: Option<bool>,
    #[serde(rename = "1")]
    pub women: Option<bool>,
    #[serde(rename = "3")]
    pub non_binary_people: Option<bool>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Dealbreakers {
    pub marijuana: bool,
    pub smoking: bool,
    pub max_distance: bool,
    pub drinking: bool,
    pub education_attained: bool,
    pub gendered_height: GenderedDealbreaker,
    pub politics: bool,
    pub relationship_types: bool,
    pub drugs: bool,
    pub dating_intentions: bool,
    pub family_plans: bool,
    pub gendered_age: GenderedDealbreaker,
    pub religions: bool,
    pub ethnicities: bool,
    pub children: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Preferences {
    pub gendered_age_ranges: GenderedRange,
    pub dealbreakers: Dealbreakers,
    pub religions: Vec<ReligionPreference>,
    pub drinking: Vec<DrinkingStatusPreference>,
    pub gendered_height_ranges: GenderedRange,
    pub marijuana: Vec<MarijuanaStatusPreference>,
    pub relationship_types: Vec<RelationshipTypePreference>,
    pub drugs: Vec<DrugStatusPreference>,
    pub max_distance: i32,
    pub children: Vec<ChildrenStatusPreference>,
    pub ethnicities: Vec<EthnicityPreference>,
    pub smoking: Vec<SmokingStatusPreference>,
    pub education_attained: Vec<EducationAttainedPreference>,
    pub family_plans: Vec<i32>, // TODO: Define proper enum
    pub dating_intentions: Vec<DatingIntentionPreference>,
    pub politics: Vec<PoliticsPreference>,
    pub gender_preferences: Vec<GenderPreferences>,
}

// Profile update DTOs
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileUpdate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub children: Option<ChildrenStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dating_intention: Option<DatingIntention>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drinking: Option<DrinkingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drugs: Option<DrugStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub marijuana: Option<MarijuanaStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smoking: Option<SmokingStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub politics: Option<Politics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub religions: Option<Religion>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ethnicities: Option<Ethnicities>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub education_attained: Option<EducationAttainedProfile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relationship_type_ids: Option<RelationshipTypeIds>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gender_id: Option<GenderEnum>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hometown: Option<Hometown>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub languages_spoken: Option<LanguagesSpoken>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zodiac: Option<Zodiac>,
}

// Shared DTOs for RSPC inputs
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkipInput {
    pub subject_id: String,
    pub rating_token: String,
    #[serde(default)]
    pub origin: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoAssetInput {
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdn_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selfie_verified: Option<bool>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateInput {
    pub subject_id: String,
    pub rating_token: String,
    #[serde(default)]
    pub origin: Option<String>,
    #[serde(default)]
    pub use_superlike: Option<bool>,
    #[serde(default)]
    pub photo: Option<PhotoAssetInput>,
    #[serde(default)]
    pub answer_text: Option<String>,
    #[serde(default)]
    pub question_text: Option<String>,
    #[serde(default)]
    pub content_id: Option<String>,
    #[serde(default)]
    pub comment: Option<String>,
}

// Additional models from reverse engineering

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserSettings {
    pub is_smart_photo_opt_in: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthSettings {
    pub apple_authed: bool,
    pub facebook_authed: bool,
    pub google_authed: bool,
    pub sms_authed: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationSettings {
    pub email: std::collections::HashMap<String, bool>,
    pub push: std::collections::HashMap<String, bool>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserTrait {
    pub id: String,
    pub user_input: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    pub subscription: JsonValue,
    pub account: std::collections::HashMap<String, bool>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportStatus {
    pub status: String,
    pub created: Option<String>,
    pub url: Option<String>,
}

// Rate/Like models
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RatePayload {
    pub rating_id: String,
    pub rating_token: String,
    pub subject_id: String,
    pub session_id: String,
    pub rating: String,
    pub origin: String,
    pub has_pairing: bool,
    pub created: String,
    pub initiated_with: String,
    pub content: Option<RateContentPayload>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RateContentPayload {
    pub comment: Option<String>,
    pub photo: Option<JsonValue>,
    pub prompt: Option<JsonValue>,
}

// Message models
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendMessagePayload {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedup_id: Option<String>,
    pub ays: bool,
    pub match_message: bool,
    pub message_type: String,
    pub message_data: MessageData,
    pub subject_id: String,
    pub origin: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MessageData {
    pub message: String,
}

// Standouts models
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandoutMediaRef {
    pub content_id: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandoutContent {
    #[serde(default)]
    pub photo: Option<StandoutMediaRef>,
    #[serde(default)]
    pub prompt: Option<StandoutMediaRef>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandoutItem {
    pub subject_id: String,
    pub rating_token: String,
    pub content: StandoutContent,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StandoutsResponse {
    pub status: String,
    #[serde(default)]
    pub expiration: Option<String>,
    #[serde(default)]
    pub standouts: Vec<StandoutItem>,
}

// Sendbird messages
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendbirdMessageMetaItem {
    pub key: String,
    pub value: Vec<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendbirdMessageUser {
    pub user_id: String,
    #[serde(default)]
    pub profile_url: String,
    #[serde(default)]
    pub require_auth_for_profile_image: bool,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub metadata: JsonValue,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub is_blocked_by_me: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendbirdMessage {
    #[serde(rename = "type")]
    pub msg_type: String,
    #[serde(deserialize_with = "de_num_to_string")]
    pub message_id: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub data: String,
    #[serde(default)]
    pub custom_type: String,
    #[serde(default)]
    pub file: JsonValue,
    #[serde(deserialize_with = "de_num_to_string")]
    pub created_at: String,
    pub user: SendbirdMessageUser,
    pub channel_url: String,
    #[serde(default, deserialize_with = "de_num_to_string")]
    pub updated_at: String,
    #[serde(default, deserialize_with = "de_num_to_string")]
    pub message_survival_seconds: String,
    #[serde(default)]
    pub mentioned_users: Vec<JsonValue>,
    #[serde(default)]
    pub mention_type: String,
    #[serde(default)]
    pub silent: bool,
    #[serde(default, deserialize_with = "de_num_to_string")]
    pub message_retention_hour: String,
    #[serde(default)]
    pub channel_type: String,
    #[serde(default)]
    pub translations: JsonValue,
    #[serde(default)]
    pub sorted_metaarray: Vec<SendbirdMessageMetaItem>,
    #[serde(default)]
    pub is_removed: bool,
    #[serde(default)]
    pub is_op_msg: bool,
    #[serde(default)]
    pub reactions_summary: Vec<JsonValue>,
    #[serde(default)]
    pub message_events: JsonValue,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendbirdMessagesResponse {
    pub messages: Vec<SendbirdMessage>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendbirdChannelMember {
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub profile_url: String,
    #[serde(default)]
    pub metadata: JsonValue,
    #[serde(default)]
    pub is_active: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendbirdGroupChannel {
    pub channel_url: String,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub cover_url: String,
    #[serde(default)]
    pub member_count: i32,
    #[serde(default)]
    pub joined_member_count: i32,
    #[serde(default)]
    pub unread_message_count: i32,
    #[serde(default)]
    pub members: Vec<SendbirdChannelMember>,
    #[serde(default)]
    pub metadata: JsonValue,
    #[serde(default)]
    pub custom_type: String,
    #[serde(default)]
    pub my_member_state: String,
    #[serde(default, deserialize_with = "de_num_to_string")]
    pub created_at: String,
    #[serde(default, deserialize_with = "de_num_to_string")]
    pub updated_at: String,
    #[serde(default)]
    pub last_message: Option<SendbirdMessage>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct SendbirdChannelsResponse {
    #[serde(default)]
    pub channels: Vec<SendbirdGroupChannel>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendbirdChannelsInput {
    #[serde(default)]
    pub limit: Option<i32>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportChatInput {
    pub channel_url: String,
    pub output_dir: String,
    #[serde(default)]
    pub include_media: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub initiation_summary_lines: Option<Vec<String>>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportedMediaFile {
    pub message_id: String,
    pub file_name: String,
    pub file_path: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportChatResult {
    pub folder_path: String,
    pub transcript_path: String,
    #[serde(default)]
    pub profile_path: Option<String>,
    pub message_count: i32,
    pub media_files: Vec<ExportedMediaFile>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionPrompt {
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub answer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionVideo {
    #[serde(default)]
    pub url: String,
    #[serde(default)]
    pub thumbnail: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bounding_box: Option<BoundingBox>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cdn_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content_id: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionContentItem {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<ConnectionPrompt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photo: Option<PhotoAsset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub video: Option<ConnectionVideo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionItem {
    #[serde(default)]
    pub initiator_id: String,
    #[serde(default)]
    pub subject_id: String,
    #[serde(default)]
    pub sent_content: Vec<ConnectionContentItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sent_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub received_time: Option<String>,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub initiated_with: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub social_media_exchanged_timestamp: Option<String>,
    #[serde(default)]
    pub is_hidden: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetailApi {
    #[serde(flatten)]
    pub connection: ConnectionItem,
    #[serde(
        default,
        rename = "showMatchNote",
        skip_serializing_if = "Option::is_none"
    )]
    pub show_match_note: Option<bool>,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionsResponse {
    #[serde(default)]
    pub connections: Vec<ConnectionItem>,
    #[serde(default)]
    pub your_turn_match_limit: i32,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendbirdChannelHandle {
    pub channel_url: String,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendbirdGetMessagesInput {
    pub channel_url: String,
    pub message_ts: String,
    pub prev_limit: i32,
}

// Sendbird WebSocket READ response models
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendbirdReadUser {
    pub name: String,
    pub image: String,
    pub require_auth_for_profile_image: bool,
    pub guest_id: String,
    #[serde(deserialize_with = "de_num_to_string")]
    pub id: String,
    pub role: String,
    #[serde(default)]
    pub metadata: JsonValue,
    pub is_bot: bool,
    pub is_ai_bot: bool,
    pub is_active: bool,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendbirdReadResponse {
    #[serde(deserialize_with = "de_num_to_string")]
    pub channel_id: String,
    pub user: SendbirdReadUser,
    #[serde(deserialize_with = "de_num_to_string")]
    pub ts: String,
    #[serde(deserialize_with = "de_num_to_string")]
    pub sts: String,
    pub channel_url: String,
    pub channel_type: String,
    pub is_super: bool,
    pub target_parent_message_id: Option<String>,
    pub req_id: String,
}

// Sendbird WebSocket SYEV (system event) models
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SendbirdSyevUserData {
    pub user_id: String,
    #[serde(default)]
    pub nickname: String,
    #[serde(default)]
    pub profile_url: String,
    #[serde(default)]
    pub require_auth_for_profile_image: bool,
    #[serde(default)]
    pub metadata: JsonValue,
}

#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SendbirdSyevEvent {
    #[serde(default)]
    pub channel_id: Option<String>,
    pub cat: i32,
    #[serde(default)]
    pub data: Option<SendbirdSyevUserData>,
    #[serde(default, deserialize_with = "de_num_to_string")]
    pub sts: String,
    #[serde(default, deserialize_with = "de_num_to_string")]
    pub ts: String,
    pub channel_url: String,
    pub channel_type: String,
    #[serde(default)]
    pub is_super: bool,
    #[serde(default)]
    pub is_access_code_required: bool,
    #[serde(default)]
    pub has_bot: bool,
    #[serde(default)]
    pub has_ai_bot: bool,
}

impl SendbirdSyevEvent {
    pub const CATEGORY_TYPING_START: i32 = 10900;
    pub const CATEGORY_TYPING_END: i32 = 10901;
}

// Sendbird Close request (for JS-side WS close)
#[cfg_attr(feature = "json-schema", derive(schemars::JsonSchema))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SendbirdCloseRequest {
    #[serde(default)]
    pub code: Option<u16>,
    #[serde(default)]
    pub reason: Option<String>,
}

// Allow numbers or strings to deserialize into String
fn de_num_to_string<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let v = serde_json::Value::deserialize(deserializer)?;
    match v {
        serde_json::Value::String(s) => Ok(s),
        serde_json::Value::Number(n) => Ok(n.to_string()),
        serde_json::Value::Null => Ok(String::new()),
        other => Err(serde::de::Error::custom(format!(
            "expected string or number, got {}",
            other
        ))),
    }
}
