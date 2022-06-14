use actix_web::{HttpRequest, dev::Payload, Error, FromRequest, error::ErrorUnauthorized};
use futures::future;
use hex::FromHex;

pub struct UnvalidatedAuthToken(pub [u8; 32]);

impl FromRequest for UnvalidatedAuthToken {
	type Error = Error;
	type Future = future::Ready<Result<Self, Error>>;

	fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
		let auth_header = req.headers().get("Authorization").and_then(|auth| auth.to_str().ok());
		let auth_token = match auth_header {
			Some(auth_header) => {
				let mut split = auth_header.split(" ");
				split.next();  // skip "Bearer"
				split.next().unwrap_or("")
			}
			None => return future::err(ErrorUnauthorized("Missing Authorization Header")),
		};
		let mut token = [0u8; 32];

		if let Err(_) = hex::decode_to_slice(auth_token, &mut token) {
			return future::err(ErrorUnauthorized("Invalid Authorization Header"));
		}

		future::ok(Self(token))
	}
}

impl UnvalidatedAuthToken {
	pub fn validate(&self, secret: &AuthToken) -> bool {
		ring::constant_time::verify_slices_are_equal(self.0.as_ref(), secret.0.as_ref()).is_ok()
	}
}


#[derive(Clone)]
pub struct AuthToken(pub [u8; 32]);

impl FromHex for AuthToken {
	type Error = hex::FromHexError;

	fn from_hex<T: AsRef<[u8]>>(hex: T) -> Result<Self, Self::Error> {
		let mut token = [0u8; 32];
		hex::decode_to_slice(hex.as_ref(), &mut token)?;
		Ok(Self(token))
	}
}