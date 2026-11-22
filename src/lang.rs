//! The script's texts: `lang/{en,fr,es,tr}.lang`, embedded and checked for the same keys while it compiles.
use sp_lang::{Catalog, Language};

static LANG: Catalog = sp_lang::catalog!("list-lite");

/// The text of `key` in the language of `locale` (English for any other), with `{0}`, `{1}` filled from `args`.
pub fn text(locale: &str, key: &str, args: &[&str]) -> String {
    let raw = LANG
        .get_or_en(Language::of_locale(locale), key)
        .unwrap_or_default();
    sp_lang::render(raw, args, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_key_the_script_names_has_a_text_in_every_language() {
        let own = [
            "description",
            "permission_use",
            "command_list",
            "no_permission",
            "online",
            "players",
            "group",
            "more",
        ];
        for key in own {
            for language in Language::ALL {
                assert!(
                    LANG.get(language, key).is_some_and(|t| !t.is_empty()),
                    "{key} in {}",
                    language.code()
                );
            }
        }
        assert_eq!(LANG.len(), own.len(), "a key nobody names");
    }

    #[test]
    fn the_first_number_of_the_first_line_is_the_player_count_in_every_language() {
        for locale in ["en_us", "fr_fr", "es_es", "tr_tr"] {
            let line = text(locale, "online", &["12", "100"]);
            let first: String = line
                .chars()
                .skip_while(|c| !c.is_ascii_digit())
                .take_while(char::is_ascii_digit)
                .collect();
            assert_eq!(first, "12", "{line}");
        }
    }
}
