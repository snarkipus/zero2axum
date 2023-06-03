DEFINE TABLE subscription_tokens SCHEMAFULL;

DEFINE FIELD subscription_token ON subscription_tokens TYPE string ASSERT $value != NONE;
DEFINE INDEX subscription_token on TABLE subscription_tokens COLUMNS subscription_token UNIQUE;