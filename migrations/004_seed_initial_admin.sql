-- 004_seed_initial_admin.sql
-- WARNING: plaintext password for dev only. Replace with hashed password for prod.

INSERT INTO system_users (id, username, password_hash, phone, display_name, email, is_active, created_at)
VALUES (
           gen_random_uuid(),
           'admin',                     -- change username as you like
           'admin123',                  -- <-- DEV plaintext password (replace/hash in prod)
           '0000000000',
           'Super Admin',
           'admin@example.com',
           true,
           now()
       )
    ON CONFLICT (username) DO NOTHING;
