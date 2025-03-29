-- Add migration script here
ALTER TABLE issue_delivery_queue
ADD retries INT DEFAULT 0;
