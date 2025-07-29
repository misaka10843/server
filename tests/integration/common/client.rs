use std::sync::Arc;

use axum_test::{TestResponse, TestServer};
use serde::Serialize;

use super::TestError;

/// Test client for making HTTP requests to the test server
pub struct TestClient {
    server: Arc<TestServer>,
    auth_token: Option<String>,
}

impl TestClient {
    /// Create a new test client
    pub fn new(server: Arc<TestServer>) -> Self {
        Self {
            server,
            auth_token: None,
        }
    }

    /// Set authentication token for subsequent requests
    pub fn with_auth_token(&mut self, token: String) {
        self.auth_token = Some(token);
    }

    /// Clear authentication token
    pub fn clear_auth(&mut self) {
        self.auth_token = None;
    }

    /// Make a GET request
    pub async fn get(&self, path: &str) -> TestResponse {
        let mut request = self.server.get(path);

        if let Some(token) = &self.auth_token {
            request = request
                .add_header("Authorization", format!("Bearer {}", token));
        }

        request.await
    }

    /// Make a POST request with JSON body
    pub async fn post_json<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<TestResponse, TestError> {
        let mut request = self
            .server
            .post(path)
            .content_type("application/json")
            .json(body);

        if let Some(token) = &self.auth_token {
            request = request
                .add_header("Authorization", format!("Bearer {}", token));
        }

        Ok(request.await)
    }

    /// Make a POST request with form data
    pub async fn post_form(
        &self,
        path: &str,
        form_data: &[(&str, &str)],
    ) -> TestResponse {
        let mut request = self
            .server
            .post(path)
            .content_type("application/x-www-form-urlencoded");

        // Build form data string
        let form_string = form_data
            .iter()
            .map(|(key, value)| format!("{}={}", key, value))
            .collect::<Vec<_>>()
            .join("&");

        request = request.text(form_string);

        if let Some(token) = &self.auth_token {
            request = request
                .add_header("Authorization", format!("Bearer {}", token));
        }

        request.await
    }

    /// Make a PUT request with JSON body
    pub async fn put_json<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<TestResponse, TestError> {
        let json_body = serde_json::to_string(body)?;

        let mut request = self
            .server
            .put(path)
            .content_type("application/json")
            .text(json_body);

        if let Some(token) = &self.auth_token {
            request = request
                .add_header("Authorization", format!("Bearer {}", token));
        }

        Ok(request.await)
    }

    /// Make a DELETE request
    pub async fn delete(&self, path: &str) -> TestResponse {
        let mut request = self.server.delete(path);

        if let Some(token) = &self.auth_token {
            request = request
                .add_header("Authorization", format!("Bearer {}", token));
        }

        request.await
    }

    /// Make a PATCH request with JSON body
    pub async fn patch_json<T: Serialize>(
        &self,
        path: &str,
        body: &T,
    ) -> Result<TestResponse, TestError> {
        let json_body = serde_json::to_string(body)?;

        let mut request = self
            .server
            .patch(path)
            .content_type("application/json")
            .text(json_body);

        if let Some(token) = &self.auth_token {
            request = request
                .add_header("Authorization", format!("Bearer {}", token));
        }

        Ok(request.await)
    }

    /// Get the underlying test server
    pub fn server(&self) -> &TestServer {
        &self.server
    }
}

/// Helper methods for authentication
impl TestClient {
    /// Sign up a new user and return the response
    pub async fn sign_up(
        &self,
        username: &str,
        password: &str,
    ) -> Result<TestResponse, TestError> {
        let signup_data = serde_json::json!({
            "name": username,
            "password": password
        });

        self.post_json("/sign_up", &signup_data).await
    }

    /// Sign in a user and automatically set the auth token
    pub async fn sign_in(
        &mut self,
        username: &str,
        password: &str,
    ) -> Result<TestResponse, TestError> {
        let signin_data = serde_json::json!({
            "name": username,
            "password": password
        });

        let response = self.post_json("/sign_in", &signin_data).await?;

        // If sign in was successful, extract and store the auth token
        if response.status_code().is_success() {
            // Note: The actual token extraction logic will depend on your authentication implementation
            // This is a placeholder - you'll need to adjust based on your auth response format
            let auth_response: serde_json::Value = response.json();
            if let Some(token) =
                auth_response.get("token").and_then(|t| t.as_str())
            {
                self.with_auth_token(token.to_string());
            }
        }

        Ok(response)
    }

    /// Sign out the current user
    pub async fn sign_out(&mut self) -> TestResponse {
        let response = self
            .post_json("/sign_out", &serde_json::json!({}))
            .await
            .unwrap_or_else(|_| panic!("Failed to sign out"));

        self.clear_auth();
        response
    }
}
