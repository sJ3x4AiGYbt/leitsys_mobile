use std::sync::LazyLock;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde::{Deserialize, Serialize};

const API_BASE_URL: &str = match option_env!("API_BASE_URL") {
    Some(url) => url,
    None => "http://localhost:3000",
};

/// Shared client so the refresh-token cookie set by `/auth/login` is kept
/// across requests. Native targets store cookies in this jar directly,
/// unlike the web build which relies on the browser's own cookie store via
/// `fetch_credentials_include()` — a fresh `reqwest::Client` per request
/// would otherwise start with an empty jar every time and drop the cookie.
static CLIENT: LazyLock<reqwest::Client> = LazyLock::new(|| {
    reqwest::Client::builder()
        .cookie_store(true)
        .build()
        .expect("failed to build the HTTP client")
});

#[derive(Serialize)]
struct LoginRequest<'a> {
    username: &'a str,
    pswd: &'a str,
}

#[derive(Deserialize)]
struct LoginResponse {
    access_token: String,
}

#[derive(Serialize)]
struct CreateUser<'a> {
    username: &'a str,
    email: &'a str,
    pswd: &'a str,
}

#[derive(Serialize)]
struct UpdateUserRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    username: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    email: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pswd: Option<&'a str>,
}

#[derive(Serialize)]
struct VerifyEmailRequest<'a> {
    token: &'a str,
}

#[derive(Serialize)]
struct ResendVerificationRequest<'a> {
    email: &'a str,
}

#[derive(Serialize)]
struct ForgotPasswordRequest<'a> {
    email: &'a str,
}

#[derive(Serialize)]
struct ResetPasswordRequest<'a> {
    token: &'a str,
    pswd: &'a str,
}

/// The claims carried by the access token, decoded client-side purely for
/// display (the backend is the one actually verifying the signature on
/// every protected request).
#[derive(Debug, Clone, Deserialize)]
pub struct Claims {
    pub user_id: i64,
    pub username: String,
    pub is_admin: bool,
}

pub fn decode_claims(token: &str) -> Option<Claims> {
    let payload = token.split('.').nth(1)?;
    let bytes = URL_SAFE_NO_PAD.decode(payload).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
}

#[derive(Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    message: Option<String>,
}

async fn send<T: serde::de::DeserializeOwned>(
    request: reqwest::RequestBuilder,
) -> Result<ApiResponse<T>, String> {
    let response = request
        .send()
        .await
        .map_err(|_| "Unable to reach the server".to_string())?;

    response
        .json::<ApiResponse<T>>()
        .await
        .map_err(|_| "Invalid response from the server".to_string())
}

pub async fn login(username: &str, pswd: &str) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/auth/login"))
        .json(&LoginRequest { username, pswd });

    let parsed: ApiResponse<LoginResponse> = send(request).await?;

    if parsed.success {
        parsed
            .data
            .map(|d| d.access_token)
            .ok_or_else(|| "Invalid response from the server".to_string())
    } else {
        Err(parsed.message.unwrap_or_else(|| "Invalid credentials".to_string()))
    }
}

pub async fn register(username: &str, email: &str, pswd: &str) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/auth/register"))
        .json(&CreateUser { username, email, pswd });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Account created successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to create the account".to_string()))
    }
}

pub async fn verify_email(token: &str) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/auth/verify-email"))
        .json(&VerifyEmailRequest { token });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Email verified.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to verify this email".to_string()))
    }
}

pub async fn resend_verification(email: &str) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/auth/resend-verification"))
        .json(&ResendVerificationRequest { email });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| {
            "If this email is registered and not yet verified, a new verification link has been sent."
                .to_string()
        }))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to resend the verification email".to_string()))
    }
}

pub async fn forgot_password(email: &str) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/auth/forgot-password"))
        .json(&ForgotPasswordRequest { email });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed
            .message
            .unwrap_or_else(|| "If this email is registered, a password reset link has been sent.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to request a password reset".to_string()))
    }
}

pub async fn reset_password(token: &str, pswd: &str) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/auth/reset-password"))
        .json(&ResetPasswordRequest { token, pswd });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Password reset successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to reset the password".to_string()))
    }
}

pub async fn refresh() -> Result<String, String> {
    let request = CLIENT.post(format!("{API_BASE_URL}/auth/refresh"));

    let parsed: ApiResponse<LoginResponse> = send(request).await?;

    if parsed.success {
        parsed
            .data
            .map(|d| d.access_token)
            .ok_or_else(|| "Invalid response from the server".to_string())
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to restore the session".to_string()))
    }
}

pub async fn logout() -> Result<(), String> {
    let request = CLIENT.post(format!("{API_BASE_URL}/auth/logout"));

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(())
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to log out".to_string()))
    }
}

pub async fn get_user(id: i64, token: &str) -> Result<User, String> {
    let request = CLIENT
        .get(format!("{API_BASE_URL}/users/{id}"))
        .bearer_auth(token);

    let parsed: ApiResponse<User> = send(request).await?;

    if parsed.success {
        parsed
            .data
            .ok_or_else(|| "Invalid response from the server".to_string())
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to fetch the user".to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Question {
    pub id: i64,
    pub title: String,
    pub answer: String,
    pub category_id: i64,
    pub current_step_id: i64,
    pub next_review_date: String,
    pub is_archived: bool,
}

/// Fetches the user's questions.
///
/// `archived` selects mastered (all steps completed) vs. active questions.
/// When `due_only` is set, restricts active questions to those due today or
/// overdue (backend's `status=todo` filter, which is `next_review_date <= now`).
pub async fn get_my_questions(user_id: i64, token: &str, archived: bool, due_only: bool) -> Result<Vec<Question>, String> {
    let mut url = format!("{API_BASE_URL}/questions/user/{user_id}?is_archived={archived}");
    if due_only {
        url.push_str("&status=todo");
    }

    let request = CLIENT.get(url).bearer_auth(token);

    let parsed: ApiResponse<Vec<Question>> = send(request).await?;

    if parsed.success {
        Ok(parsed.data.unwrap_or_default())
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to fetch questions".to_string()))
    }
}

#[derive(Serialize)]
struct CreateQuestionRequest<'a> {
    title: &'a str,
    answer: &'a str,
    category_id: Option<i64>,
}

pub async fn create_question(token: &str, title: &str, answer: &str, category_id: i64) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/questions"))
        .bearer_auth(token)
        .json(&CreateQuestionRequest { title, answer, category_id: Some(category_id) });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Question recorded successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to create the question".to_string()))
    }
}

#[derive(Serialize)]
struct UpdateQuestionRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    answer: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category_id: Option<i64>,
}

pub async fn update_question(
    id: i64,
    token: &str,
    title: Option<&str>,
    answer: Option<&str>,
    category_id: Option<i64>,
) -> Result<String, String> {
    let request = CLIENT
        .put(format!("{API_BASE_URL}/questions/{id}"))
        .bearer_auth(token)
        .json(&UpdateQuestionRequest { title, answer, category_id });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Question updated successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to update the question".to_string()))
    }
}

pub async fn delete_question(id: i64, token: &str) -> Result<String, String> {
    let request = CLIENT
        .delete(format!("{API_BASE_URL}/questions/{id}"))
        .bearer_auth(token);

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Question deleted successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to delete the question".to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Category {
    pub id: i64,
    pub title: String,
    pub color_code: String,
}

pub async fn get_my_categories(user_id: i64, token: &str) -> Result<Vec<Category>, String> {
    let request = CLIENT
        .get(format!("{API_BASE_URL}/categories/user/{user_id}"))
        .bearer_auth(token);

    let parsed: ApiResponse<Vec<Category>> = send(request).await?;

    if parsed.success {
        Ok(parsed.data.unwrap_or_default())
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to fetch categories".to_string()))
    }
}

#[derive(Serialize)]
struct CreateCategoryRequest<'a> {
    title: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    color_code: Option<&'a str>,
}

pub async fn create_category(token: &str, title: &str, color_code: Option<&str>) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/categories"))
        .bearer_auth(token)
        .json(&CreateCategoryRequest { title, color_code });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Category recorded successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to create the category".to_string()))
    }
}

#[derive(Serialize)]
struct UpdateCategoryRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color_code: Option<&'a str>,
}

pub async fn update_category(id: i64, token: &str, title: Option<&str>, color_code: Option<&str>) -> Result<String, String> {
    let request = CLIENT
        .put(format!("{API_BASE_URL}/categories/{id}"))
        .bearer_auth(token)
        .json(&UpdateCategoryRequest { title, color_code });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Category updated successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to update the category".to_string()))
    }
}

pub async fn delete_category(id: i64, token: &str) -> Result<String, String> {
    let request = CLIENT
        .delete(format!("{API_BASE_URL}/categories/{id}"))
        .bearer_auth(token);

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Category deleted successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to delete the category".to_string()))
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Step {
    pub id: i64,
    pub title: String,
    pub step_order: i64,
    pub spacing_days: i64,
    pub color_code: String,
}

pub async fn get_my_steps(user_id: i64, token: &str) -> Result<Vec<Step>, String> {
    let request = CLIENT
        .get(format!("{API_BASE_URL}/steps/user/{user_id}"))
        .bearer_auth(token);

    let parsed: ApiResponse<Vec<Step>> = send(request).await?;

    if parsed.success {
        Ok(parsed.data.unwrap_or_default())
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to fetch steps".to_string()))
    }
}

#[derive(Serialize)]
struct CreateStepRequest<'a> {
    title: &'a str,
    spacing_days: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    color_code: Option<&'a str>,
}

pub async fn create_step(token: &str, title: &str, spacing_days: i64, color_code: Option<&str>) -> Result<String, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/steps"))
        .bearer_auth(token)
        .json(&CreateStepRequest { title, spacing_days, color_code });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Step recorded successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to create the step".to_string()))
    }
}

#[derive(Serialize)]
struct UpdateStepRequest<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    step_order: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spacing_days: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    color_code: Option<&'a str>,
}

pub async fn update_step(
    id: i64,
    token: &str,
    title: Option<&str>,
    step_order: Option<i64>,
    spacing_days: Option<i64>,
    color_code: Option<&str>,
) -> Result<String, String> {
    let request = CLIENT
        .put(format!("{API_BASE_URL}/steps/{id}"))
        .bearer_auth(token)
        .json(&UpdateStepRequest { title, step_order, spacing_days, color_code });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Step updated successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to update the step".to_string()))
    }
}

pub async fn delete_step(id: i64, token: &str) -> Result<String, String> {
    let request = CLIENT
        .delete(format!("{API_BASE_URL}/steps/{id}"))
        .bearer_auth(token);

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Step deleted successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to delete the step".to_string()))
    }
}

#[derive(Serialize)]
struct CreateAnswerRequest<'a> {
    question_id: i64,
    user_response: &'a str,
    step: i64,
    is_correct: bool,
}

#[derive(Deserialize)]
struct CreatedAnswer {
    id: i64,
}

/// Records an answer and returns its id, so the review flow can immediately
/// follow up with `mark_answer_correct`/`mark_answer_incorrect`.
pub async fn create_answer(token: &str, question_id: i64, user_response: &str, step: i64, is_correct: bool) -> Result<i64, String> {
    let request = CLIENT
        .post(format!("{API_BASE_URL}/answers"))
        .bearer_auth(token)
        .json(&CreateAnswerRequest { question_id, user_response, step, is_correct });

    let parsed: ApiResponse<CreatedAnswer> = send(request).await?;

    if parsed.success {
        parsed
            .data
            .map(|d| d.id)
            .ok_or_else(|| "Invalid response from the server".to_string())
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to record the answer".to_string()))
    }
}

pub async fn mark_answer_correct(answer_id: i64, token: &str) -> Result<String, String> {
    let request = CLIENT
        .patch(format!("{API_BASE_URL}/answers/{answer_id}/correct"))
        .bearer_auth(token);

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Correct answer! Question moved to the next step.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to record the result".to_string()))
    }
}

pub async fn mark_answer_incorrect(answer_id: i64, token: &str) -> Result<String, String> {
    let request = CLIENT
        .patch(format!("{API_BASE_URL}/answers/{answer_id}/error"))
        .bearer_auth(token);

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Incorrect answer. Question reset to the first step.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to record the result".to_string()))
    }
}

pub async fn update_profile(id: i64, token: &str, username: &str, email: &str) -> Result<String, String> {
    let request = CLIENT
        .put(format!("{API_BASE_URL}/users/{id}"))
        .bearer_auth(token)
        .json(&UpdateUserRequest { username: Some(username), email: Some(email), pswd: None });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Profile updated successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to update the profile".to_string()))
    }
}

pub async fn change_password(id: i64, token: &str, pswd: &str) -> Result<String, String> {
    let request = CLIENT
        .put(format!("{API_BASE_URL}/users/{id}"))
        .bearer_auth(token)
        .json(&UpdateUserRequest { username: None, email: None, pswd: Some(pswd) });

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Password changed successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to change the password".to_string()))
    }
}

pub async fn delete_user(id: i64, token: &str) -> Result<String, String> {
    let request = CLIENT
        .delete(format!("{API_BASE_URL}/users/{id}"))
        .bearer_auth(token);

    let parsed: ApiResponse<()> = send(request).await?;

    if parsed.success {
        Ok(parsed.message.unwrap_or_else(|| "Account deleted successfully.".to_string()))
    } else {
        Err(parsed.message.unwrap_or_else(|| "Unable to delete the account".to_string()))
    }
}
