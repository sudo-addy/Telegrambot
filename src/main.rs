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

// Define a struct to match the login form fields.
// The fields `username` and `password` match the HTML form input 'name' attributes.
#[derive(Deserialize)]
struct LoginData {
    username: String,
    password: String,
}

#[tokio::main]
async fn main() {
    // --- STEP 1: Connect to MongoDB ---
    // We assume MongoDB is running locally at the default address: mongodb://localhost:27017
    println!("Connecting to MongoDB...");
    let mut client_options = ClientOptions::parse("mongodb://127.0.0.1:27017")
        .await
        .expect("Failed to parse MongoDB connection URI");
    
    // Set application name for logging in MongoDB
    client_options.app_name = Some("TelegramBotBackend".to_string());
    
    let client = Client::with_options(client_options)
        .expect("Failed to initialize MongoDB client");

    // Access the database named "bot_manager" and the collection named "users"
    let db = client.database("bot_manager");
    let users_collection = db.collection::<Document>("users");

    // --- STEP 2: Seed Sample Data (Trained Data Set) ---
    // If the database has no users, we will insert some default members.
    seed_database(&users_collection).await;

    // --- STEP 3: Setup Axum Router with Shared MongoDB State ---
    // We pass `users_collection` to `with_state` so all our route handlers can access it.
    let app = Router::new()
        .route("/", get(home))
        .route("/login", get(show_login_form).post(handle_login))
        .with_state(users_collection);

    // Run the server locally on port 5000
    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000")
        .await
        .unwrap();
    println!("Rust & MongoDB server is running on http://127.0.0.1:5000");

    axum::serve(listener, app).await.unwrap();
}

// Function to seed/train the database with some sample users if it's empty
async fn seed_database(collection: &Collection<Document>) {
    // Count the current documents in the 'users' collection
    match collection.count_documents(doc! {}).await {
        Ok(count) => {
            if count == 0 {
                println!("Database is empty. Seeding sample users...");
                
                // Create a list of sample users (Trained Data Set)
                let sample_users = vec![
                    doc! { "username": "admin", "password": "password123" },
                    doc! { "username": "user1", "password": "mysecretpass" },
                    doc! { "username": "john", "password": "OpenSesame" },
                ];
                
                // Insert them into MongoDB
                match collection.insert_many(sample_users).await {
                    Ok(_) => println!("Successfully seeded database with 3 sample users."),
                    Err(e) => println!("Failed to seed database: {}", e),
                }
            } else {
                println!("Database already has {} user(s). Skipping seeding.", count);
            }
        }
        Err(e) => {
            println!("Could not connect to MongoDB server or count documents: {}.", e);
            println!("IMPORTANT: Please ensure 'mongod' is running on your system! (Run: sudo systemctl start mongod)");
        }
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

// Handler to process login (POST /login)
// We extract the MongoDB collection using `State` and the submitted form data using `Form`.
async fn handle_login(
    State(collection): State<Collection<Document>>,
    Form(form): Form<LoginData>,
) -> Html<String> {
    // Build a query to find a user document matching the submitted username
    let query = doc! { "username": &form.username };

    // Query MongoDB
    match collection.find_one(query).await {
        Ok(Some(user_doc)) => {
            // Retrieve the password from the retrieved document
            if let Ok(correct_password) = user_doc.get_str("password") {
                // Compare passwords
                if form.password == correct_password {
                    Html(format!(
                        "<h1>Success! Welcome back, {} (Validated by MongoDB)!</h1>",
                        form.username
                    ))
                } else {
                    Html("<h1>Error: Incorrect password.</h1><a href='/login'>Try Again</a>".to_string())
                }
            } else {
                Html("<h1>Error: Could not read password from database.</h1>".to_string())
            }
        }
        Ok(None) => {
            // Username was not found in the database
            Html("<h1>Error: Username not found.</h1><a href='/login'>Try Again</a>".to_string())
        }
        Err(e) => {
            // Database query failed (e.g. if database is offline)
            Html(format!("<h1>Error: Database communication failure: {}</h1>", e))
        }
    }
}
