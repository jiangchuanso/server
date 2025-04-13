use crate::{AppError, AppState, translation::perform_translation};
use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct TranslationRequest {
    text: String,
    from: Option<String>,
    to: String,
}

#[derive(Debug, Serialize)]
pub struct TranslationResponse {
    text: String,
    from: String,
    to: String,
}

pub async fn translate(
    State(state): State<Arc<AppState>>,
    Json(request): Json<TranslationRequest>,
) -> Result<Json<TranslationResponse>, AppError> {
    let (text, from_lang, to_lang) =
        perform_translation(&state.translator, &request.text, request.from, &request.to).await?;

    Ok(Json(TranslationResponse {
        text,
        from: from_lang,
        to: to_lang,
    }))
}

#[derive(Debug, Deserialize)]
pub struct KissTranslationRequest {
    text: String,
    from: Option<String>,
    to: String,
}

#[derive(Debug, Serialize)]
pub struct KissTranslationResponse {
    text: String,
    from: String,
    to: String,
}

pub async fn translate_kiss(
    State(state): State<Arc<AppState>>,
    Json(request): Json<KissTranslationRequest>,
) -> Result<Json<KissTranslationResponse>, AppError> {
    let (text, from_lang, to_lang) =
        perform_translation(&state.translator, &request.text, request.from, &request.to).await?;

    Ok(Json(KissTranslationResponse {
        text,
        from: from_lang,
        to: to_lang,
    }))
}

#[derive(Debug, Deserialize)]
pub struct ImmersiveTranslationRequest {
    source_lang: Option<String>,
    target_lang: String,
    text_list: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ImmersiveTranslationItem {
    detected_source_lang: String,
    text: String,
}

#[derive(Debug, Serialize)]
pub struct ImmersiveTranslationResponse {
    translations: Vec<ImmersiveTranslationItem>,
}

pub async fn translate_immersive(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ImmersiveTranslationRequest>,
) -> Result<Json<ImmersiveTranslationResponse>, AppError> {
    let mut translations = Vec::with_capacity(request.text_list.len());

    for text in request.text_list {
        let (translated_text, from_lang, _) = perform_translation(
            &state.translator,
            &text,
            request.source_lang.clone(),
            &request.target_lang,
        )
        .await?;

        translations.push(ImmersiveTranslationItem {
            detected_source_lang: from_lang,
            text: translated_text,
        });
    }

    Ok(Json(ImmersiveTranslationResponse { translations }))
}

#[derive(Debug, Deserialize)]
pub struct HcfyTranslationRequest {
    text: String,
    source: Option<String>,
    destination: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct HcfyTranslationResponse {
    text: String,
    from: String,
    to: String,
    result: Vec<String>,
}

pub async fn translate_hcfy(
    State(state): State<Arc<AppState>>,
    Json(request): Json<HcfyTranslationRequest>,
) -> Result<Json<HcfyTranslationResponse>, AppError> {
    const LANGUAGE_CODE_MAP: &[(&str, &str)] =
        &[("中文(简体)", "zh"), ("英语", "en"), ("日语", "jp")];

    const LANGUAGE_NAME_MAP: &[(&str, &str)] =
        &[("zh", "中文(简体)"), ("en", "英语"), ("jp", "日语")];

    fn convert_language_name(lang: &str) -> String {
        LANGUAGE_CODE_MAP
            .iter()
            .find(|&&(name, _)| name == lang)
            .map(|&(_, code)| code)
            .unwrap_or_else(|| {
                if ["zh", "en", "jp"].contains(&lang) {
                    lang
                } else {
                    lang
                }
            })
            .to_string()
    }

    fn get_language_name(code: &str) -> String {
        LANGUAGE_NAME_MAP
            .iter()
            .find(|&&(c, _)| c == code)
            .map(|&(_, name)| name)
            .unwrap_or(code)
            .to_string()
    }

    let source_lang = request.source.as_deref().map(convert_language_name);

    let target_lang = match (
        request.destination.get(0),
        source_lang.as_deref(),
        request.destination.get(1),
    ) {
        (None, _, _) => "en".to_string(),
        (Some(first), Some(src), Some(second)) if convert_language_name(first) == src => {
            convert_language_name(second)
        }
        (Some(first), _, _) => convert_language_name(first),
    };

    let (translated_text, detected_source, _) =
        perform_translation(&state.translator, &request.text, source_lang, &target_lang).await?;

    Ok(Json(HcfyTranslationResponse {
        text: request.text,
        from: get_language_name(&detected_source),
        to: get_language_name(&target_lang),
        result: vec![translated_text],
    }))
}
