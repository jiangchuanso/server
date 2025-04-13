use crate::AppError;
use bergamot::Translator;
use isolang::Language;

fn language_detect(text: &str) -> Result<Language, AppError> {
    whatlang::detect_lang(text)
        .ok_or_else(|| {
            AppError::TranslationError(
                "Language detection failed: text may be too short or ambiguous".to_string(),
            )
        })
        .and_then(|info| {
            Language::from_639_3(info.code()).ok_or_else(|| {
                AppError::TranslationError(format!(
                    "Failed to convert ISO-639-3 code '{}' to language",
                    info.code()
                ))
            })
        })
}

fn parse_language_code(code: &str) -> Result<Language, AppError> {
    Language::from_639_1(code).ok_or_else(|| {
        AppError::TranslationError(format!(
            "Invalid language code: '{}'. Please use ISO 639-1 format.",
            code
        ))
    })
}

fn get_iso_code(lang: &Language) -> Result<&'static str, AppError> {
    lang.to_639_1().ok_or_else(|| {
        AppError::TranslationError(format!(
            "Language '{}' doesn't have an ISO 639-1 code",
            lang
        ))
    })
}

pub async fn perform_translation(
    translator: &Translator,
    text: &str,
    from_lang: Option<String>,
    to_lang: &str,
) -> Result<(String, String, String), AppError> {
    let source_lang = match from_lang.as_deref() {
        None | Some("") | Some("auto") => language_detect(text)?,
        Some(code) => parse_language_code(code)?,
    };

    let target_lang = parse_language_code(to_lang)?;

    let from_code = get_iso_code(&source_lang)?;
    let to_code = get_iso_code(&target_lang)?;

    if !translator.is_supported(from_code, to_code)? {
        return Err(AppError::TranslationError(format!(
            "Translation from '{}' to '{}' is not supported",
            from_code, to_code
        )));
    }

    let translated_text = translator.translate(from_code, to_code, text)?;

    Ok((translated_text, from_code.to_string(), to_code.to_string()))
}
