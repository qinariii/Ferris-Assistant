use std::collections::HashMap;
use std::fs;

use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use once_cell::sync::Lazy;
use unic_langid::LanguageIdentifier;

/// Supported languages
const SUPPORTED_LANGS: &[&str] = &["en", "id"];

/// Global locale bundles: lang_code -> FluentBundle
static BUNDLES: Lazy<HashMap<String, FluentBundle<FluentResource>>> = Lazy::new(|| {
    let mut map = HashMap::new();
    for lang in SUPPORTED_LANGS {
        match load_bundle(lang) {
            Ok(bundle) => {
                log::info!("Loaded locale: {}", lang);
                map.insert(lang.to_string(), bundle);
            }
            Err(e) => {
                log::error!("Failed to load locale '{}': {}", lang, e);
            }
        }
    }
    map
});

/// Load all .ftl files for a given language into a FluentBundle
fn load_bundle(lang: &str) -> Result<FluentBundle<FluentResource>, String> {
    let langid: LanguageIdentifier = lang
        .parse()
        .map_err(|e| format!("Invalid lang id '{}': {}", lang, e))?;

    let mut bundle = FluentBundle::new_concurrent(vec![langid]);

    let dir = format!("locales/{}", lang);
    let entries = fs::read_dir(&dir).map_err(|e| format!("Cannot read {}: {}", dir, e))?;

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map(|e| e == "ftl").unwrap_or(false) {
            let source = fs::read_to_string(&path)
                .map_err(|e| format!("Cannot read {:?}: {}", path, e))?;
            let resource = FluentResource::try_new(source)
                .map_err(|(_res, errs)| format!("Parse errors in {:?}: {:?}", path, errs))?;
            bundle
                .add_resource(resource)
                .map_err(|errs| format!("Bundle errors for {:?}: {:?}", path, errs))?;
        }
    }

    Ok(bundle)
}

/// Get a translated message by key.
///
/// - `lang`: language code (e.g. "en", "id"). Falls back to "en" if not found.
/// - `key`: message identifier (e.g. "bans-ban-success")
/// - `args`: optional key-value pairs for placeholders
///
/// Returns the formatted string, or the key itself if not found.
pub fn t(lang: &str, key: &str, args: Option<&[(&str, &str)]>) -> String {
    let bundle = BUNDLES
        .get(lang)
        .or_else(|| BUNDLES.get("en"));

    let bundle = match bundle {
        Some(b) => b,
        None => return key.to_string(),
    };

    let msg = match bundle.get_message(key) {
        Some(m) => m,
        None => {
            // Fallback to English if not found in target language
            if lang != "en" {
                if let Some(en_bundle) = BUNDLES.get("en") {
                    if let Some(en_msg) = en_bundle.get_message(key) {
                        if let Some(pattern) = en_msg.value() {
                            let fluent_args = build_args(args);
                            let mut errors = vec![];
                            return en_bundle
                                .format_pattern(pattern, fluent_args.as_ref(), &mut errors)
                                .to_string();
                        }
                    }
                }
            }
            return key.to_string();
        }
    };

    let pattern = match msg.value() {
        Some(p) => p,
        None => return key.to_string(),
    };

    let fluent_args = build_args(args);
    let mut errors = vec![];
    bundle
        .format_pattern(pattern, fluent_args.as_ref(), &mut errors)
        .to_string()
}

/// Build FluentArgs from a slice of (&str, &str) pairs
fn build_args<'a>(args: Option<&[(&'a str, &'a str)]>) -> Option<FluentArgs<'a>> {
    args.map(|pairs| {
        let mut fa = FluentArgs::new();
        for (k, v) in pairs {
            fa.set(*k, FluentValue::from(*v));
        }
        fa
    })
}

/// Get the language code for a chat. Defaults to "en".
pub async fn get_chat_lang(pool: &crate::db::Pool, chat_id: i64) -> String {
    crate::db::queries::get_chat(pool, chat_id)
        .await
        .ok()
        .flatten()
        .map(|c| c.language)
        .unwrap_or_else(|| "en".to_string())
}

/// Convenience: get translated text using chat language from DB.
/// If chat has no language set, defaults to "en".
#[allow(dead_code)]
pub async fn t_chat(
    pool: &crate::db::Pool,
    chat_id: i64,
    key: &str,
    args: Option<&[(&str, &str)]>,
) -> String {
    let lang = crate::db::queries::get_chat(pool, chat_id)
        .await
        .ok()
        .flatten()
        .map(|c| c.language)
        .unwrap_or_else(|| "en".to_string());
    t(&lang, key, args)
}
