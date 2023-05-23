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
        timeout: std::time::Duration,
    ) -> Self {
        let http_client = Client::builder().timeout(timeout).build().unwrap();
        Self {
            http_client,
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
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_content,
            text_content,
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
            .await?
            .error_for_status()?;
        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_content: &'a str,
    text_content: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::{Paragraph, Sentence};
    use fake::{Fake, Faker};
    use mockito::{Matcher, Server};
    use secrecy::{ExposeSecret, Secret};
    use serde_json::json;

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake()
    }

    fn email() -> SubscriberEmail {
        SubscriberEmail::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            Secret::new(Faker.fake::<String>()),
            std::time::Duration::from_millis(200),
        )
    }

    #[tokio::test]
    async fn send_email_sends_the_expected_request() {
        // Arrange
        let mut server = Server::new_async().await;
        let email_client = email_client(server.url());
        let (email, subject, content) = (email(), subject(), content());

        let mock = server
            .mock("POST", "/email")
            .match_header(
                "X-Postmark-Server-Token",
                email_client.authorization_token.expose_secret().as_str(),
            )
            .match_header("Content-Type", "application/json")
            .match_body(Matcher::Json(json!({
                "From": email_client.sender.as_ref(),
                "To": email.as_ref(),
                "Subject": subject,
                "HtmlContent": content,
                "TextContent": "",
            })))
            .with_status(200)
            .create_async()
            .await;

        // Act
        let _ = email_client.send_email(email, &subject, &content, "").await;

        // Assert
        mock.assert_async().await;
    }

    #[tokio::test]
    async fn send_email_times_out_if_the_server_takes_too_long() {
        // Arrange
        let mut server = Server::new_async().await;
        let email_client = email_client(server.url());
        let (email, subject, content) = (email(), subject(), content());

        let mock = server
            .mock("POST", "/email")
            .with_status(200)
            .with_chunked_body(|writer| {
                std::thread::sleep(std::time::Duration::from_millis(500));
                writer.write_all(
                    json!({
                        "message": "success",
                        "status": "ok"
                    })
                    .to_string()
                    .as_bytes(),
                )
            })
            .create_async()
            .await;

        // Act
        let outcome = email_client.send_email(email, &subject, &content, "").await;

        // Assert
        mock.assert_async().await;
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        // Arrange
        let mut server = Server::new_async().await;
        let email_client = email_client(server.url());
        let (email, subject, content) = (email(), subject(), content());

        let mock = server
            .mock("POST", "/email")
            .with_status(500)
            .create_async()
            .await;

        // Act
        let outcome = email_client.send_email(email, &subject, &content, "").await;

        // Assert
        mock.assert_async().await;
        assert_err!(outcome);
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        // Arrange
        let mut server = Server::new_async().await;
        let email_client = email_client(server.url());
        let (email, subject, content) = (email(), subject(), content());

        let mock = server
            .mock("POST", "/email")
            .with_status(200)
            .create_async()
            .await;

        // Act
        let outcome = email_client.send_email(email, &subject, &content, "").await;

        // Assert
        mock.assert_async().await;
        assert_ok!(outcome);
    }
}
