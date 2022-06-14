A service for doing A-B tests of AI generated text.  I use it for testing sampling configurations.


API:
	Include API_SECRET in Authorization header.

	* POST /new_project
		* Authorization: Bearer ADMIN_KEY
		* Returns: Json { project_id, project_admin_key }
	* POST /project/new_sample
		* Authorization: Bearer PROJECT_ADMIN_KEY
		* Body: Json { project: PROJECT_ID, text1: String, text2: String }
		* Returns: EMPTY BODY
	* GET /project/get_samples
		* Authorization: Bearer PROJECT_ID
		* Returns: Json [ { sample_id, text1, text2 }]
	* GET /project/get_sample
		* Authorization: Bearer PROJECT_ID
		* Returns: Json { sample_id, text1, text2 }
	* POST /project/new_rating
		* Authorization: Bearer PROJECT_ID
		* Body: Json { sample_id, rating }
		* Returns: EMPTY BODY
	* GET /project/get_ratings
		* Authorization: Bearer PROJECT_ID
		* Returns: Json [ { id, sample_id, ip, rating }]
		* NOTE: ip is cryptographically hashed to improve user privacy.  Only the ADMIN could possibly reverse the hash.



Run locally:
	* `./run-test-db.sh`
	* `cargo run`



Build docker container:
	* `docker build -t ai-ab-tester .`
	* `docker tag ai-ab-tester DEST`
	* `docker push DEST`