use std::time::Duration;

use reqwest::{Client, ClientBuilder};
use secrecy::{ExposeSecret, Secret};
use serde::Serialize;

use crate::domain::SubscriberEmail;

#[derive(Debug)]
pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
    auth_token: Secret<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")] // Mock expects in this format ?
pub struct SendEmailRequest<'a> {
    to: &'a str,
    from: &'a str,
    subject: &'a str,
    text_body: &'a str,
    html_body: &'a str,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail, auth_token: Secret<String>, timeout: Duration) -> Self {
        let http_client = ClientBuilder::new()
                .timeout(timeout)
                .build()
                .unwrap();

        Self {
            sender,
            base_url,
            http_client,
            auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: &SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> reqwest::Result<reqwest::Response> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };
        let response = self.http_client
            .post(&url)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .json(&request_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use serde_json::Value;
    use tokio;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Match, Mock, MockServer, ResponseTemplate};

    // helpers

    /// Get a test instance of `EmailClient`.
    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(base_url, email(), Secret::new(Faker.fake()), Duration::from_millis(200))
    }

    /// Generate a random email subject
    fn subject() -> String {
        Sentence(1..2).fake()
    }

    /// Generate a random email content
    fn content() -> String {
        Paragraph(1..10).fake()
    }

    /// Generate a random subscriber email
    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    struct SendEmailBodyMatcher;
    impl Match for SendEmailBodyMatcher {
        fn matches(&self, request: &wiremock::Request) -> bool {
            let result: Result<Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("To").is_some()
                    && body.get("From").is_some()
                    && body.get("Subject").is_some()
                    && body.get("HtmlBody").is_some()
                    && body.get("TextBody").is_some()
            } else {
                return false;
            }
        }
    }

    #[tokio::test]
    async fn send_email_is_ok_if_server_returns_200() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let response = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_ok!(response);
    }

    #[tokio::test]
    async fn send_email_is_err_if_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(500))
            .expect(1)
            .mount(&mock_server)
            .await;

        let response = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_err!(response);
    }

    #[tokio::test]
    async fn fails_if_timeout_exceeded() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        // here
        let response_tempalte = ResponseTemplate::new(200).set_delay(Duration::from_millis(500));

        Mock::given(any())
            .respond_with(response_tempalte)
            .expect(1)
            .mount(&mock_server)
            .await;

        let response = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;

        assert_err!(response);
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        // Act
        let _ = email_client
            .send_email(&email(), &subject(), &content(), &content())
            .await;
    }
}
