DEFINE TABLE subscriptions SCHEMAFULL;

DEFINE FIELD email ON subscriptions TYPE string ASSERT $value != NONE AND is::email($value);
DEFINE INDEX email ON TABLE subscriptions COLUMNS email UNIQUE;
DEFINE FIELD name ON subscriptions TYPE string ASSERT $value != NONE;
DEFINE FIELD subscribed_at ON subscriptions TYPE datetime ASSERT $value != NONE;