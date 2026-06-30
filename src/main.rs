use axum::{
    response::Html,
    routing::get,
    Form, Router,
};
use serde::Deserialize;
use std::fs;

// Define a struct to match the login form fields.
// The #[derive(Deserialize)] macro allows Serde to automatically convert
// the form data from the POST request into this Rust struct.
// The fields `username` and `password` match the HTML form input 'name' attributes.
#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

// Tokio's main macro to start the asynchronous runtime.
#[tokio::main]
async fn main() {
    // Build our application with routes:
    // 1. GET "/" -> Serves a welcome message.
    // 2. GET "/login" -> Serves the login.html page.
    // 3. POST "/login" -> Processes the submitted username and password.
    let app = Router::new()
        .route("/", get(home))
        .route("/login", get(show_login_form).post(handle_login));

    // Define the address to run our server (localhost, port 5000)
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000")
        .await
        .unwrap();
    println!("Rust server is running on http://127.0.0.1:5000");

    // Start serving requests
    axum::serve(listener, app).await.unwrap();
}

// Handler for the home page. Returns simple HTML.
async fn home() -> Html<&'static str> {
    Html(
        "<h1>Welcome to the Rust Bot Control Panel</h1>\
         <p>Please <a href='/login'>Login here</a> to access the dashboard.</p>"
    )
}

// Handler to show the login form (GET /login).
// It reads the `login.html` file from the disk and sends it to the browser.
async fn show_login_form() -> Html<String> {
    // Try reading the file; if it fails, return an error message
    match fs::read_to_string("login.html") {
        Ok(html_content) => Html(html_content),
        Err(_) => Html("<h1>Error: login.html file was not found!</h1>".to_string()),
    }
}

// Handler to process the login form submission (POST /login).
// It extracts the form data into our `LoginData` struct.
async fn handle_login(Form(form): Form<LoginData>) -> Html<String> {
    // Simple mock credentials for testing
    let correct_username = "admin";
    let correct_password = "password123";

    // Validate the credentials
    if form.username == correct_username && form.password == correct_password {
        Html(format!(
            "<h1>Success! Welcome back (Rust Edition), {}!</h1>",
            form.username
        ))
    } else {
        Html(
            "<h1>Error: Incorrect username or password.</h1>\
             <a href='/login'>Try Again</a>"
                .to_string(),
        )
    }
}
