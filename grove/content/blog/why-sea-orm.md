---
title: "Why SeaORM over JavaScript client database options?"
description: ""
date: 2026-03-11
author: "Adefemi Adeoye"
tags: ["toolchain", "open-source", "rust", "engineering"]
---

---

## Overview

A few weeks ago, I made an announcement about [Wild Almonds](https://opeolluwa.github.io/almonds). This post reflects on some of the decisions that influenced the choice of technologies behind the project, starting with SeaORM.

In case you missed out on it, [Wild Almonds](https://opeolluwa.github.io/almonds) (or simply _**Almonds**_) is a developer productivity tool I'm building because it's tiring to track snippets with Github gist, while planning each day with Microsoft todo, while keeping reminders on phone, taking notes with Notion and more — I just want a single source.

That said, the choice of technology can greatly influence the performance and long-term viability of a product across several parameters, including size, speed, and security.ƒ

Rust was an easy choice because it checks many of these boxes. Its performance, memory safety guarantees, and strong ecosystem make it a compelling foundation for building reliable applications.

In this article I'm reflecting on one of the earliest architectural concerns I had when planning Wild Almonds's **data replication**, across devices. While the sync server is still a WIP, it is imperative that things are done right from the ground up.

The plan is simple, save on local first then replicate to cloud, I explain what mad me choose Sea-orm amidst the myriads of alternatives.

## Introduction

There are quite a number of offline-first client databases available today, including very ergonomic ones PouchDB, RxDB, and Dexi which I've interacted with at some point in the time past. However, it was important for me to streamline both forward and backward compatibility of the data model from the beginning. Managing NoSQL databases can quickly become murky, especially when the data eventually needs to integrate with relational databases such as Oracle, MySQL, or Postgres.

Amidst these considerations, SeaORM fits nicely into the architecture. It allows me to use SQLite locally for the client database while keeping a structured relational model. Combined with Seaography and some other features which I'll talk about sooner, this also makes it possible to replicate or expose the data to a cloud database in a consistent and maintainable way.

In the following section I'll highlight feature that resonated strongly with me starting with other database client in the Rust ecosystem

### SeaORM migration

```rust
use sea_orm_migration::{prelude::*, schema::*, sea_orm::DbBackend};

use crate::{
    m20260221_065202_create_reminder_table::Reminder,
    m20260224_214545_create_workspaces::Workspaces,
};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db_backend = manager.get_database_backend();
        let db_connection = manager.get_connection();
        if db_backend == DbBackend::Sqlite {
            manager
                .create_table(
                    Table::create()
                        .table("reminders_new")
                        .if_not_exists()
                        .col(pk_uuid(Reminder::Identifier))
                        // ... more lines removed
                        .to_owned(),
                )
                .await?;

            db_connection
                .execute_unprepared(
                    r#"
                    INSERT INTO "reminders_new" ("identifier", "title", "description", "recurring", "recurrence_rule", "alarm_sound", "remind_at", "created_at", "updated_at")
                    --- lines removed --
                    "#,
                )
                .await?;
            return Ok(());
        }

        manager
            .alter_table(
                Table::alter()
                    .table(Reminder::Table)
                    .add_column(ColumnDef::new("workspace_identifier").uuid())
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_reminder_workspace_identifier")
                    .from(Reminder::Table, "workspace_identifier")
                    .to(Workspaces::Table, "identifier")
                    .on_delete(ForeignKeyAction::Cascade)
                    .to_owned(),
            )
            .await
    }


```

### SeaORm entity generation

### Using Seaography

```sh

[working-directory: 'kernel']
@generate-entities url:
	sea-orm-cli generate entity \
		--database-url {{url}} \
		--with-serde both \
		--model-extra-attributes 'serde(rename_all="camelCase")' \
		-o src/entities

[working-directory: 'orchard']
@generate-graphql-bindings url:
	sea-orm-cli generate entity \
		--database-url {{url}} \
		-o src/entities \
		--seaography

```

### Role based access control

### Testing

## Conclusion

Aside the The main advantage of an ORM is abstraction and the code generation. Instead of writing raw SQL for every interaction, the data model is expressed through entities and relationships. This allows the application logic to remain largely independent of the underlying database engine.

For Wild Almonds, this flexibility is important. I do not want the application logic to depend heavily on whether the database is SQLite, MySQL, Postgres, or another supported backend. SeaORM provides a clean way to model the data while still allowing the project to remain portable across multiple database systems.

SeaORM helped unify a lot of code that would have otherwise become platform specific
