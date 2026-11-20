//! Règles métier pures (sans base de données) : rôles, statuts, droits, validation.

use chrono::NaiveDate;

pub const MAX_TEACHER_LEN: usize = 255;
pub const MAX_COMMENT_LEN: usize = 5000;
pub const MAX_REPLY_LEN: usize = 5000;
/// Un ticket ne peut pas porter sur une date plus ancienne que ça.
pub const MAX_AGE_DAYS: i64 = 365;

// ---------------------------------------------------------------- emails

pub fn normalize_email(raw: &str) -> String {
    raw.trim().to_lowercase()
}

/// Vrai si l'adresse (déjà normalisée) appartient exactement au domaine donné.
pub fn email_in_domain(email: &str, domain: &str) -> bool {
    match email.rsplit_once('@') {
        Some((local, d)) => !local.is_empty() && !local.contains('@') && d == domain,
        None => false,
    }
}

// ---------------------------------------------------------------- rôles

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Student,
    Admin,
    Superadmin,
}

impl Role {
    pub fn is_admin(self) -> bool {
        matches!(self, Role::Admin | Role::Superadmin)
    }
    pub fn is_superadmin(self) -> bool {
        self == Role::Superadmin
    }
}

// ---------------------------------------------------------------- statuts

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Ouvert,
    EnCours,
    Ferme,
    Annule,
    Supprime,
}

impl Status {
    pub const ADMIN_SETTABLE: [Status; 3] = [Status::Ouvert, Status::EnCours, Status::Ferme];
    pub const VISIBLE: [Status; 4] = [Status::Ouvert, Status::EnCours, Status::Ferme, Status::Annule];

    pub fn as_str(self) -> &'static str {
        match self {
            Status::Ouvert => "ouvert",
            Status::EnCours => "en_cours",
            Status::Ferme => "ferme",
            Status::Annule => "annule",
            Status::Supprime => "supprime",
        }
    }

    pub fn parse(s: &str) -> Option<Status> {
        Some(match s {
            "ouvert" => Status::Ouvert,
            "en_cours" => Status::EnCours,
            "ferme" => Status::Ferme,
            "annule" => Status::Annule,
            "supprime" => Status::Supprime,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Status::Ouvert => "Ouvert",
            Status::EnCours => "En cours",
            Status::Ferme => "Fermé",
            Status::Annule => "Annulé par l'étudiant",
            Status::Supprime => "Supprimé",
        }
    }

    /// `annule` et `supprime` sont définitifs.
    pub fn is_terminal(self) -> bool {
        matches!(self, Status::Annule | Status::Supprime)
    }
}

// ---------------------------------------------------------------- droits

/// Ce qu'on sait d'un ticket pour décider des droits.
#[derive(Debug, Clone, Copy)]
pub struct TicketState<'a> {
    pub author_email: &'a str,
    pub status: Status,
    pub has_reply: bool,
}

pub fn can_view(role: Role, user_email: &str, t: &TicketState) -> bool {
    t.status != Status::Supprime && (role.is_admin() || t.author_email == user_email)
}

/// « Non traité » = ouvert et sans réponse.
pub fn is_untreated(t: &TicketState) -> bool {
    t.status == Status::Ouvert && !t.has_reply
}

/// Seul l'auteur peut annuler, et seulement un ticket non traité.
pub fn can_cancel(user_email: &str, t: &TicketState) -> bool {
    t.author_email == user_email && is_untreated(t)
}

pub fn can_set_status(role: Role, t: &TicketState, to: Status) -> bool {
    role.is_admin() && !t.status.is_terminal() && Status::ADMIN_SETTABLE.contains(&to)
}

pub fn can_reply(role: Role, t: &TicketState) -> bool {
    role.is_admin() && !t.status.is_terminal()
}

pub fn can_delete(role: Role, t: &TicketState) -> bool {
    role.is_superadmin() && t.status != Status::Supprime
}

// ---------------------------------------------------------------- validation

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewTicket {
    pub date: NaiveDate,
    pub category: String,
    pub teacher: String,
    pub rating: u8,
    pub comment: String,
}

/// Valide les champs bruts du formulaire. `today` = date du jour (heure de Paris).
pub fn validate_new_ticket(
    date: &str,
    category: &str,
    teacher: &str,
    rating: &str,
    comment: &str,
    categories: &[String],
    today: NaiveDate,
) -> Result<NewTicket, Vec<String>> {
    let mut errors = Vec::new();

    let date = match NaiveDate::parse_from_str(date.trim(), "%Y-%m-%d") {
        Ok(d) if d > today => {
            errors.push("La date ne peut pas être dans le futur.".to_string());
            None
        }
        Ok(d) if (today - d).num_days() > MAX_AGE_DAYS => {
            errors.push("La date ne peut pas dater de plus d'un an.".to_string());
            None
        }
        Ok(d) => Some(d),
        Err(_) => {
            errors.push("La date est obligatoire.".to_string());
            None
        }
    };

    let category = category.trim();
    if !categories.iter().any(|c| c == category) {
        errors.push("Choisissez une catégorie.".to_string());
    }

    let teacher = teacher.trim();
    if teacher.chars().count() > MAX_TEACHER_LEN {
        errors.push(format!("Le nom de l'intervenant est limité à {MAX_TEACHER_LEN} caractères."));
    }

    let rating = match rating.trim().parse::<u8>() {
        Ok(r) if r <= 5 => Some(r),
        _ => {
            errors.push("Choisissez une note entre 0 et 5.".to_string());
            None
        }
    };

    let comment = comment.trim();
    if comment.chars().count() > MAX_COMMENT_LEN {
        errors.push(format!("Le commentaire est limité à {MAX_COMMENT_LEN} caractères."));
    }

    match (date, rating) {
        (Some(date), Some(rating)) if errors.is_empty() => Ok(NewTicket {
            date,
            category: category.to_string(),
            teacher: teacher.to_string(),
            rating,
            comment: comment.to_string(),
        }),
        _ => Err(errors),
    }
}

pub fn validate_reply(raw: &str) -> Result<String, String> {
    let reply = raw.trim();
    if reply.is_empty() {
        return Err("La réponse ne peut pas être vide.".into());
    }
    if reply.chars().count() > MAX_REPLY_LEN {
        return Err(format!("La réponse est limitée à {MAX_REPLY_LEN} caractères."));
    }
    Ok(reply.to_string())
}

// ---------------------------------------------------------------- tests

#[cfg(test)]
mod tests {
    use super::*;

    const ME: &str = "etu@normandiewebschool.fr";
    const OTHER: &str = "autre@normandiewebschool.fr";

    fn t(author: &str, status: Status, has_reply: bool) -> TicketState<'_> {
        TicketState { author_email: author, status, has_reply }
    }

    #[test]
    fn domain_check() {
        let d = "normandiewebschool.fr";
        assert!(email_in_domain("a.b@normandiewebschool.fr", d));
        assert!(!email_in_domain("a@gmail.com", d));
        assert!(!email_in_domain("a@evil-normandiewebschool.fr", d));
        assert!(!email_in_domain("a@normandiewebschool.fr.evil.com", d));
        assert!(!email_in_domain("@normandiewebschool.fr", d));
        assert!(!email_in_domain("normandiewebschool.fr", d));
        assert_eq!(normalize_email("  A.B@NormandieWebSchool.FR "), "a.b@normandiewebschool.fr");
    }

    #[test]
    fn status_roundtrip() {
        for s in [Status::Ouvert, Status::EnCours, Status::Ferme, Status::Annule, Status::Supprime] {
            assert_eq!(Status::parse(s.as_str()), Some(s));
        }
        assert_eq!(Status::parse("n'importe"), None);
    }

    #[test]
    fn visibility() {
        assert!(can_view(Role::Student, ME, &t(ME, Status::Ouvert, false)));
        assert!(can_view(Role::Student, ME, &t(ME, Status::Annule, false)));
        assert!(!can_view(Role::Student, ME, &t(OTHER, Status::Ouvert, false)));
        assert!(can_view(Role::Admin, ME, &t(OTHER, Status::Annule, false)));
        // Un ticket supprimé n'est visible par personne, même le superadmin.
        assert!(!can_view(Role::Student, ME, &t(ME, Status::Supprime, false)));
        assert!(!can_view(Role::Superadmin, ME, &t(OTHER, Status::Supprime, false)));
    }

    #[test]
    fn cancel_only_untreated_own_ticket() {
        assert!(can_cancel(ME, &t(ME, Status::Ouvert, false)));
        assert!(!can_cancel(ME, &t(ME, Status::Ouvert, true)), "déjà une réponse");
        assert!(!can_cancel(ME, &t(ME, Status::EnCours, false)));
        assert!(!can_cancel(ME, &t(ME, Status::Ferme, false)));
        assert!(!can_cancel(ME, &t(ME, Status::Annule, false)));
        assert!(!can_cancel(ME, &t(OTHER, Status::Ouvert, false)), "pas l'auteur");
    }

    #[test]
    fn admin_status_changes() {
        let open = t(OTHER, Status::Ouvert, false);
        let closed = t(OTHER, Status::Ferme, true);
        assert!(can_set_status(Role::Admin, &open, Status::EnCours));
        assert!(can_set_status(Role::Admin, &closed, Status::Ouvert), "réouverture");
        assert!(!can_set_status(Role::Student, &open, Status::Ferme));
        assert!(!can_set_status(Role::Admin, &open, Status::Annule));
        assert!(!can_set_status(Role::Superadmin, &open, Status::Supprime));
        assert!(!can_set_status(Role::Admin, &t(OTHER, Status::Annule, false), Status::Ouvert));
    }

    #[test]
    fn reply_and_delete() {
        assert!(can_reply(Role::Admin, &t(OTHER, Status::Ferme, true)));
        assert!(!can_reply(Role::Admin, &t(OTHER, Status::Annule, false)));
        assert!(!can_reply(Role::Student, &t(ME, Status::Ouvert, false)));

        assert!(can_delete(Role::Superadmin, &t(OTHER, Status::Annule, false)));
        assert!(!can_delete(Role::Admin, &t(OTHER, Status::Ouvert, false)));
        assert!(!can_delete(Role::Superadmin, &t(OTHER, Status::Supprime, false)));
    }

    #[test]
    fn ticket_validation() {
        let cats = vec!["Cours".to_string(), "Autre".to_string()];
        let today = NaiveDate::from_ymd_opt(2026, 11, 20).unwrap();

        let ok = validate_new_ticket("2026-11-18", "Cours", "  M. Dupont ", "4", " Bien ", &cats, today)
            .unwrap();
        assert_eq!(ok.teacher, "M. Dupont");
        assert_eq!(ok.comment, "Bien");
        assert_eq!(ok.rating, 4);

        // Champs facultatifs vides, note 0 et date du jour acceptées.
        assert!(validate_new_ticket("2026-11-20", "Autre", "", "0", "", &cats, today).is_ok());

        let err = validate_new_ticket("", "Inconnue", "", "6", "", &cats, today).unwrap_err();
        assert_eq!(err.len(), 3);
        assert!(validate_new_ticket("2026-11-21", "Cours", "", "3", "", &cats, today).is_err());
        assert!(validate_new_ticket("2025-11-01", "Cours", "", "3", "", &cats, today).is_err());
        assert!(validate_new_ticket("2026-11-18", "Cours", "", "-1", "", &cats, today).is_err());
        assert!(validate_new_ticket("2026-11-18", "Cours", "", "", "", &cats, today).is_err());
        let long = "x".repeat(MAX_COMMENT_LEN + 1);
        assert!(validate_new_ticket("2026-11-18", "Cours", "", "3", &long, &cats, today).is_err());
    }

    #[test]
    fn reply_validation() {
        assert_eq!(validate_reply("  Merci  ").unwrap(), "Merci");
        assert!(validate_reply("   ").is_err());
        assert!(validate_reply(&"x".repeat(MAX_REPLY_LEN + 1)).is_err());
    }
}
