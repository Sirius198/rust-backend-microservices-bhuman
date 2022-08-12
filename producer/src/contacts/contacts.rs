use derive_more::From;

use serde::Deserialize;
use serde::Serialize;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NameInfo {
    pub displayName: Option<String>,
    pub familyName: Option<String>,
    pub givenName: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhotoInfo {
    pub url: Option<String>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PhoneInfo {
    pub value: String,
    pub formattedType: String,
}

impl PhoneInfo {
    pub fn e164_format(&mut self) {
        let mut filter: String = self.value.chars().filter(|c| c.is_digit(10)).collect();                        
        if filter.len() == 0 {
            return;
        }

        let ch = filter.chars().nth(0).unwrap();
        if ch != '+' {
            filter.insert_str(0, "+");
            self.value = filter;
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailInfo {
    pub value: String,
    pub formattedType: String,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactInfo {
    pub names: Option<Vec<NameInfo>>,
    pub photos: Option<Vec<PhotoInfo>>,
    pub phoneNumbers: Option<Vec<PhoneInfo>>,
    pub emailAddresses: Option<Vec<EmailInfo>>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactList {
    pub connections: Vec<ContactInfo>
}

impl From<String> for ContactList {
    fn from(json: String) -> Self {

        let mut conns = Vec::new();
        let decoded: serde_json::Value = serde_json::from_str(&json).unwrap();
        let list: Vec<serde_json::Value> = serde_json::from_value(decoded["value"].clone()).unwrap();
        for item in list.iter() {
            let mut emails = Vec::new();
            let emailList: Vec<serde_json::Value> = serde_json::from_value(item["emailAddresses"].clone()).unwrap();
            for item in emailList {
                emails.push( {
                    EmailInfo {
                        value: item["address"].to_string(),
                        formattedType: item["name"].to_string(),
                    }
                });
            }

            let mut mobilePhone = String::from("");
            let phone = Some(item["mobilePhone"].clone());
            if phone.is_some() {
                mobilePhone = phone.unwrap().to_string();
            }

            let url = Some(format!("https://graph.microsoft.com/v1.0/users/{}/photo/$value", item["id"]));

            let contact = ContactInfo {
                names: Some(Vec::from([
                    NameInfo {
                        displayName: Some(item["displayName"].to_string()),
                        familyName: Some(item["surname"].to_string()),
                        givenName: Some(item["givenName"].to_string()),
                    },
                ])),
                emailAddresses: Some(emails),
                phoneNumbers: Some(Vec::from([
                    PhoneInfo {
                        value: mobilePhone,
                        formattedType: "Mobile".to_string(),
                    }
                ])),
                photos: Some(Vec::from([
                    PhotoInfo {
                        url
                    }
                ])),
            };

            conns.push(contact);
        }

        Self {
            connections: conns,
        }        
    }
}

impl ContactList {
    pub fn e164_format(&mut self) {
        for connection in &mut self.connections {
            match &mut connection.phoneNumbers {
                Some(ref mut phones) => {
                     for phone in phones {
                        phone.e164_format();                
                    }
                }
                None => {}
            }
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactSync {
    pub user_id: String,
    pub email: String,
    pub phone: String,
    pub provider: String,
    pub token: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactRes {
    pub user_id: String,
    pub email: String,
    pub phone: String,
    pub provider: String,
    pub contacts: String,
}

impl Default for ContactRes {
    fn default() -> Self {
        Self {
            user_id: String::from(""),
            email: String::from(""),
            phone: String::from(""),
            provider: String::from(""),
            contacts: String::from(""),
        }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ContactQuery {
    pub user_id: String,
    pub provider: String,
}