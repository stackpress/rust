//! Explicit small translation fixture; no browser detection or dictionary fetch.

use stackpress_framework_ui::language::Language;

/// Construct the checked English/French fixture used identically by every target.
///
/// Other valid locale keys have an empty map and retain source phrase fallback.
/// # Errors
/// Rejects invalid locale keys through the framework's checked public language API.
pub fn profile_language(locale: &str) -> Result<Language, &'static str> {
    let phrases = if locale == "fr" {
        [
            ("Profiles", "Profils"),
            (
                "Search public demo profiles",
                "Rechercher des profils publics",
            ),
            (
                "Find a person by profile ID.",
                "Trouvez une personne par son identifiant.",
            ),
            ("Profile ID prefix", "Préfixe de l’identifiant"),
            ("For example, abc", "Par exemple, abc"),
            ("Search", "Rechercher"),
            ("Searching…", "Recherche en cours…"),
            ("%s profiles", "%s profils"),
            (
                "Showing %s public profiles.",
                "%s profils publics affichés.",
            ),
            (
                "No profiles match this ID prefix. Try another prefix.",
                "Aucun profil ne correspond. Essayez un autre préfixe.",
            ),
            (
                "Use the first characters of a profile ID. IDs are case sensitive.",
                "Utilisez les premiers caractères d’un identifiant. La casse est respectée.",
            ),
            ("Retry search", "Réessayer la recherche"),
            ("Cancel search", "Annuler la recherche"),
            ("Dark theme", "Thème sombre"),
            ("Light theme", "Thème clair"),
        ]
        .into_iter()
        .map(|(key, value)| (key.to_owned(), value.to_owned()))
        .collect()
    } else {
        // Selecting a language without a map deliberately clears previous translations.
        Default::default()
    };
    Language::new(locale.into(), phrases, "%s".into())
}
