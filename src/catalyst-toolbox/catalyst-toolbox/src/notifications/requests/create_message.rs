use color_eyre::{
    eyre::{bail, eyre},
    Report,
};
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use time::{format_description::FormatItem, macros::format_description, OffsetDateTime};

pub const DATETIME_FMT: &[FormatItem] = format_description!("[year]-[month]-[day] [hour]:[minute]");

pub type MultiLanguageContent = HashMap<String, String>;

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentType {
    Plain(String),
    MultiLanguage(MultiLanguageContent),
}

#[derive(Serialize, Deserialize)]
pub struct ContentSettings {
    send_date: String,
    content: ContentType,
    ignore_user_timezones: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    timezone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    campaign: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    filter: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateMessage {
    /// API access token from Pushwoosh Control Panel
    auth: String,
    /// Pushwoosh application code
    application: String,
    /// Push notifications properties
    notifications: Vec<ContentSettings>,
}

pub struct ContentSettingsBuilder {
    send_date: String,
    content: Option<ContentType>,
    ignore_user_timezones: bool,
    timezone: Option<String>,
    campaign: Option<String>,
    filter: Option<String>,
}

impl Default for ContentSettingsBuilder {
    fn default() -> Self {
        Self {
            send_date: "now".to_string(),
            content: None,
            ignore_user_timezones: false,
            timezone: None,
            campaign: None,
            filter: None,
        }
    }
}

impl ContentSettingsBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_send_date(mut self, datetime: OffsetDateTime) -> Self {
        self.send_date = datetime
            .format(&DATETIME_FMT)
            .expect("could not format date");
        self
    }

    pub fn with_plain_content(mut self, content: String) -> Self {
        self.content = Some(ContentType::Plain(content));
        self
    }

    pub fn with_multi_content(mut self, content: MultiLanguageContent) -> Self {
        self.content = Some(ContentType::MultiLanguage(content));
        self
    }

    pub fn with_content(self, content: ContentType) -> Self {
        match content {
            ContentType::Plain(content) => self.with_plain_content(content),
            ContentType::MultiLanguage(content) => self.with_multi_content(content),
        }
    }

    pub fn with_ignore_user_timezones(mut self, ignore: bool) -> Self {
        self.ignore_user_timezones = ignore;
        self
    }

    pub fn with_timezone(mut self, timezone: Option<String>) -> Self {
        self.timezone = timezone;
        self
    }

    pub fn with_campaign(mut self, campaign: Option<String>) -> Self {
        self.campaign = campaign;
        self
    }

    pub fn with_filter(mut self, filter: Option<String>) -> Self {
        self.filter = filter;
        self
    }

    pub fn build(self) -> Result<ContentSettings, Report> {
        let content = self.content.ok_or_else(|| eyre!("missing field content"))?;

        Ok(ContentSettings {
            send_date: self.send_date,
            content,
            ignore_user_timezones: self.ignore_user_timezones,
            timezone: self.timezone,
            campaign: self.campaign,
            filter: self.filter,
        })
    }
}

#[derive(Default)]
pub struct CreateMessageBuilder {
    auth: Option<String>,

    application: Option<String>,

    notifications: Vec<ContentSettings>,
}

impl CreateMessageBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_auth(mut self, auth: String) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn with_application(mut self, application: String) -> Self {
        self.application = Some(application);
        self
    }

    pub fn add_content_settings(mut self, parameters: ContentSettings) -> Self {
        self.notifications.push(parameters);
        self
    }

    pub fn build(self) -> Result<CreateMessage, Report> {
        if self.notifications.is_empty() {
            bail!("empty content settings");
        }

        let auth = self.auth.ok_or_else(|| eyre!("missing field auth"))?;
        let application = self
            .application
            .ok_or_else(|| eyre!("missing field application"))?;

        Ok(CreateMessage {
            auth,
            application,
            notifications: self.notifications,
        })
    }
}
