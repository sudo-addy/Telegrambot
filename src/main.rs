use axum::{
    extract::State,
    response::Html,
    routing::get,
    Form, Router,
};
use mongodb::{
    bson::{doc, Document},
    options::ClientOptions,
    Client, Collection,
};
use serde::Deserialize;
use std::fs;

// Struct to match the login form fields
#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

// Struct to match the custom string saving form
#[derive(Deserialize)]
struct SaveData {
    username: String,
    custom_string: String,
}

#[tokio::main]
async fn main() {
    // --- Connect to MongoDB ---
    println!("Connecting to MongoDB...");
    let mut client_options = ClientOptions::parse("mongodb://127.0.0.1:27017")
        .await
        .expect("Failed to parse MongoDB connection URI");
    client_options.app_name = Some("TelegramBotBackend".to_string());
    
    let client = Client::with_options(client_options)
        .expect("Failed to initialize MongoDB client");

    let db = client.database("bot_manager");
    let users_collection = db.collection::<Document>("users");

    // Seed/train the database with some default users if it's empty
    seed_database(&users_collection).await;

    // --- Setup Routes ---
    let app = Router::new()
        .route("/", get(home))
        .route("/login", get(show_login_form).post(handle_login))
        .route("/save", axum::routing::post(handle_save)) // New route to save strings
        .with_state(users_collection);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000")
        .await
        .unwrap();
    println!("Rust & MongoDB server is running on http://127.0.0.1:5000");

    axum::serve(listener, app).await.unwrap();
}

// Function to seed/train the database with some sample users if it's empty
async fn seed_database(collection: &Collection<Document>) {
    match collection.count_documents(doc! {}).await {
        Ok(count) => {
            if count == 0 {
                println!("Database is empty. Seeding sample users...");
                let sample_users = vec![
                    doc! { "username": "admin", "password": "password123", "notes": [] },
                    doc! { "username": "user1", "password": "mysecretpass", "notes": [] },
                    doc! { "username": "john", "password": "sdOpenSesame", "notes": [] },
                ];
                let _ = collection.insert_many(sample_users).await;
            }
        }
        Err(e) => println!("MongoDB Connection/Seeding Error: {}", e),
    }
}

// Handler for the home page (GET /)
async fn home() -> Html<&'static str> {
    Html(
        "<h1>Welcome to the Rust & MongoDB Bot Control Panel</h1>\
         <p>Please <a href='/login'>Login here</a> to access the dashboard.</p>"
    )
}

// Handler to show the login form (GET /login)
async fn show_login_form() -> Html<String> {
    match fs::read_to_string("login.html") {
        Ok(html_content) => Html(html_content),
        Err(_) => Html("<h1>Error: login.html file was not found!</h1>".to_string()),
    }
}

// Helper function to render the user's dashboard with their notes/strings
fn render_dashboard(username: &str, notes: &[String]) -> Html<String> {
    // Build the list of notes as HTML bullet points
    let mut notes_html = String::new();
    if notes.is_empty() {
        notes_html.push_str("<li>No notes saved yet.</li>");
    } else {
        for note in notes {
            notes_html.push_str(&format!("<li>{}</li>", note));
        }
    }

    Html(format!(
        r#"
        <!DOCTYPE html>
        <html>
        <head>
            <title>Dashboard</title>
            <style>
                body {{ font-family: sans-serif; margin: 40px; background-color: #f4f6f9; }}
                .container {{ background: white; padding: 30px; border-radius: 8px; box-shadow: 0 4px 6px rgba(0,0,0,0.1); max-width: 600px; margin: auto; }}
                h1 {{ color: #333; }}
                ul {{ background: #eee; padding: 20px; border-radius: 4px; list-style-type: square; }}
                input[type="text"] {{ width: 70%; padding: 10px; margin-right: 10px; border: 1px solid #ccc; border-radius: 4px; }}
                button {{ padding: 10px 20px; background: #007bff; color: white; border: none; border-radius: 4px; cursor: pointer; }}
                button:hover {{ background: #0056b3; }}
            </style>
        </head>
        <body>
            <div class="container">
                <h1>Welcome back, {}!</h1>
                <h3>Your Saved Strings:</h3>
                <ul>
                    {}
                </ul>

                <h3>Save a new String to Database:</h3>
                <form action="/save" method="POST">
                    <!-- Hidden field to remember who is saving the string -->
                    <input type="hidden" name="username" value="{}">
                    <input type="text" name="custom_string" placeholder="Type a message or note..." required>
                    <button type="submit">Save String</button>
                </form>
                <br>
                <a href="/login">Logout</a>
            </div>
        </body>
        </html>
        "#,
        username, notes_html, username
    ))
}

// Handler to process login (POST /login)
async fn handle_login(
    State(collection): State<Collection<Document>>,
    Form(form): Form<LoginData>,
) -> Html<String> {
    let query = doc! { "username": &form.username };

    match collection.find_one(query).await {
        Ok(Some(user_doc)) => {
            if let Ok(correct_password) = user_doc.get_str("password") {
                if form.password == correct_password {
                    // Extract existing notes array from the document
                    let mut notes = Vec::new();
                    if let Ok(notes_array) = user_doc.get_array("notes") {
                        for bson_val in notes_array {
                            if let Some(note_str) = bson_val.as_str() {
                                notes.push(note_str.to_string());
                            }
                        }
                    }
                    // Render and return the dashboard page
                    render_dashboard(&form.username, &notes)
                } else {
                    Html("<h1>Error: Incorrect password.</h1><a href='/login'>Try Again</a>".to_string())
                }
            } else {
                Html("<h1>Error: Could not read password from database.</h1>".to_string())
            }
        }
        Ok(None) => Html("<h1>Error: Username not found.</h1><a href='/login'>Try Again</a>".to_string()),
        Err(e) => Html(format!("<h1>Error: Database connection failure: {}</h1>", e)),
    }
}

// Handler to process saving a new string (POST /save)
async fn handle_save(
    State(collection): State<Collection<Document>>,
    Form(form): Form<SaveData>,
) -> Html<String> {
    // 1. Update the document in MongoDB by appending the new string to the 'notes' array
    let filter = doc! { "username": &form.username };
    let update = doc! { "$push": { "notes": &form.custom_string } };

    match collection.update_one(filter, update).await {
        Ok(_) => {
            // 2. Fetch the updated user document to display the fresh list of notes
            match collection.find_one(doc! { "username": &form.username }).await {
                Ok(Some(user_doc)) => {
                    let mut notes = Vec::new();
                    if let Ok(notes_array) = user_doc.get_array("notes") {
                        for bson_val in notes_array {
                            if let Some(note_str) = bson_val.as_str() {
                                notes.push(note_str.to_string());
                            }
                        }
                    }
                    // Show the dashboard again with the newly updated list
                    render_dashboard(&form.username, &notes)
                }
                _ => Html("<h1>Error reloading dashboard.</h1>".to_string()),
            }
        }
        Err(e) => Html(format!("<h1>Error saving to database: {}</h1>", e)),
    }
}
