DEFINE TABLE users SCHEMAFULL;

DEFINE FIELD username ON users TYPE string ASSERT $value != NONE;
DEFINE FIELD password ON users TYPE string ASSERT $value != NONE;
DEFINE INDEX username on TABLE users COLUMNS username UNIQUE;