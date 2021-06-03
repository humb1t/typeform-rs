//! This crate wraps Typeform's REST API.
//! Main entry-point is [`Typeform`] - once build it can be used to receive responses.

#![warn(
    anonymous_parameters,
    missing_copy_implementations,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    private_doc_tests,
    trivial_casts,
    trivial_numeric_casts,
    unused,
    future_incompatible,
    nonstandard_style,
    unsafe_code,
    unused_import_braces,
    unused_results,
    variant_size_differences
)]

use isahc::{prelude::*, Request};
use serde::Deserialize;

const DEFAULT_TYPEFORM_URL: &str = "https://api.typeform.com";
const GET_FORM_RESPONSES_PATH: &str = "/forms/{form_id}/responses";

/// Main entry point to work with.
#[derive(Debug)]
pub struct Typeform {
    url: String,
    form_id: String,
    token: String,
}

impl Typeform {
    /// Default [`Typeform`] constructor.
    pub fn new(form_id: &str, token: &str) -> Typeform {
        Typeform {
            url: DEFAULT_TYPEFORM_URL.to_string(),
            form_id: form_id.to_string(),
            token: token.to_owned(),
        }
    }

    /// Retrieve all [`Responses`].
    pub fn responses(&self) -> Result<Responses, String> {
        Request::get(format!(
            "{}{}",
            self.url,
            GET_FORM_RESPONSES_PATH.replace("{form_id}", &self.form_id),
        ))
        .header("Authorization", format!("Bearer {}", &self.token))
        .body(())
        .map_err(|error| format!("Failed to build a request: {}", error))?
        .send()
        .map_err(|error| format!("Failed to send get request: {}", error))?
        .json()
        .map_err(|error| format!("Failed to deserialize a response: {}", error))
    }

    /// Retrieve all [`Responses`] which goes after response with [`token`].
    pub fn responses_after(&self, token: &str) -> Result<Responses, String> {
        Request::get(format!(
            "{}{}?after={}&page_size=1",
            self.url,
            GET_FORM_RESPONSES_PATH.replace("{form_id}", &self.form_id),
            token,
        ))
        .header("Authorization", format!("Bearer {}", &self.token))
        .body(())
        .map_err(|error| format!("Failed to build a request: {}", error))?
        .send()
        .map_err(|error| format!("Failed to send get request: {}", error))?
        .json()
        .map_err(|error| format!("Failed to deserialize a response: {}", error))
    }
}

/// Paged list of [`Response`]s.
#[derive(Clone, Debug, Deserialize)]
pub struct Responses {
    /// Total number of items in the retrieved collection.
    total_items: Option<u16>,
    /// Number of pages.
    page_count: Option<u8>,
    /// Array of [Responses](Response).
    pub items: Vec<Response>,
}

/// Unique form's response.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Response {
    token: String,
    /// Unique ID for the response. Note that response_id values are unique per form but are not unique globally.
    response_id: Option<String>,
    /// Time of the form landing. In ISO 8601 format, UTC time, to the second, with T as a delimiter between the date and time.
    landed_at: String,
    /// Time that the form response was submitted. In ISO 8601 format, UTC time, to the second, with T as a delimiter between the date and time.
    submitted_at: String,
    /// Metadata about a client's HTTP request.
    metadata: Metadata,
    /// Subset of a complete form definition to be included with a submission.
    definition: Option<Definition>,
    answers: Option<Answers>,
    calculated: Calculated,
}

/// Metadata about a client's HTTP request.
#[derive(Clone, Debug, Deserialize)]
struct Metadata {
    user_agent: String,
    /// Derived from user agent.
    platform: Option<String>,
    referer: String,
    /// IP of the client.
    network_id: String,
}

/// Subset of a complete form definition to be included with a submission.
#[derive(Clone, Debug, Deserialize)]
struct Definition {
    fields: Fields,
}

type Fields = Vec<Field>;

#[derive(Clone, Debug, Deserialize)]
struct Field {
    id: String,
    _type: String,
    title: String,
    description: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Answers(Vec<Answer>);

#[derive(Clone, Debug, Deserialize)]
struct Answer {
    field: AnswerField,
    /// The answer-fields's type.
    #[serde(rename = "type")]
    _type: AnswerType,
    /// Represents single choice answers for dropdown-like fields.
    choice: Option<Choice>,
    /// Represents multiple choice answers.
    choices: Option<Choices>,
    date: Option<String>,
    email: Option<String>,
    file_url: Option<String>,
    number: Option<i32>,
    boolean: Option<bool>,
    text: Option<String>,
    url: Option<String>,
    payment: Option<Payment>,
    phone_number: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct AnswerField {
    /// The unique id of the form field the answer refers to.
    id: String,
    /// The field's type in the original form.
    #[serde(rename = "type")]
    _type: String,
    /// The reference for the question the answer relates to. Use the ref value to match answers with questions. The Responses payload only includes ref for the fields where you specified them when you created the form.
    #[serde(rename = "ref")]
    _ref: String,
    /// The form field's title which the answer is related to.
    title: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum AnswerType {
    Choice,
    Choices,
    Date,
    Email,
    Url,
    FileUrl,
    Number,
    Boolean,
    Text,
    Payment,
    PhoneNumber,
}

#[derive(Clone, Debug, Deserialize)]
struct Choice {
    label: String,
    other: Option<String>,
}

type Labels = Vec<String>;

#[derive(Clone, Debug, Deserialize)]
struct Choices {
    labels: Labels,
    other: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
struct Payment {
    amount: String,
    last4: String,
    name: String,
}

#[derive(Clone, Debug, Deserialize)]
struct Calculated {
    score: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn parse_valid_responses_from_json_should_pass() {
        let file = File::open("tests/typeform_responses.json").expect("Failed to open a file.");
        let reader = BufReader::new(file);
        let _responses: Responses =
            serde_json::from_reader(reader).expect("Failed to build responses from reader.");
    }

    #[test]
    fn parse_valid_responses2_from_json_should_pass() {
        let file = File::open("tests/typeform_responses2.json").expect("Failed to open a file.");
        let reader = BufReader::new(file);
        let _responses: Responses =
            serde_json::from_reader(reader).expect("Failed to build responses from reader.");
    }

    #[test]
    fn parse_valid_responses3_from_json_should_pass() {
        let file = File::open("tests/typeform_responses3.json").expect("Failed to open a file.");
        let reader = BufReader::new(file);
        let _responses: Responses =
            serde_json::from_reader(reader).expect("Failed to build responses from reader.");
    }
}
