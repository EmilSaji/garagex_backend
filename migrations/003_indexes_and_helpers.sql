-- 003_indexes_and_helpers.sql
CREATE INDEX IF NOT EXISTS idx_system_users_phone ON system_users (phone);
CREATE INDEX IF NOT EXISTS idx_vehicles_vehicle_number ON vehicles (vehicle_number);
CREATE INDEX IF NOT EXISTS idx_jobs_garage_status ON jobs (garage_id, status);
CREATE INDEX IF NOT EXISTS idx_jobs_customer_phone ON jobs (customer_phone);
CREATE INDEX IF NOT EXISTS idx_garage_users_garage ON garage_users (garage_id);
CREATE INDEX IF NOT EXISTS idx_job_parts_job ON job_parts (job_id);
CREATE INDEX IF NOT EXISTS idx_invoice_items_invoice ON invoice_items (invoice_id);
CREATE INDEX IF NOT EXISTS idx_status_history_job ON job_status_history (job_id);
CREATE INDEX IF NOT EXISTS idx_notifications_recipient ON notifications (recipient_type, recipient_id);
CREATE INDEX IF NOT EXISTS idx_otp_phone ON otp_sessions (phone);
