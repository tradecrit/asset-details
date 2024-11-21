use reqwest::RequestBuilder;

/// Send a request with exponential backoff, note this works for only reqwest requests that are
/// clone, which means no streams allowed. If you want to use streams you have to rebuild the
/// request every single time.
/// 
/// # Arguments
/// 
/// * `built_request` - The built request to send
/// 
/// # Returns
/// 
/// The response from the request
/// 
/// # Errors
/// 
/// * If the request fails, returns a reqwest::Error
pub async fn request(built_request: RequestBuilder) -> Result<reqwest::Response, reqwest::Error> {
    let mut backoff = 1;

    loop {
        let request_clone = built_request.try_clone().expect("Failed to clone request");
        let response = request_clone.send().await;

        match response {
            Ok(response) => {
                return Ok(response);
            },
            Err(error) => {
                tracing::error!("Failed to send request {:?}", error);
                tokio::time::sleep(std::time::Duration::from_secs(backoff)).await;
                backoff *= 2;
                
                if backoff > 64 {
                    return Err(error);
                }
            }
        }
    }
}
