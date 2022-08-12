use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SenderInfo {
    pub user_id: String,    // ex stytch user id
    pub first_name: String, // invitor's first name
    pub last_name: String,  // invitor's last name
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReceiverInfo {
    pub email: String,      // invitee's email
    pub phone: String,      // invitee's phone
    pub first_name: String, // invitee's first name
    pub last_name: String,  // invitee's last name
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InviteUser {
    pub sender: SenderInfo,
    pub receivers: Vec<ReceiverInfo>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InviteLink {
    pub link: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InviteCheck {
    pub hash: String,
    pub account: String, // email or phone
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CheckResult {
    pub invitors: Vec<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailBody {
    pub From: String,
    pub To: String,
    pub Subject: String,
    pub TextBody: String,
    pub MessageStream: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SmsBody {
    pub From: String,
    pub To: String,
    pub Body: String,
}