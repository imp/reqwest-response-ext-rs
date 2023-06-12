use super::*;

/// Holds raw response body, while remembering desired shape of the success (`T`)
/// and failure (`E`) variants.
///
#[derive(Clone, Debug)]
pub struct TypedResponse<T, E> {
    body: bytes::Bytes,
    result: Result<PhantomData<T>, PhantomData<E>>,
}

impl<T, E> TypedResponse<T, E>
where
    T: de::DeserializeOwned,
    E: de::DeserializeOwned + From<json::Error>,
{
    /// Converts `reqwest::blocking::Response` into `TypedResponse<T, E>`
    ///
    pub fn try_from_response(response: reqwest::blocking::Response) -> reqwest::Result<Self> {
        let result = match response.status().is_success() {
            false => Err(PhantomData),
            true => Ok(PhantomData),
        };

        // Bail early on server error
        if response.status().is_server_error() {
            response.error_for_status_ref()?;
        }

        let body = response.bytes()?;

        Ok(Self { body, result })
    }

    /// Access the raw HTTP response as bytes
    ///
    pub fn bytes(&self) -> &bytes::Bytes {
        &self.body
    }

    /// Access the raw HTTP response body as text
    ///
    pub fn text(&self) -> Cow<'_, str> {
        String::from_utf8_lossy(&self.body)
    }

    /// Convert this response into `Result<serde_json::Value, serde_json::Value>`
    /// where `Ok` and `Err` variants are based on the original HTTP Status
    /// In case the body is not a valid JSON by itself it creates a JSON object
    /// with deserialization error as a string content.
    ///
    pub fn into_json(self) -> Result<json::Value, json::Value> {
        let json_err = |e: json::Error| json::json! { e.to_string() };
        match self.result {
            Ok(_) => Ok(json::from_slice(&self.body).map_err(json_err)?),
            Err(_) => Err(json::from_slice(&self.body).map_err(json_err)?),
        }
    }

    /// Convert this response into `Result<T, E>` where `Ok` and `Err` variants
    /// are based on the original HTTP Status and type parameters. In case of
    /// JSON deserialization error it will be converted into `E`.
    pub fn into_result(self) -> Result<T, E> {
        match self.result {
            Ok(_) => Ok(json::from_slice(&self.body)?),
            Err(_) => Err(json::from_slice(&self.body)?),
        }
    }
}

pub trait ResponseExt: Sized {
    fn try_from_response<T, E>(self) -> reqwest::Result<TypedResponse<T, E>>
    where
        T: de::DeserializeOwned + Send,
        E: de::DeserializeOwned + From<json::Error> + Send;
}

impl ResponseExt for reqwest::blocking::Response {
    fn try_from_response<T, E>(self) -> reqwest::Result<TypedResponse<T, E>>
    where
        T: de::DeserializeOwned + Send,
        E: de::DeserializeOwned + From<json::Error> + Send,
    {
        TypedResponse::try_from_response(self)
    }
}
