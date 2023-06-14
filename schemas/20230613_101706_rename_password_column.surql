BEGIN;
DEFINE FIELD password_hash ON users TYPE string ASSERT $value != NONE;
UPDATE users SET password_hash = password;
REMOVE FIELD password ON users;
UPDATE users;
COMMIT;