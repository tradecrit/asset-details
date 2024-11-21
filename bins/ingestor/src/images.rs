use cloudflare_sdk::images::ImageResponse;
use polygon_sdk::models::Branding;
use utils::error::{Error, ErrorType};

#[derive(Debug)]
enum ImageType {
    Logo,
    Icon,
}

impl ImageType {
    fn to_string(&self) -> String {
        match self {
            ImageType::Logo => "logo".to_string(),
            ImageType::Icon => "icon".to_string(),
        }
    }
}

fn parse_image_format(image_url: &String) -> Option<String> {
    let image_format_regex = regex::Regex::new(r"\.[^.]*$").unwrap();
    let accepted_formats = vec![".png", ".jpg", ".jpeg", ".gif", ".svg", ".webp"];

    let image_format = image_format_regex.captures(&image_url).and_then(|captures| {
        captures.get(0).map(|format| format.as_str().to_string())
    });

    if image_format.is_none() {
        tracing::error!("Failed to extract image format from logo URL: {}", image_url);
        return None;
    }

    if !accepted_formats.contains(&image_format.as_ref().unwrap().as_str()) {
        tracing::error!("Unsupported image format: {}", image_format.unwrap());
        return None;
    }

    Some(image_format.unwrap())
}

async fn upload_image_to_cdn(
    cloudflare_client: &cloudflare_sdk::Client,
    source_image_url: String,
    image_id: String,
) -> Result<ImageResponse, Error> {
    let upload_result = cloudflare_client.upload_image_from_url(source_image_url, image_id, vec![]).await;

    match upload_result {
        Ok(response) => {
            tracing::debug!("Called upload image with response: {:?}", response);
            Ok(response)
        },
        Err(e) => {
            tracing::error!("Failed to upload image for {}", e);
            Err(Error::new(ErrorType::ThirdPartyError, format!("Failed to upload image for {}", e)))
        }
    }
}

async fn process_image_url(cloudflare_client: &cloudflare_sdk::Client, image_type: ImageType, url: String, ticker: String, polygon_api_key: String) -> Result<String, Error> {
    tracing::debug!("Processing image URL: {}", url);

    let image_format: Option<String> = parse_image_format(&url);

    tracing::debug!("Image format: {:?}", image_format);

    let image_id = image_format.and_then(|format| {
        Some(format!("{}/{}{}", image_type.to_string(), ticker, format))
    });

    tracing::debug!("Image ID: {:?}", image_id);

    let response = match image_id.clone() {
        Some(image_id) => {
            let signed_source_url = format!("{}?apiKey={}", url, polygon_api_key);

            tracing::debug!("Signed source URL: {}", signed_source_url);

            upload_image_to_cdn(cloudflare_client, signed_source_url, image_id).await?
        },
        None => {
            return Err(Error::new(ErrorType::ParseError, "Failed to parse logo URL".to_string()))
        }
    };

    if response.success {
        let new_url = response.result
            .ok_or_else(|| Error::new(ErrorType::ThirdPartyError, "Failed to upload image".to_string()))?
            .variants.first()
            .ok_or_else(|| Error::new(ErrorType::ThirdPartyError, "Failed to upload image".to_string()))?
            .to_owned();

        return Ok(new_url);
    }

    if response.errors.len() > 0 {
        if let Some(error) = response.errors.first() {
            if error.code == 5409 {
                tracing::debug!("Image already exists, returning existing URL");

                // TODO fix this, the hash I need to refactor
                let existing_url = format!("https://imagedelivery.net/2TmEWA4hLHH8IZk5hCKYgg/{}/{}{}/public", image_type.to_string(), ticker, image_id.unwrap());

                return Ok(existing_url);
            }

            return Err(Error::new(ErrorType::ThirdPartyError, error.message.clone()));
        }

        tracing::error!("Failed to upload image: {:?}", response.errors);
    }

    Err(Error::new(ErrorType::ThirdPartyError, "Failed to upload image".to_string()))
}

pub async fn process_branding_images(
    cloudflare_client: &cloudflare_sdk::Client,
    ticker: String,
    branding: Branding,
    polygon_api_key: String
) -> Result<Branding, Error> {
    tracing::info!("Processing branding images for {}", ticker);

    let mut new_branding = Branding {
        logo_url: None,
        icon_url: None,
    };

    match branding.icon_url {
        Some(icon_url) => {
            tracing::debug!("Processing icon URL for {}", ticker);

            let new_icon_url = process_image_url(
                cloudflare_client,
                ImageType::Icon,
                icon_url,
                ticker.to_string(),
                polygon_api_key.clone()
            ).await?;

            new_branding.icon_url = Some(new_icon_url);
        },
        None => {
            tracing::debug!("No icon URL found for {}", ticker);
        }
    }

    match branding.logo_url {
        Some(logo_url) => {
            tracing::debug!("Processing logo URL for {}", ticker);

            let new_logo_url = process_image_url(
                cloudflare_client,
                ImageType::Logo,
                logo_url,
                ticker,
                polygon_api_key.clone()
            ).await?;

            new_branding.logo_url = Some(new_logo_url);
        },
        None => {
            tracing::debug!("No logo URL found for {}", ticker);
        }
    }

    Ok(new_branding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cloudflare_sdk::Client;

    #[tokio::test]
    async fn test_parse_image_format() {
        let png_url = "https://example.com/image.png".to_string();
        let jpg_url = "https://example.com/image.jpg".to_string();
        let jpeg_url = "https://example.com/image.jpeg".to_string();
        let gif_url = "https://example.com/image.gif".to_string();
        let svg_url = "https://example.com/image.svg".to_string();
        let webp_url = "https://example.com/image.webp".to_string();
        let invalid_url = "https://example.com/image".to_string();

        assert_eq!(parse_image_format(&png_url), Some(".png".to_string()));
        assert_eq!(parse_image_format(&jpg_url), Some(".jpg".to_string()));
        assert_eq!(parse_image_format(&jpeg_url), Some(".jpeg".to_string()));
        assert_eq!(parse_image_format(&gif_url), Some(".gif".to_string()));
        assert_eq!(parse_image_format(&svg_url), Some(".svg".to_string()));
        assert_eq!(parse_image_format(&webp_url), Some(".webp".to_string()));
        assert_eq!(parse_image_format(&invalid_url), None);
    }
}
