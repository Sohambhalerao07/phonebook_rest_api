use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, FromRow, Pool, Postgres};
use tower_http::cors::CorsLayer;
use uuid::Uuid;

#[derive(Clone)]
struct AppState {
    db_pool: Pool<Postgres>,
}

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
struct Contact {
    id: Uuid,
    first_name: String,
    last_name: String,
    phone: String,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct CreateContact {
    first_name: String,
    last_name: String,
    phone: String,
}

#[derive(Debug, Deserialize)]
struct UpdateContact {
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set in .env file");
    
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await?;
    
    let state = AppState { db_pool };
    
    let app = Router::new()
        .route("/contacts", get(list_contacts).post(create_contact))
        .route("/contacts/:id", put(update_contact))
        .route("/contacts/search", get(search_contact_by_phone))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await?;
    println!("Listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn list_contacts(State(state): State<AppState>) -> Result<Json<Vec<Contact>>, (StatusCode, String)> {
    sqlx::query_as::<_, Contact>("SELECT * FROM contacts ORDER BY first_name, last_name")
        .fetch_all(&state.db_pool)
        .await
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn create_contact(
    State(state): State<AppState>,
    Json(payload): Json<CreateContact>,
) -> Result<(StatusCode, Json<Contact>), (StatusCode, String)> {
    let contact = sqlx::query_as::<_, Contact>(
        r#"
        INSERT INTO contacts (id, first_name, last_name, phone)
        VALUES ($1, $2, $3, $4)
        RETURNING *
        "#
    )
    .bind(Uuid::new_v4())
    .bind(payload.first_name)
    .bind(payload.last_name)
    .bind(payload.phone)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok((StatusCode::CREATED, Json(contact)))
}

async fn update_contact(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateContact>,
) -> Result<Json<Contact>, (StatusCode, String)> {
    let mut contact = sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| match e {
        sqlx::Error::RowNotFound => (StatusCode::NOT_FOUND, "Contact not found".to_string()),
        _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    })?;
    
    if let Some(first_name) = payload.first_name {
        contact.first_name = first_name;
    }
    if let Some(last_name) = payload.last_name {
        contact.last_name = last_name;
    }
    if let Some(phone) = payload.phone {
        contact.phone = phone;
    }
    
    let updated_contact = sqlx::query_as::<_, Contact>(
        r#"
        UPDATE contacts
        SET 
            first_name = $1,
            last_name = $2,
            phone = $3,
            updated_at = NOW()
        WHERE id = $4
        RETURNING *
        "#
    )
    .bind(&contact.first_name)
    .bind(&contact.last_name)
    .bind(&contact.phone)
    .bind(id)
    .fetch_one(&state.db_pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(updated_contact))
}

#[derive(Deserialize)]
struct SearchQuery {
    phone: String,
}

async fn search_contact_by_phone(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<Vec<Contact>>, (StatusCode, String)> {
    sqlx::query_as::<_, Contact>(
        "SELECT * FROM contacts WHERE phone = $1"
    )
    .bind(query.phone)
    .fetch_all(&state.db_pool)
    .await
    .map(Json)
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}
