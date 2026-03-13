# EduSense — Journal des décisions d'architecture

> Ce fichier est un document vivant. Il évolue à chaque décision validée dans le projet.  
> Convention : chaque décision est datée, justifiée, et les alternatives rejetées sont documentées.

---

## Sommaire

- [Stack & Contexte](#stack--contexte)
- [Services](#services)
- [Décisions d'architecture](#décisions-darchitecture)
- [À décider](#à-décider)

---

## Stack & Contexte

| Élément | Choix | Statut |
|---|---|---|
| Langage back-end | Rust | ✅ Validé |
| Cible | Applications web (back-end) | ✅ Validé |
| Style | Micro-services interconnectés | ✅ Validé |
| Principe directeur | Security by design | ✅ Validé |

---

## Services

### 🔐 Auth Service *(en cours de développement)*

Service d'authentification — premier service développé, socle de sécurité pour tous les autres.

**Responsabilités :**
- Authentifier les utilisateurs
- Émettre des tokens/sessions
- Déléguer vers les providers d'identité

**Providers prévus :**
- [ ] Login / mot de passe (stockage local)
- [ ] Google OAuth
- [ ] SAML
- [ ] *(extensible)*

**Structure du service :**
```
src/
├── main.rs                 ← Démarrage serveur, assemblage des providers
├── routes.rs               ← Handler HTTP, dispatch vers les providers
└── providers/
    ├── mod.rs              ← Trait AuthProvider (le contrat)
    ├── local.rs            ← Provider local (login/password)
    ├── google.rs           ← Provider Google OAuth
    └── saml.rs             ← Provider SAML
```

---

## Décisions d'architecture

---

### [ADR-001] Modèle d'abstraction du service d'authentification

- **Date :** 2026-03-13
- **Statut :** ✅ Validé

#### Décision
Utilisation du **pattern Provider/Strategy** : un service d'auth unique exposant une interface unifiée, déléguant en interne vers des providers interchangeables.

```
Client → Auth Service → [ Provider Google ]
                      → [ Provider Local  ]
                      → [ Provider SAML   ]
```

En Rust, les providers seront modélisés via des **trait objects**, ce qui permet l'interchangeabilité et la testabilité.

#### Justification
- Point d'entrée unique → logique de session et de token centralisée
- Les `trait` Rust sont idiomatiques pour ce pattern
- Complexité opérationnelle maîtrisée à ce stade du projet

#### Alternatives rejetées

| Option | Description | Raison du rejet |
|---|---|---|
| API Gateway + micro-services spécialisés | Un service par provider, routage par gateway | Complexité réseau prématurée, surcoût opérationnel injustifié à ce stade |

#### Évolution possible
Si un provider devient critique (fort volume, équipe dédiée), il pourra être extrait en micro-service indépendant sans refonte de l'interface exposée aux autres services.

---

### [ADR-002] Framework HTTP retenu pour l'Auth Service

- **Date :** 2026-03-13
- **Statut :** ✅ Validé

#### Décision
Utilisation d'**Axum** comme framework HTTP, avec **Tokio** comme runtime asynchrone.

```toml
axum = "0.8"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
```

#### Justification
- Maintenu par l'équipe Tokio — garantie de cohérence entre le framework et le runtime
- Erreurs de compilation lisibles, pédagogiques pour une montée en compétence Rust
- Expose les concepts Rust fondamentaux (traits, types génériques) sans les masquer derrière des macros
- Adoption croissante dans l'écosystème Rust en 2026

#### Implémentation retenue
Le pattern Provider/Strategy est implémenté via des **trait objects** :

```rust
// Le trait définit le contrat commun
pub trait AuthProvider {
    fn name(&self) -> &str;
    fn authenticate(&self, username: &str, password: &str) -> bool;
}

// Les providers sont partagés entre threads via Arc
Arc<Vec<Box<dyn AuthProvider + Send + Sync>>>
```

- `Box<dyn AuthProvider>` — polymorphisme sur le heap, dispatch dynamique via vtable
- `Arc` — partage thread-safe sans copie entre les handlers HTTP
- `Send + Sync` — garanties de sécurité concurrente vérifiées à la compilation

#### Alternatives rejetées

| Option | Description | Raison du rejet |
|---|---|---|
| **Actix-web** | Framework async, runtime Actix | Macros plus opaques, concepts Rust moins visibles — moins adapté à un contexte pédagogique |
| **Warp** | Framework async, filtres composables | Messages d'erreur difficiles à déchiffrer, verbosité élevée |

#### Note sur HTTPS
En développement, le service tourne en HTTP pur sur le port 3000. En production, TLS sera géré par un reverse proxy (Nginx, Caddy) en amont du service — conformément au principe de séparation des responsabilités.

```
[Client] → HTTPS → [Reverse Proxy] → HTTP → [Auth Service :3000]
```

---

## À décider

> Les sujets suivants seront traités dans les prochaines étapes.

- [ ] Protocole de communication inter-services (REST / gRPC / message broker)
- [ ] Format de token (JWT, sessions serveur, PASETO…)
- [ ] Stratégie de stockage (base de données, type, ORM ou pas)
- [ ] Gestion des secrets et variables d'environnement
- [ ] Stratégie de tests

---

*Dernière mise à jour : 2026-03-13*
