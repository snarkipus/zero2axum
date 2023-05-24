BEGIN TRANSACTION;
UPDATE subscriptions SET status = 'confirmed' WHERE status = NONE;
DEFINE FIELD status ON subscriptions TYPE string ASSERT $value != NONE;
COMMIT TRANSACTION;