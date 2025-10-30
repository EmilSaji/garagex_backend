-- 001_create_extensions_and_enums.sql
CREATE EXTENSION IF NOT EXISTS "pgcrypto";  -- for gen_random_uuid()

-- Job status enum
DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'job_status') THEN
CREATE TYPE job_status AS ENUM (
          'CREATED',
          'PENDING_INSPECTION',
          'WAITING_FOR_PARTS',
          'UNDER_REPAIR',
          'READY',
          'DELIVERED'
        );
END IF;
END$$;

DO $$ BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'garage_user_role') THEN
CREATE TYPE garage_user_role AS ENUM ('GARAGE_ADMIN', 'MECHANIC');
END IF;
END$$;
