use std::{net::{SocketAddr, IpAddr}, fs};
use actix_web::{HttpServer, middleware::Logger, App, web::{self, Data}, HttpResponse, HttpRequest};
use anyhow::Context;
use env_logger::Env;
use ring::{rand::SystemRandom, hmac};
use serde::{Serialize, Deserialize};
use sqlx::{postgres::{PgPoolOptions, PgRow}, Row};
use serde_json::json;
use actix_files::Files;

use crate::{error::ServerError, auth::UnvalidatedAuthToken, auth::AuthToken};

mod error;
mod auth;


#[derive(Clone, Deserialize)]
struct Config {
	#[serde(with = "hex")]
	admin_secret: AuthToken,
	database_url: String,
	bind: SocketAddr,
}


const MAXIMUM_REQUEST_SIZE: usize = 1 * 1024 * 1024; // bytes


#[actix_web::main]
async fn main() -> Result<(), anyhow::Error> {
	// Read config
	let config_path = std::env::var("CONFIG_PATH").unwrap_or("config.toml".to_string());
	let config = fs::read_to_string(config_path).context("reading config")?;
	let config: Config = toml::from_str(&config).context("reading config")?;

	// Env logger
	env_logger::Builder::from_env(Env::default().default_filter_or("warn,actix_web=debug,ai_ab_tester=debug,actix_server=info")).init();

	// Setup Database
	let db_pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(&config.database_url).await?;
	sqlx::migrate!("./migrations")
		.run(&db_pool)
		.await?;

	// Setup HTTP server
	let data_config = Data::new(config.clone());
	let server = HttpServer::new(move || {
		let logger = Logger::default();

		App::new()
			.wrap(logger)
			.app_data(web::PayloadConfig::default().limit(MAXIMUM_REQUEST_SIZE))
			.app_data(Data::new(db_pool.clone()))
			.app_data(data_config.clone())
			.service(new_project)
			.service(new_sample)
			.service(get_sample)
			.service(get_samples)
			.service(new_rating)
			.service(get_ratings)
			.service(get_my_ratings)
			.service(Files::new("/", "webapp/dist/").index_file("index.html"))
	})
	.bind(&config.bind)?
	.run();

	server.await?;

	Ok(())
}


#[actix_web::post("/admin/new_project")]
async fn new_project(db_pool: Data<sqlx::PgPool>, auth_token: UnvalidatedAuthToken, server_config: Data<Config>) -> Result<HttpResponse, ServerError> {
	if !auth_token.validate(&server_config.admin_secret) {
		return Ok(HttpResponse::Unauthorized().body("Invalid Authorization"));
	}

	let rng = SystemRandom::new();
	let project_id: [u8; 32] = ring::rand::generate(&rng).unwrap().expose();
	let project_admin_token: [u8; 32] = ring::rand::generate(&rng).unwrap().expose();

	sqlx::query("INSERT INTO projects (id, admin_key) VALUES ($1,$2)")
		.bind(&project_id[..])
		.bind(&project_admin_token[..])
		.execute(&**db_pool)
		.await
		.context("Insert new project")?;

	Ok(HttpResponse::Ok().json(json!({
		"project_id": hex::encode(project_id),
		"admin_token": hex::encode(project_admin_token),
	})))
}


#[derive(Deserialize)]
struct NewSampleRequest {
	#[serde(with = "hex")]
	project: AuthToken,
	text1: String,
	text2: String,
	source1: String,
	source2: String,
}

#[actix_web::post("/project/new_sample")]
async fn new_sample(db_pool: Data<sqlx::PgPool>, auth_token: UnvalidatedAuthToken, request: web::Json<NewSampleRequest>) -> Result<HttpResponse, ServerError> {
	// Query the database for project to get the admin key
	let project_admin_key= sqlx::query("SELECT admin_key FROM projects WHERE id = $1")
		.bind(&request.project.0[..])
		.map(|row: PgRow| {
			let data: Option<Vec<u8>> = row.get(0);
			let foo: Option<Option<[u8; 32]>> = data.map(|data| data.try_into().ok());
			foo.flatten()
		})
		.fetch_optional(&**db_pool)
		.await
		.context("Query project")?
		.flatten()
		.map(|x| AuthToken(x));
	
	let project_admin_key = match project_admin_key {
		Some(key) => key,
		None => return Ok(HttpResponse::NotFound().body("Unknown Project")),
	};

	if !auth_token.validate(&project_admin_key) {
		return Ok(HttpResponse::Unauthorized().body("Invalid Authorization"));
	}

	// Insert the sample
	sqlx::query("INSERT INTO samples (project_id, text1, text2, source1, source2) VALUES ($1,$2,$3,$4,$5) ON CONFLICT DO NOTHING")
		.bind(&request.project.0[..])
		.bind(&request.text1)
		.bind(&request.text2)
		.bind(&request.source1)
		.bind(&request.source2)
		.execute(&**db_pool)
		.await
		.context("Insert new sample")?;

	Ok(HttpResponse::Ok().finish())
}


#[actix_web::get("/project/get_sample")]
async fn get_sample(db_pool: Data<sqlx::PgPool>, project_id: UnvalidatedAuthToken) -> Result<HttpResponse, ServerError> {
	#[derive(sqlx::FromRow, Serialize)]
	struct DbSample {
		id: i64,
		text1: String,
		text2: String,
	}

	// Query the database for a sample
	let sample = sqlx::query_as::<_, DbSample>("SELECT id, text1, text2 FROM samples WHERE project_id = $1 ORDER BY random() LIMIT 1")
		.bind(&project_id.0[..])
		.fetch_optional(&**db_pool)
		.await
		.context("Query sample")?;
	
	match sample {
		Some(sample) => Ok(HttpResponse::Ok().json(sample)),
		None => Ok(HttpResponse::NotFound().body("Unknown Project")),
	}
}


fn get_masked_ip(http_request: &HttpRequest, server_config: &Config) -> Result<hmac::Tag, ServerError> {
	let connection_info = http_request.connection_info();
	let user_ip = connection_info.realip_remote_addr().ok_or_else(|| anyhow::Error::msg("Could not get remote address"))?;
	let user_ip: IpAddr = user_ip.parse().context("Could not parse remote address")?;
	let user_ip_octets = match user_ip {
		IpAddr::V4(ip) => ip.octets().to_vec(),
		IpAddr::V6(ip) => ip.octets().to_vec(),
	};
	let hash_key = hmac::Key::new(hmac::HMAC_SHA256, &server_config.admin_secret.0);
	Ok(hmac::sign(&hash_key, &user_ip_octets))
}


#[derive(Deserialize)]
struct NewRatingRequest {
	sample_id: i64,
	rating: i64,
}

#[actix_web::post("/project/new_rating")]
async fn new_rating(db_pool: Data<sqlx::PgPool>, project_id: UnvalidatedAuthToken, request: web::Json<NewRatingRequest>, http_request: HttpRequest, server_config: Data<Config>) -> Result<HttpResponse, ServerError> {
	// Query the database to make sure the project_id is valid
	let project_exists = sqlx::query("SELECT id FROM projects WHERE id = $1")
		.bind(&project_id.0[..])
		.fetch_optional(&**db_pool)
		.await
		.context("Query project")?;
	
	if project_exists.is_none() {
		return Ok(HttpResponse::NotFound().body("Unknown Project"));
	}

	let hashed_ip = get_masked_ip(&http_request, &server_config)?;

	// Insert the rating
	sqlx::query("INSERT INTO ratings (project_id, sample_id, ip, rating) VALUES ($1,$2,$3,$4)")
		.bind(&project_id.0[..])
		.bind(request.sample_id)
		.bind(hashed_ip.as_ref())
		.bind(request.rating)
		.execute(&**db_pool)
		.await
		.context("Insert new rating")?;

	Ok(HttpResponse::Ok().finish())
}


#[actix_web::get("/project/get_ratings")]
async fn get_ratings(db_pool: Data<sqlx::PgPool>, project_id: UnvalidatedAuthToken) -> Result<HttpResponse, ServerError> {
	#[derive(sqlx::FromRow, Serialize)]
	struct DbRating {
		id: i64,
		sample_id: i64,
		#[serde(with = "hex")]
		ip: Vec<u8>,
		rating: i32,
	}

	// Query the database for ratings
	let ratings = sqlx::query_as::<_, DbRating>("SELECT id, sample_id, ip, rating FROM ratings WHERE project_id = $1")
		.bind(&project_id.0[..])
		.fetch_all(&**db_pool)
		.await
		.context("Query ratings")?;
	
	Ok(HttpResponse::Ok().json(ratings))
}


#[actix_web::get("/project/get_my_ratings")]
async fn get_my_ratings(db_pool: Data<sqlx::PgPool>, project_id: UnvalidatedAuthToken, http_request: HttpRequest, server_config: Data<Config>) -> Result<HttpResponse, ServerError> {
	#[derive(sqlx::FromRow, Serialize)]
	struct DbRating {
		id: i64,
		sample_id: i64,
		rating: i32,
	}

	let hashed_ip = get_masked_ip(&http_request, &server_config)?;

	// Query the database for ratings
	let ratings = sqlx::query_as::<_, DbRating>("SELECT id, sample_id, rating FROM ratings WHERE project_id = $1 AND ip = $2")
		.bind(&project_id.0[..])
		.bind(hashed_ip.as_ref())
		.fetch_all(&**db_pool)
		.await
		.context("Query ratings")?;
	
	Ok(HttpResponse::Ok().json(ratings))
}


#[actix_web::get("/project/get_samples")]
async fn get_samples(db_pool: Data<sqlx::PgPool>, project_id: UnvalidatedAuthToken) -> Result<HttpResponse, ServerError> {
	#[derive(sqlx::FromRow, Serialize)]
	struct DbSample {
		id: i64,
		text1: String,
		text2: String,
		source1: String,
		source2: String,
	}

	// Query the database for samples
	let samples = sqlx::query_as::<_, DbSample>("SELECT id, text1, text2, source1, source2 FROM samples WHERE project_id = $1")
		.bind(&project_id.0[..])
		.fetch_all(&**db_pool)
		.await
		.context("Query samples")?;
	
	Ok(HttpResponse::Ok().json(samples))
}
