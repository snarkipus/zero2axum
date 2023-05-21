use crate::domain::{SubscriberEmail, SubscriberName};
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
