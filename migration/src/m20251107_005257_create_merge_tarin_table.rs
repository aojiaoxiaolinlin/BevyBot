use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MergeTrain::Table)
                    .if_not_exists()
                    .col(pk_auto(MergeTrain::Id))
                    .col(string(MergeTrain::Title))
                    .col(string(MergeTrain::Cid))
                    .to_owned(),
            )
            .await?;

        manager.create_index(
            Index::create()
                .name("idx-cid")
                .table(MergeTrain::Table)
                .col(MergeTrain::Cid)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MergeTrain::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MergeTrain {
    Table,
    Id,
    Title,
    Cid
}
