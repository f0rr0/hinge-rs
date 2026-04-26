use crate::enums::EducationAttainedProfile;
use crate::models::{ConnectionContentItem, ConnectionItem, ProfileContentFull, PublicUserProfile};
use crate::prompts_manager::HingePromptsManager;
use std::collections::HashSet;
use std::fmt::Write as _;

const CHILDREN_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't have children"),
    (2, "Have children"),
];

const DATING_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Unknown"),
    (1, "Life partner"),
    (2, "Long-term relationship"),
    (3, "Long-term, open to short"),
    (4, "Short-term, open to long"),
    (5, "Short-term relationship"),
    (6, "Figuring out their dating goals"),
];

const DRINKING_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't drink"),
    (2, "Drink"),
    (3, "Sometimes"),
];

const SMOKING_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't smoke"),
    (2, "Smoke"),
    (3, "Sometimes"),
];

const MARIJUANA_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't use marijuana"),
    (2, "Use marijuana"),
    (3, "Sometimes"),
    (4, "No preference"),
];

const DRUG_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (0, "Prefer not to say"),
    (1, "Don't use drugs"),
    (2, "Use drugs"),
    (3, "Sometimes"),
];

const RELATIONSHIP_TYPE_LABELS: &[(i32, &str)] = &[
    (-1, "Open to all"),
    (1, "Monogamy"),
    (2, "Ethical non-monogamy"),
    (3, "Open relationship"),
    (4, "Polyamory"),
    (5, "Open to exploring"),
];

fn label_from_map(map: &'static [(i32, &'static str)], code: Option<i32>) -> Option<&'static str> {
    let key = code?;
    map.iter().find(|(c, _)| *c == key).map(|(_, label)| *label)
}

fn labels_from_map(
    map: &'static [(i32, &'static str)],
    codes: &Option<Vec<i32>>,
) -> Vec<&'static str> {
    match codes {
        Some(values) => values
            .iter()
            .filter_map(|code| map.iter().find(|(c, _)| c == code).map(|(_, label)| *label))
            .collect(),
        None => Vec::new(),
    }
}
fn education_attained_label(value: &EducationAttainedProfile) -> &'static str {
    use EducationAttainedProfile::*;
    match value {
        PreferNotToSay => "Prefer not to say",
        HighSchool => "High school",
        TradeSchool => "Trade school",
        InCollege => "In college",
        Undergraduate => "Undergraduate degree",
        InGradSchool => "In grad school",
        Graduate => "Graduate degree",
    }
}
pub(super) fn summarize_connection_initiation(
    connection: &ConnectionItem,
    self_user_id: &str,
    peer_user_id: &str,
    peer_display_name: &str,
) -> Option<Vec<String>> {
    let initiator_id = connection.initiator_id.trim();
    let initiator_label = if initiator_id.is_empty() {
        "Unknown".to_string()
    } else if initiator_id == self_user_id {
        "You".to_string()
    } else if initiator_id == peer_user_id {
        peer_display_name.to_string()
    } else {
        initiator_id.to_string()
    };

    let mut lines = Vec::new();
    if let Some(with_label) = prettify_initiated_with(&connection.initiated_with) {
        lines.push(format!(
            "Conversation initiated by {} via {}.",
            initiator_label, with_label
        ));
    } else {
        lines.push(format!("Conversation initiated by {}.", initiator_label));
    }

    let mut seen: HashSet<String> = HashSet::new();
    let mut detail_lines = Vec::new();
    for content in &connection.sent_content {
        for description in describe_connection_content_item(content) {
            if seen.insert(description.clone()) {
                detail_lines.push(description);
            }
        }
    }

    for detail in detail_lines {
        lines.push(format!("  • {}", detail));
    }

    Some(lines)
}

fn describe_connection_content_item(item: &ConnectionContentItem) -> Vec<String> {
    let mut lines = Vec::new();
    if let Some(prompt) = &item.prompt {
        let question = prompt.question.trim();
        let answer = prompt.answer.trim();
        if !question.is_empty() && !answer.is_empty() {
            lines.push(format!("Prompt \"{}\" – \"{}\"", question, answer));
        } else if !question.is_empty() {
            lines.push(format!("Prompt \"{}\"", question));
        } else if !answer.is_empty() {
            lines.push(format!("Prompt answer \"{}\"", answer));
        }
    }

    if let Some(comment) = &item.comment {
        let trimmed = comment.trim();
        if !trimmed.is_empty() {
            lines.push(format!("Comment: {}", trimmed));
        }
    }

    if let Some(photo) = &item.photo {
        let caption = photo.caption.as_deref().map(str::trim).unwrap_or("");
        if !caption.is_empty() {
            lines.push(format!("Photo liked – {}", caption));
        } else {
            lines.push("Photo liked".to_string());
        }
    }

    if let Some(video) = &item.video {
        if !video.url.trim().is_empty() {
            lines.push("Video shared".to_string());
        } else {
            lines.push("Video interaction".to_string());
        }
    }

    lines
}

fn prettify_initiated_with(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }

    let words: Vec<String> = trimmed
        .split(['_', ' '])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            if let Some(first) = chars.next() {
                let mut result = first.to_uppercase().collect::<String>();
                result.push_str(&chars.as_str().to_lowercase());
                result
            } else {
                String::new()
            }
        })
        .filter(|s| !s.is_empty())
        .collect();

    if words.is_empty() {
        None
    } else {
        Some(words.join(" "))
    }
}

pub(super) fn render_profile(
    profile: Option<&PublicUserProfile>,
    content: Option<&ProfileContentFull>,
    prompts: Option<&HingePromptsManager>,
) -> String {
    let mut out = String::new();

    if let Some(wrapper) = profile {
        let p = &wrapper.profile;
        let _ = writeln!(out, "Name: {}", p.first_name);
        if let Some(age) = p.age {
            let _ = writeln!(out, "Age: {}", age);
        }
        if let Some(height) = p.height {
            let _ = writeln!(out, "Height: {} cm", height);
        }
        if let Some(children) = label_from_map(CHILDREN_LABELS, p.children) {
            let _ = writeln!(out, "Children: {}", children);
        }
        if let Some(label) = label_from_map(DATING_LABELS, p.dating_intention) {
            let _ = writeln!(out, "Dating intention: {}", label);
        }
        if let Some(label) = label_from_map(DRINKING_LABELS, p.drinking) {
            let _ = writeln!(out, "Drinking: {}", label);
        }
        if let Some(label) = label_from_map(SMOKING_LABELS, p.smoking) {
            let _ = writeln!(out, "Smoking: {}", label);
        }
        if let Some(label) = label_from_map(MARIJUANA_LABELS, p.marijuana) {
            let _ = writeln!(out, "Marijuana: {}", label);
        }
        if let Some(label) = label_from_map(DRUG_LABELS, p.drugs) {
            let _ = writeln!(out, "Drugs: {}", label);
        }
        let relationship_labels =
            labels_from_map(RELATIONSHIP_TYPE_LABELS, &p.relationship_type_ids);
        if !relationship_labels.is_empty() {
            let _ = writeln!(
                out,
                "Relationship types: {}",
                relationship_labels.join(", ")
            );
        }
        if let Some(job) = p.job_title.as_ref().filter(|v| !v.trim().is_empty()) {
            let _ = writeln!(out, "Job title: {}", job);
        }
        if let Some(work) = p.works.as_ref().filter(|v| !v.trim().is_empty()) {
            let _ = writeln!(out, "Workplace: {}", work);
        }
        if let Some(level) = p.education_attained.as_ref() {
            let _ = writeln!(out, "Education level: {}", education_attained_label(level));
        }
        if let Some(schools) = p.educations.as_ref() {
            let entries: Vec<&str> = schools
                .iter()
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !entries.is_empty() {
                let _ = writeln!(out, "Education: {}", entries.join(", "));
            }
        }
        if !p.location.name.trim().is_empty() {
            let _ = writeln!(out, "Location: {}", p.location.name);
        }
        out.push('\n');
    } else {
        out.push_str("Profile information unavailable.\n\n");
    }

    if let Some(full) = content
        && !full.content.answers.is_empty()
    {
        out.push_str("Prompts:\n");
        for answer in &full.content.answers {
            let response = answer
                .response
                .as_ref()
                .map(|text| text.trim())
                .filter(|text| !text.is_empty());
            if let Some(resp) = response {
                let mut question: Option<String> = None;

                if let Some(mgr) = prompts
                    && let Some(prompt_id) = answer.prompt_id.as_ref()
                {
                    let text = mgr.get_prompt_display_text(prompt_id);
                    if !text.trim().is_empty() && text != "Unknown Question" {
                        question = Some(text);
                    }
                }

                if question.is_none()
                    && let Some(mgr) = prompts
                    && let Some(question_id) = answer.question_id.as_ref()
                {
                    let text = mgr.get_prompt_display_text(question_id);
                    if !text.trim().is_empty() && text != "Unknown Question" {
                        question = Some(text);
                    }
                }

                if question.is_none() {
                    question = answer
                        .content
                        .as_ref()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }

                if question.is_none() {
                    question = answer
                        .question_id
                        .as_ref()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }

                if question.is_none() {
                    question = answer
                        .prompt_id
                        .as_ref()
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty());
                }

                let question = question.unwrap_or_else(|| "Prompt".to_string());
                let _ = writeln!(out, "- {}: {}", question, resp);
            }
        }
        out.push('\n');
    }

    out
}
