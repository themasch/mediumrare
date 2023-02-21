use maud::{html, PreEscaped, DOCTYPE};

pub fn html_page(title: &str, body: &str) -> String {
    let css = r#" body { background-color: #111; color: #eee; font-family: sans-serif; font-size: 130%; }
                    article { width: 60rem; margin: auto }
                    img { max-width: 100% }
                    pre { background-color: #000; padding: 1rem; border-radius: .5rem; overflow-y: scroll; }
                    code { background-color: #000; padding: .25rem; border-radius: .5rem; }
                    blockquote { background-color: #333; margin: 0; padding: 1rem;  padding-left: 2rem; border-left: 5px solid gray; }
                    a { color: cornflowerblue }
                    .post-head {  background-color: #333; margin: 0; padding: 1rem; font-size: 80%; } "#;
    html! {
        (DOCTYPE)
        html {
            head {
                style { (css) }
                title { (title) }
            }
            body {
                (PreEscaped(body))
            }
        }
    }
    .into_string()
}

pub fn home() -> String {
    let js = r#"
        document.addEventListener('submit', (evt) => {
            evt.preventDefault();
            const url = document.getElementById("url_input").value;
            const matches = url.match(/-([a-f0-9]+)$/);
            window.location = matches[1];
            return false;
        });
    "#;
    html_page(
        "mediumrare",
        &html! {
            h1 { ("WHAT?") }
            form {
                input #url_input type="text";
            }
            script { (PreEscaped(js)) }
        }
        .into_string(),
    )
}
