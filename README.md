# Phone Book API

![Rust](https://img.shields.io/badge/Rust-1.75+-black?logo=rust)
![Axum](https://img.shields.io/badge/Axum-0.7-blue)
![PostgreSQL](https://img.shields.io/badge/PostgreSQL-15+-blue?logo=postgresql)

A high-performance REST API for managing phone contacts, built with Rust and Axum, backed by PostgreSQL.

## Features

- **Full CRUD Operations**:
  - Create, read, and update contacts
  - Search by phone number
- **Robust Data Model**:
  - UUID primary keys
  - Automatic timestamps (`created_at`, `updated_at`)
- **Production-Ready**:
  - Database migrations (SQLx)
  - Configurable via environment variables
  - CORS support

## API Endpoints

| Method | Endpoint                | Description                          |
|--------|-------------------------|--------------------------------------|
| POST   | `/contacts`             | Create new contact                   |
| GET    | `/contacts`             | List all contacts                    |
| PUT    | `/contacts/{id}`        | Update contact by ID                 |
| GET    | `/contacts/search`      | Search contacts by phone number      |


  "created
