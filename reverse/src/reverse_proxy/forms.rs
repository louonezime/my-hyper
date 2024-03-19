use hyper::{Body, Client, Response};
use hyper_tls::HttpsConnector;

use scraper::{Html, Selector};
use std::sync::Arc;
use toml::Value;

use super::config::setup_form;

mod post;

#[derive(Debug, PartialEq)]
pub struct Input {
    id: String,
    types: String,
    pub name: String,
    value: String,
}

#[derive(Debug, PartialEq)]
pub struct Form {
    id: String,
    pub action: String,
    method: String,
    pub inputs: Vec<Input>,
}

#[derive(Debug)]
pub struct ServerFormElements {
    pub forms: Vec<Form>,
}

impl Input {
    fn new(id: &str, types: &str, name: &str, value: &str) -> Self {
        let mut type_def = String::from("text");
        if types.is_empty() {
            type_def = types.to_string();
        }

        Input {
            id: id.to_string(),
            types: type_def,
            name: name.to_string(),
            value: value.to_string(),
        }
    }
}

impl Form {
    fn new(id: &str, action: &str, method: &str) -> Self {
        Form {
            id: id.to_string(),
            action: action.to_string(),
            method: method.to_string(),
            inputs: Vec::new(),
        }
    }

    fn add_input(&mut self, input: Input) {
        self.inputs.push(input);
    }

    fn _remove_input(&mut self, input: Input) {
        if let Some(pos) = self.inputs.iter().position(|x| *x == input) {
            self.inputs.remove(pos);
        }
    }
}

impl ServerFormElements {
    fn new() -> Self {
        ServerFormElements { forms: Vec::new() }
    }

    fn add_form(&mut self, form: Form) {
        self.forms.push(form);
    }

    fn _remove_form(&mut self, form: Form) {
        if let Some(pos) = self.forms.iter().position(|x| *x == form) {
            self.forms.remove(pos);
        }
    }

    fn is_empty(&mut self) -> bool {
        if self.forms.is_empty() {
            return true;
        }
        false
    }

    fn _print(&self) {
        for (i, form) in self.forms.iter().enumerate() {
            println!("Form {}:", i + 1);
            println!("  ID: {}", form.id);
            println!("  Action: {}", form.action);
            if !form.inputs.is_empty() {
                println!("  Elements:");
                for (i, input) in form.inputs.iter().enumerate() {
                    println!("    Input #{}:", i + 1);
                    println!("      ID: {}", input.id);
                    println!("      Type: {}", input.types);
                    println!("      Name: {} + Value: {}", input.name, input.value);
                }
            }
            println!();
        }
    }
}

fn input_elements(document: Html, input_selector: Selector, form: &mut Form) {
    for element in document.select(&input_selector) {
        let id = element.value().attr("id").unwrap_or("").to_string();
        let type_elem = element.value().attr("type").unwrap_or("text").to_string();
        let name = element.value().attr("name").unwrap_or("").to_string();
        let value = element.value().attr("value").unwrap_or("").to_string();

        if filter_inputs(type_elem.clone(), name.clone()).is_some() {
            let input = Input::new(&id, &type_elem, &name, &value);

            form.add_input(input);
        }
    }
}

fn form_base_elements(
    html: &str,
    form_select: Selector,
    input_select: Selector,
) -> ServerFormElements {
    let document = Html::parse_document(html);
    let mut form_elems = ServerFormElements::new();

    for element in document.select(&form_select) {
        let id = element.value().attr("id").unwrap_or("").to_string();
        let action = element.value().attr("action").unwrap_or("").to_string();
        let method = element.value().attr("method").unwrap_or("get").to_string();

        if method.to_lowercase() != "get" {
            let mut form = Form::new(&id, &action, &method);

            input_elements(document.clone(), input_select.clone(), &mut form);
            ServerFormElements::add_form(&mut form_elems, form)
        }
    }
    form_elems
}

fn filter_inputs(input: String, name: String) -> Option<String> {
    if input == "text"
        || input == "email"
        || input == "password"
        || input == "submit"
        || input == "hidden"
    {
        return Some(input.clone());
    }
    if name.is_empty() {
        return None;
    }
    None
}

pub fn extract_form_elements(html: &str) -> Option<ServerFormElements> {
    let mut forms = ServerFormElements::new();

    match Selector::parse("form") {
        Ok(form_selector) => {
            match Selector::parse("input") {
                Ok(input_selector) => {
                    forms = form_base_elements(html, form_selector, input_selector)
                }
                Err(err) => eprintln!("{}", err),
            };
        }
        Err(err) => eprintln!("{}", err),
    };

    if !ServerFormElements::is_empty(&mut forms) {
        return Some(forms);
    }
    None
}

fn match_credential(target_url: String, config: Value, form: &Form) -> Vec<(String, String)> {
    let mut post_data: Vec<(String, String)> = Vec::new();

    if let Some(form_auth) = setup_form(config) {
        for (key, value) in form_auth {
            if key == target_url {
                post_data = value;
            }
        }
    }
    if !form.inputs.is_empty() {
        // Filled in inputs will be sent as they already were
        for input in form.inputs.iter() {
            if !input.value.is_empty() {
                post_data.push((input.name.clone(), input.value.clone()));
            }
        }
    }
    post_data
}

pub async fn handle_forms(
    body: String,
    target_url: &str,
    client: Arc<Client<HttpsConnector<hyper::client::HttpConnector>>>,
    session: &str,
    config: Value,
) -> Result<Response<Body>, ()> {
    if let Some(forms) = extract_form_elements(&body) {
        println!("Form element identified!");
        if let Some(form) = forms.forms.iter().next() {
            let form_cred: Vec<(String, String)> =
                match_credential(target_url.to_string(), config, form);

            let mut action = form.action.clone();
            if action.starts_with('/') {
                action = format!("{}{}", target_url, form.action.clone())
            }

            return post::handle_post(action, form_cred, client, session).await;
        }
    }
    Err(())
}
