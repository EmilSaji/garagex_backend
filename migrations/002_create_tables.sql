-- 002_create_tables.sql
-- System users (garage staff + admins)
CREATE TABLE IF NOT EXISTS system_users
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    username text UNIQUE,
    password_hash text,
    phone text NOT NULL,
    display_name text,
    email text,
    is_active boolean DEFAULT true,
    created_at timestamptz DEFAULT now
(
),
    updated_at timestamptz DEFAULT now
(
),
    deleted_at timestamptz
    );

-- Garages
CREATE TABLE IF NOT EXISTS garages
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    name text NOT NULL,
    address text,
    phone text,
    email text,
    metadata jsonb,
    created_at timestamptz DEFAULT now
(
),
    updated_at timestamptz DEFAULT now
(
),
    deleted_at timestamptz
    );

-- Garage users (map users to garage with role)
CREATE TABLE IF NOT EXISTS garage_users
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    garage_id uuid NOT NULL REFERENCES garages
(
    id
) ON DELETE CASCADE,
    username text ,
    password_hash text,
    display_name text NULL,
    phone text NULL,
    email text NULL,
    role text NOT NULL DEFAULT 'ADMIN', -- 'ADMIN' or 'MECHANIC'
    metadata jsonb DEFAULT '{}'::jsonb,
    is_active boolean NOT NULL DEFAULT true,
    created_at timestamptz NOT NULL DEFAULT now
(
),
    updated_at timestamptz NULL,
    deleted_at timestamptz NULL
    );

-- Customers (OTP login)
CREATE TABLE IF NOT EXISTS customers
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    phone text NOT NULL UNIQUE,
    name text,
    email text,
    created_at timestamptz DEFAULT now
(
),
    updated_at timestamptz DEFAULT now
(
)
    );

-- Vehicles
CREATE TABLE IF NOT EXISTS vehicles
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    customer_id uuid NOT NULL REFERENCES customers
(
    id
) ON DELETE CASCADE,
    vehicle_number text NOT NULL,
    make text,
    model text,
    year smallint,
    vin text,
    created_at timestamptz DEFAULT now
(
),
    updated_at timestamptz DEFAULT now
(
),
    UNIQUE
(
    customer_id,
    vehicle_number
)
    );

-- Jobs
CREATE TABLE IF NOT EXISTS jobs
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    job_identifier text NOT NULL UNIQUE,
    garage_id uuid NOT NULL REFERENCES garages
(
    id
) ON DELETE CASCADE,
    vehicle_id uuid REFERENCES vehicles
(
    id
),
    customer_phone text,
    customer_name text,
    complaint text,
    estimated_delivery_date date,
    estimated_time text,
    status job_status DEFAULT 'CREATED',
    created_by uuid REFERENCES system_users
(
    id
),
    current_assigned_to uuid REFERENCES system_users
(
    id
),
    remarks text,
    metadata jsonb,
    created_at timestamptz DEFAULT now
(
),
    updated_at timestamptz DEFAULT now
(
),
    deleted_at timestamptz
    );

-- Job status history
CREATE TABLE IF NOT EXISTS job_status_history
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    job_id uuid NOT NULL REFERENCES jobs
(
    id
) ON DELETE CASCADE,
    from_status job_status,
    to_status job_status NOT NULL,
    changed_by uuid REFERENCES system_users
(
    id
),
    note text,
    created_at timestamptz DEFAULT now
(
)
    );

-- Parts catalog
CREATE TABLE IF NOT EXISTS parts_catalog
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    sku text UNIQUE,
    name text NOT NULL,
    description text,
    unit_price numeric
(
    12,
    2
),
    tax_percent numeric
(
    5,
    2
) DEFAULT 0,
    created_at timestamptz DEFAULT now
(
),
    updated_at timestamptz DEFAULT now
(
)
    );

-- Job parts
CREATE TABLE IF NOT EXISTS job_parts
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    job_id uuid NOT NULL REFERENCES jobs
(
    id
) ON DELETE CASCADE,
    part_id uuid REFERENCES parts_catalog
(
    id
),
    name text NOT NULL,
    quantity integer DEFAULT 1,
    unit_price numeric
(
    12,
    2
) NOT NULL,
    tax_percent numeric
(
    5,
    2
) DEFAULT 0,
    created_at timestamptz DEFAULT now
(
)
    );

-- Invoices
CREATE TABLE IF NOT EXISTS invoices
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    job_id uuid NOT NULL REFERENCES jobs
(
    id
) UNIQUE,
    invoice_number text UNIQUE,
    parts_subtotal numeric
(
    12,
    2
) DEFAULT 0,
    labor_charge numeric
(
    12,
    2
) DEFAULT 0,
    tax_amount numeric
(
    12,
    2
) DEFAULT 0,
    total_amount numeric
(
    12,
    2
) DEFAULT 0,
    include_tax boolean DEFAULT true,
    created_by uuid REFERENCES system_users
(
    id
),
    created_at timestamptz DEFAULT now
(
),
    whatsapp_sent boolean DEFAULT false,
    whatsapp_sent_at timestamptz
    );

-- Invoice items
CREATE TABLE IF NOT EXISTS invoice_items
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    invoice_id uuid NOT NULL REFERENCES invoices
(
    id
) ON DELETE CASCADE,
    description text NOT NULL,
    quantity integer DEFAULT 1,
    unit_price numeric
(
    12,
    2
) NOT NULL,
    tax_percent numeric
(
    5,
    2
) DEFAULT 0,
    line_total numeric
(
    12,
    2
) NOT NULL,
    created_at timestamptz DEFAULT now
(
)
    );

-- Notifications
CREATE TABLE IF NOT EXISTS notifications
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    recipient_type text NOT NULL,
    recipient_id uuid NOT NULL,
    title text,
    body text,
    related_job uuid REFERENCES jobs
(
    id
),
    is_read boolean DEFAULT false,
    channel text,
    metadata jsonb,
    created_at timestamptz DEFAULT now
(
)
    );

-- OTP sessions (short lived) - use Redis in prod if you prefer
CREATE TABLE IF NOT EXISTS otp_sessions
(
    id
    uuid
    PRIMARY
    KEY
    DEFAULT
    gen_random_uuid
(
),
    phone text NOT NULL,
    otp_code text NOT NULL,
    purpose text,
    expires_at timestamptz NOT NULL,
    attempts smallint DEFAULT 0,
    created_at timestamptz DEFAULT now
(
)
    );
