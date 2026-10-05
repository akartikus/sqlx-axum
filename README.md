# ticket-api

API REST minimale en Rust (Axum + SQLx + PostgreSQL) servant de référence pour une structure de projet **idiomatique**.

Ce document explique l'organisation du code, le rôle de chaque couche, et les concepts Rust mobilisés. Il est pensé comme un guide d'apprentissage, pas seulement comme une doc technique.

---

## Stack

| Rôle | Crate |
|---|---|
| Runtime asynchrone | `tokio` |
| Framework HTTP | `axum` |
| Accès BDD (compile-time checked SQL) | `sqlx` |
| BDD | PostgreSQL |
| Erreurs typées | `thiserror` |
| Erreurs "boîte noire" (démarrage) | `anyhow` |
| Logging structuré | `tracing` + `tracing-subscriber` |
| Middlewares HTTP | `tower-http` |
| Sérialisation JSON | `serde` / `serde_json` |
| Dates/heures | `time` |

---

## Démarrage rapide

```sh
# 1. Variables d'environnement
cp .env.example .env
# édite DATABASE_URL=postgres://user:pass@localhost:5432/ticket_api

# 2. Lancer une base Postgres (Docker, par exemple)
docker run --name ticket-pg -e POSTGRES_PASSWORD=pass -p 5432:5432 -d postgres:16

# 3. Build + run
cargo run

# Le serveur écoute sur http://localhost:3000
```

Les migrations SQL dans `migrations/` sont appliquées automatiquement au démarrage (voir `src/db.rs`).

---

## Endpoints

| Méthode | Chemin | Description |
|---|---|---|
| `POST` | `/api/tickets` | Crée un ticket (titre requis, description optionnelle) |
| `GET` | `/api/tickets` | Liste tous les tickets (du plus récent au plus ancien) |
| `GET` | `/api/tickets/{id}` | Récupère un ticket par son id |

Exemple :

```sh
curl -X POST http://localhost:3000/api/tickets \
  -H "Content-Type: application/json" \
  -d '{"title":"Mon ticket","description":"Un test"}'
```

---

## Structure du projet

```
ticket-api/
├── Cargo.toml
├── .env.example
├── migrations/
│   └── 20261005092631_create_tickets.sql
└── src/
    ├── main.rs           # point d'entrée binaire
    ├── lib.rs            # déclaration des modules du crate
    ├── config.rs         # chargement de la configuration (env)
    ├── db.rs             # pool Postgres + exécution des migrations
    ├── state.rs          # AppState partagé entre les handlers
    ├── error.rs          # type d'erreur unifié (AppError)
    ├── app.rs            # assemblage du routeur racine
    └── ticket/           # fonctionnalité "ticket" (feature-based)
        ├── mod.rs        #   re-export des sous-modules
        ├── model.rs      #   entité métier (Ticket)
        ├── dto.rs        #   requêtes/réponses HTTP
        ├── repository.rs #   accès BDD (générique sur PgExecutor)
        ├── service.rs    #   logique métier + validation
        ├── handler.rs    #   contrôleurs HTTP (Axum)
        └── routes.rs     #   définition des routes ticket
```

### Pourquoi cette organisation est "idiomatique"

1. **Feature-based, pas type-based.** Le dossier `ticket/` regroupe TOUT ce qui concerne les tickets (modèle, DTO, repo, service, handlers). Quand tu ajouteras `users/`, tu feras de même. On évite les dossiers globaux `controllers/`, `services/`, `repositories/` qui mélangent des responsabilités sans rapport.

2. **`main.rs` / `lib.rs` séparés.** `main.rs` ne contient que le bootstrapping (très court). Toute la logique est dans `lib.rs` — ce qui permet de tester le crate en tant que bibliothèque sans lancer un serveur.

3. **Pas de couches abstraites "pour être propre".** Pas de trait `Repository` ou `Service` partout. Les fonctions ordinaires suffisent. Rust préfère le concret au pattern-abstraction prématuré (contrairement à Java/Spring). On n'abstrait que quand on a plusieurs implémentations.

4. **Une seule erreur applicative (`AppError`)** qui implémente `IntoResponse`. Les handlers retournent `Result<T, AppError>` et Axum convertit automatiquement en réponse HTTP. Pas de `unwrap()`, pas de panics côté métier.

5. **Repository générique sur `PgExecutor`** — pas couplé à `&PgPool`. Permet d'exécuter les requêtes dans une transaction (pour les tests ou les workflows multi-étapes) sans dupliquer le code.

6. **DTOs séparés du modèle métier.** `Ticket` (model) ≠ `TicketResponse` (dto). L'API peut évoluer (rename, champs cachés) sans toucher au métier, et inversement.

---

## Anatomie des couches

### `main.rs` — Bootstrap

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()...init();

    let config = Config::from_env()?;
    let db = db::connect(&config.database_url).await?;
    db::migrate(&db).await?;

    let state = AppState { db };
    let app = app::router(state);

    let listener = TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}
```

- `anyhow::Result` ici : les erreurs au démarrage sont fatales, on n'a pas besoin de les différencier. `anyhow` est l'outil parfait pour ça.
- Tout le reste est délégué aux modules.

### `lib.rs` — Racine du crate

```rust
pub mod app;
pub mod config;
pub mod db;
pub mod error;
pub mod state;
pub mod ticket;
```

Juste la déclaration des modules. Le point d'entrée de tout le code testable.

### `config.rs` — Configuration

```rust
pub struct Config {
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> { ... }
}
```

Centralise le chargement des variables d'environnement. Une seule structure typée plutôt que des `env::var(...)` éparpillés.

### `db.rs` — Pool + migrations

```rust
pub async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> { ... }

pub async fn migrate(pool: &PgPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
```

- `PgPool` : un pool de connexions réutilisables (les connexions sont chères à créer).
- `sqlx::migrate!` est une macro compile-time : le chemin `./migrations` est vérifié à la compilation, pas au runtime.

### `state.rs` — État partagé

```rust
#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}
```

`#[derive(Clone)]` est **obligatoire** : Axum clone le state pour chaque requête. Pas de souci : `PgPool` est bon marché à cloner (c'est un `Arc` en interne, pas une nouvelle connexion).

### `error.rs` — Erreur applicative unifiée

```rust
#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,
    #[error("{0}")]
    Validation(String),
    #[error(transparent)]
    Database(#[from] sqlx::Error),
}
```

- `thiserror` génère `Display`, `From<sqlx::Error>`, etc. automatiquement.
- `#[from]` permet `?` pour propager les erreurs SQL automatiquement.
- L'impl `IntoResponse` mappe chaque variante à un status HTTP :
  - `NotFound` → 404
  - `Validation` → 400
  - `Database` → 500 (avec log détaillé côté serveur, message générique au client)

### `app.rs` — Routeur racine

```rust
pub fn router(state: AppState) -> Router {
    Router::<AppState>::new()
        .nest("/api", ticket::routes::router())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
```

- `.nest("/api", ...)` : monte le sous-routeur ticket sous le préfixe `/api`.
- `.layer(TraceLayer::...)` : middleware global (logging HTTP).
- `.with_state(state)` : "scelle" l'état dans le routeur. Indispensable pour qu'Axum puisse le fournir aux handlers via `State(state)`.

### `ticket/` — La fonctionnalité

#### `model.rs` — Entité métier

```rust
#[derive(Debug)]
pub struct Ticket {
    pub id: i32,
    pub title: String,
    pub description: Option<String>,
    pub created_at: OffsetDateTime,
}
```

Représentation "domaine" d'un ticket, mappée sur la table SQL. Pas de logique HTTP ici.

#### `dto.rs` — Data Transfer Objects

```rust
#[derive(Deserialize)]
pub struct CreateTicketRequest { ... }  // corps de POST /tickets

#[derive(Serialize)]
pub struct TicketResponse { ... }        // corps des réponses

impl From<Ticket> for TicketResponse { ... }
```

- `Deserialize` pour parser le JSON entrant.
- `Serialize` pour générer le JSON sortant.
- `From<Ticket>` pour convertir sans bruit dans le handler.

#### `repository.rs` — Accès BDD

```rust
pub async fn find_by_id<'e, E>(executor: E, id: i32) -> Result<Option<Ticket>, sqlx::Error>
where
    E: PgExecutor<'e>,
{
    sqlx::query_as!(Ticket, "SELECT ... WHERE id = $1", id)
        .fetch_optional(executor)
        .await
}
```

- **`sqlx::query_as!`** vérifie la SQL **à la compilation** contre la BDD (via `DATABASE_URL` à `cargo build`). Une colonne inexistante = erreur de compile, pas d'erreur runtime.
- **Générique sur `E: PgExecutor<'e>`** : la fonction accepte indifféremment `&PgPool` ou `&mut Transaction<...>`. Voir "Concepts Rust" ci-dessous.
- Retourne `Option<Ticket>` car l'id peut ne pas exister (le `None` n'est pas une erreur, juste une absence).

#### `service.rs` — Logique métier

```rust
pub async fn create(db: &PgPool, input: CreateTicketRequest) -> Result<Ticket, AppError> {
    let title = input.title.trim();
    if title.is_empty() {
        return Err(AppError::Validation("title cannot be empty".into()));
    }
    if title.chars().count() > 255 {
        return Err(AppError::Validation("title cannot exceed 255 characters".into()));
    }
    Ok(repository::create(db, title, input.description.as_deref()).await?)
}
```

- Valide les règles métier (ici la longueur du titre).
- Délègue au repository pour la persistance.
- Convertit les `sqlx::Error` en `AppError` automatiquement via `?` + `#[from]`.

#### `handler.rs` — Contrôleurs HTTP

```rust
pub async fn create(
    State(state): State<AppState>,
    Json(input): Json<CreateTicketRequest>,
) -> Result<(StatusCode, Json<TicketResponse>), AppError> {
    let ticket = service::create(&state.db, input).await?;
    Ok((StatusCode::CREATED, Json(ticket.into())))
}
```

- `State(state): State<AppState>` : extracteur Axum pour récupérer l'état injecté via `.with_state()`.
- `Json(input): Json<CreateTicketRequest>` : extrait + parse le corps JSON.
- Retourne `(StatusCode, Json<TicketResponse>)` : Axum assemble la réponse HTTP.

Le handler ne fait pas de SQL, ne valide pas, ne sérialise pas à la main. Il orchestre uniquement.

#### `routes.rs` — Définition des routes

```rust
pub fn router() -> Router<AppState> {
    Router::new()
        .route("/tickets", post(handler::create).get(handler::list))
        .route("/tickets/{id}", get(handler::get_by_id))
}
```

Retourne un `Router<AppState>` (l'état n'est PAS encore attaché — c'est `app.rs` qui le fera via `.with_state()`). Cela permet de réutiliser ce routeur dans différents contextes (prod, tests, etc.).

---

## Concepts Rust mobilisés

### 1. Généricité + lifetimes (`<'e, E>`)

```rust
pub async fn find_by_id<'e, E>(executor: E, id: i32) -> Result<Option<Ticket>, sqlx::Error>
where
    E: PgExecutor<'e>,
```

- `E` : un type paramètre (l'exécuteur SQL).
- `'e` : un lifetime, indiquant combien de temps vivent les ressources de `E`.
- `where E: PgExecutor<'e>` : contrainte — `E` doit pouvoir exécuter des requêtes Postgres pour le lifetime `'e`.

Permet d'appeler `find_by_id` avec un `&PgPool` ou un `&mut Transaction<...>` sans dupliquer le code.

### 2. `Result<T, E>` et l'opérateur `?`

```rust
let ticket = repository::create(db, title, ...).await?;  // propage l'erreur
```

`?` dépaquette le `Ok` ou retourte tôt avec l'`Err`. Couplé à `#[from]` sur `AppError::Database`, les erreurs `sqlx::Error` sont automatiquement converties.

### 3. Pattern matching dans les paramètres

```rust
State(state): State<AppState>,
```

Destructuration de l'extracteur : `State<AppState>` est le type, `State(state)` déstructure et lie la valeur interne à `state`.

### 4. `From` / `Into`

```rust
impl From<Ticket> for TicketResponse { ... }
// puis dans le handler:
Json(ticket.into())
```

`.into()` convertit `Ticket` en `TicketResponse` grâce à l'impl `From`. Rust privilégie ces conversions explicites typées.

### 5. Pas d'héritage, de la composition

Pas de classes abstraites. La composition se fait via :
- des **modules** (organisation du code),
- des **traits** (partage de comportement — utilisé par Axum avec `PgExecutor`, `IntoResponse`, etc.),
- des **génériques + bounds** (paramétrage de fonctions par type).

---

## Patterns à retenir

| Pattern | Pourquoi |
|---|---|
| `anyhow` au démarrage, `thiserror` dans le métier | `anyhow` : erreurs fatales non typées. `thiserror` : erreurs typées à mapper vers HTTP. |
| `query_as!` plutôt que `FromRow` | SQL vérifié à la compile. Une faute de frappe dans une colonne = erreur de build. |
| `AppState` + `Clone` | Permet à Axum de passer l'état à chaque requête sans locks globaux. |
| DTO ≠ Model | Découplage API / métier. Évolution indépendante. |
| Feature-based folders (`ticket/`) | Cohésion du code lié à une fonctionnalité. |
| `service` retourne `AppError`, `repository` retourne `sqlx::Error` | Le repo parle BDD, le service parle métier. Conversion au passage. |

---

## Patterns à éviter (anti-Spring)

- ❌ Traits `TicketRepository` / `TicketServiceImpl` pour "faire propre" sans plusieurs implémentations.
- ❌ Dossier global `controllers/` mélangant des fonctionnalités sans rapport.
- ❌ `unwrap()` / `expect()` dans le métier (préfère `Result`).
- ❌ `Box<dyn Trait>` quand un type concret suffit.
- ❌ État global mutable (`lazy_static`, `OnceCell`) — passe par `AppState`.

---

## Développement

```sh
cargo check                     # vérification rapide (sans linker)
cargo build                     # compile le binaire
cargo run                       # build + run
cargo clippy -- -W clippy::all  # linter rust idiomatique
cargo fmt                       # formate le code
cargo test                      # lance les tests (quand il y en aura)
```

Pour les tests d'intégration avec BDD, crée un dossier `tests/` à la racine. Le repository générique sur `PgExecutor` permet de tester dans une transaction qu'on rollback à la fin — pattern classique en Rust/sqlx.

---

## Ajouter une nouvelle fonctionnalité

1. Crée `src/<feature>/` avec les mêmes sous-modules : `mod.rs`, `model.rs`, `dto.rs`, `repository.rs`, `service.rs`, `handler.rs`, `routes.rs`.
2. Déclare `pub mod <feature>;` dans `src/lib.rs`.
3. Monte-le dans `src/app.rs` : `.nest("/api/<feature>", <feature>::routes::router())`.
4. Crée une migration dans `migrations/` (timestamp auto via `sqlx migrate add`).

---

## Références

- [Axum documentation](https://docs.rs/axum)
- [SQLx documentation](https://docs.rs/sqlx)
- [The Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Zero To Production In Rust (Luca Palmieri)](https://www.zero2prod.com/) — référence livre pour ce type d'architecture
