//! some code for reading and markdowning html files

use html_to_markdown_rs;

pub fn get_markdown_from_web(url: &str) -> String {
    let html_text = get_html_from_web(url);
    if html_text != "" {
        let try_markdown_text = html_to_markdown_rs::convert(&html_text, None);
        match try_markdown_text {
            Ok(md_text) => md_text,
            Err(e) => {
                log::warn!("Could not convert markdown for {url}, because {e}");
                String::from("")
            }
        }
    } else {
        // Simply return empty text
        html_text
    }
}

fn get_html_from_web(url: &str) -> String {
    let try_response = reqwest::blocking::get(url);

    match try_response {
        Ok(response) => {
            let try_response_text = response.text();
            match try_response_text {
                Ok(response_text) => response_text, //return
                Err(e) => {
                    log::warn!("Could not convert html to text for {url}, because {e}");
                    String::from("")
                }
            }
        }
        Err(e) => {
            log::warn!("Did not get website {url}, because {e}");
            String::from("")
        }
    }
}
