-- Add migration script here
CREATE TABLE issue_delivery_queue(
  issue_id uuid NOT NULL REFERENCES newsletter_issues(issue_id),
  email TEXT NOT NULL,
  PRIMARY KEY(issue_id, email)
);
