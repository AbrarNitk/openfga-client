use openfga_client::OpenFGAClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a new OpenFGA client
    // Replace with your actual OpenFGA server endpoint
    let mut client = OpenFGAClient::new("http://localhost:8080".to_string()).await?;

    let store_id = "01HXXX-STORE-ID-XXXX".to_string(); // Replace with actual store ID

    // Example 1: Write a relationship tuple
    println!("Writing a relationship tuple...");
    let write_request = OpenFGAClient::create_write_request(
        store_id.clone(),
        "document".to_string(),
        "doc1".to_string(),
        "reader".to_string(),
        "user".to_string(),
        "alice".to_string(),
    );

    match client.write(write_request).await {
        Ok(_) => {
            println!("✓ Successfully wrote relationship: user:alice is reader of document:doc1")
        }
        Err(e) => println!("✗ Failed to write relationship: {}", e),
    }

    // Example 2: Check if a user has access
    println!("\nChecking if user has access...");
    let check_request = OpenFGAClient::create_check_request(
        store_id.clone(),
        "document:doc1".to_string(),
        "reader".to_string(),
        "user:alice".to_string(),
    );

    match client.check(check_request).await {
        Ok(response) => {
            let check_response = response.into_inner();
            if check_response.allowed {
                println!("✓ User alice has reader access to document:doc1");
            } else {
                println!("✗ User alice does NOT have reader access to document:doc1");
            }
        }
        Err(e) => println!("✗ Failed to check access: {}", e),
    }

    // Example 3: Check access for a different user
    println!("\nChecking access for a different user...");
    let check_request_bob = OpenFGAClient::create_check_request(
        store_id.clone(),
        "document:doc1".to_string(),
        "reader".to_string(),
        "user:bob".to_string(),
    );

    match client.check(check_request_bob).await {
        Ok(response) => {
            let check_response = response.into_inner();
            if check_response.allowed {
                println!("✓ User bob has reader access to document:doc1");
            } else {
                println!("✗ User bob does NOT have reader access to document:doc1");
            }
        }
        Err(e) => println!("✗ Failed to check access: {}", e),
    }

    Ok(())
}

/*
To run this example:
1. Make sure OpenFGA server is running on localhost:8080
2. Create a store and get the store ID
3. Update the store_id variable above
4. Run: cargo run --example openfga_client_example
*/
