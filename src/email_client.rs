use reqwest::Client;
use serde::Deserialize;

use crate::domain::SubscriberEmail;

pub struct EmailClient {
    sender: SubscriberEmail,
    http_client: Client,
    base_url: String,
}

impl EmailClient {
    pub fn new(base_url: String, sender: SubscriberEmail) -> Self {
        Self {
            sender,
            base_url,
            http_client: Client::new(),
        }
    }

    pub async fn send_email(
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), String> {
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use wiremock::matchers::any;
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[test]
    async fn send_email_fires_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let sender_email = SubscriberEmail::parse(SafeEmail().fake());
        let mock_client = EmailClient::new(mock_server.uri(), sender_email);

        Mock::given(any)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&mock_server)
            .await;

        let mock_recipient = SubscriberEmail::parse(SafeEmail().fake());
        let subject = Sentence(1..2).fake();
        let content = Paragraph(1..5).fake();

        let _ = mock_client.send_email(mock_recipient, &subject, &content, &content).await;

    }
}
