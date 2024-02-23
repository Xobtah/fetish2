use dialoguer::{theme::ColorfulTheme, Input};

pub fn ask_user(prompt: &str) -> String {
    Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .unwrap()
        .trim()
        .into()
}
