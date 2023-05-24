DEFINE TABLE subscription_tokens SCHEMAFULL;

DEFINE FIELD subscription_token ON subscription_tokens TYPE string ASSERT $value != NONE;
DEFINE FIELD subscriber_id ON TABLE subscription_tokens TYPE string ASSERT $value != NONE;
DEFINE INDEX subscriber_id on TABLE subscription_tokens COLUMNS subscriber_id UNIQUE;