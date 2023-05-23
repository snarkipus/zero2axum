use reqwest::Client;
use secrecy::{ExposeSecret, Secret};

use crate::domain::SubscriberEmail;

#[derive(Debug, Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: String,
    sender: SubscriberEmail,
    authorization_token: Secret<String>,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: SubscriberEmail,
        authorization_token: Secret<String>,
    ) -> Self {
        Self {
            http_client: Client::new(),
            base_url,
            sender,
            authorization_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = format!("{}/email", self.base_url);
        let request_body = SendEmailRequest {
            from: self.sender.as_ref().to_owned(),
            to: recipient.as_ref().to_owned(),
            subject: subject.to_owned(),
            html_content: html_content.to_owned(),
            text_content: text_content.to_owned(),
        };

        let _builder = self
            .http_client
            .post(url)
            .header(
                "X-Postmark-Server-Token",
                self.authorization_token.expose_secret(),
            )
            .json(&request_body)
            .send()
            .await?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest {
    from: String,
    to: String,
    subject: String,
    html_content: String,
    text_content: String,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use mockito::{Server, Matcher};
    use secrecy::Secret;
    use serde_json::json;

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mut server = Server::new_async().await;
        let sender = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let authorization_token = Faker.fake::<String>();
        let email_client = EmailClient::new(
            server.url(),
            sender.clone(),
            Secret::new(authorization_token.clone()),
        );

        let subscriber_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
        let subject = Sentence(1..2).fake::<String>();
        let html_content = Paragraph(1..10).fake::<String>();

        let mock = server
            .mock("POST", "/email")
            .match_header("X-Postmark-Server-Token", authorization_token.as_str())
            .match_header("Content-Type", "application/json")
            .match_body(
                Matcher::JsonString(
                    json!({
                        "From": sender.as_ref(),
                        "To": subscriber_email.as_ref(),
                        "Subject": subject,
                        "HtmlContent": html_content,
                        "TextContent": "",
                    })
                    .to_string(),
                ),
            )
            .with_status(200)
            .create_async()
            .await;



        // Act
        let _ = email_client
            .send_email(subscriber_email, &subject, &html_content, "")
            .await;

        // Assert
        mock.assert();
    }
}
