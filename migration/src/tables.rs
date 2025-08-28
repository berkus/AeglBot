use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
pub enum Alerts {
    Table,
    Id,
    Guid,
    Title,
    Type,
    StartDate,
    ExpiryDate,
    Faction,
    Flavor,
}

#[derive(DeriveIden)]
pub enum Activities {
    Table,
    Id,
    Name,
    Mode,
    MinFireteamSize,
    MaxFireteamSize,
    MinLight,
    MinLevel,
}

#[derive(DeriveIden)]
pub enum ActivityShortcuts {
    Table,
    Id,
    Name,
    Game,
    Link,
}

#[derive(DeriveIden)]
pub enum PlannedActivities {
    Table,
    Id,
    AuthorId,
    ActivityId,
    Details,
    Start,
}

#[derive(DeriveIden)]
pub enum PlannedActivityMembers {
    Table,
    Id,
    PlannedActivityId,
    UserId,
    Added,
}

#[derive(DeriveIden)]
pub enum Guardians {
    Table,
    Id,
    TelegramName,
    TelegramId,
    PsnName,
    Email,
    PsnClan,
    CreatedAt,
    UpdatedAt,
    DeletedAt,
    Tokens,
    PendingActivationCode,
    IsAdmin,
    IsSuperadmin,
    RisingUid,
    RisingNickname,
}
