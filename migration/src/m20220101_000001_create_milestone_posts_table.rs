use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(MilestonePost::Table)
                    .if_not_exists()
                    .col(pk_auto(MilestonePost::Id))
                    .col(string(MilestonePost::Title))
                    .col(big_unsigned(MilestonePost::IssueId))
                    .col(string(MilestonePost::Milestone))
                    .to_owned(),
            )
            .await?;

        // 添加索引
        manager.create_index(
            Index::create()
                .name("idx-issueid-milestone")
                .table(MilestonePost::Table)
                .col(MilestonePost::IssueId)
                .col(MilestonePost::Milestone)
                .to_owned()
        ).await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(MilestonePost::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum MilestonePost {
    Table,
    Id,
    Title,
    IssueId,
    Milestone
}
