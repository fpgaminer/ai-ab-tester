#!/usr/bin/env python3
import requests
import random


# TODO: Read from config.toml when Python 3.11 is available
ADMIN_SECRET = "314afd07c4b48c37d3fc85770b7a5d393238910d1eb617ed71dcdadd2e23c446"
SERVER_URL = "http://127.0.0.1:8080"


def main():
	# Create a new project
	resp = requests.post(f"{SERVER_URL}/admin/new_project", headers={"Authorization": f"Bearer {ADMIN_SECRET}"})
	resp.raise_for_status()
	resp = resp.json()
	project_id = resp["project_id"]
	project_admin_token = resp["admin_token"]

	# Create some samples
	expected_samples = []
	for _ in range(10):
		text1 = str(random.random())
		text2 = str(random.random())
		source1 = str(random.random())
		source2 = str(random.random())
		resp = requests.post(f"{SERVER_URL}/project/new_sample", headers={"Authorization": f"Bearer {project_admin_token}"}, json={
			"project": project_id,
			"text1": text1,
			"text2": text2,
			"source1": source1,
			"source2": source2,
		})
		resp.raise_for_status()
		expected_samples.append([text1, text2, source1, source2])
	
	# Submit a duplicate sample to test the unique constraint
	resp = requests.post(f"{SERVER_URL}/project/new_sample", headers={"Authorization": f"Bearer {project_admin_token}"}, json={
		"project": project_id,
		"text1": text1,
		"text2": text2,
		"source1": source1,
		"source2": source2,
	})
	resp.raise_for_status()
	
	# Verify the samples
	resp = requests.get(f"{SERVER_URL}/project/get_samples", headers={"Authorization": f"Bearer {project_id}"})
	resp.raise_for_status()
	server_samples = resp.json()
	server_samples.sort(key=lambda x: x["id"])
	for sample,server_sample in zip(expected_samples, server_samples):
		assert server_sample['text1'] == sample[0]
		assert server_sample['text2'] == sample[1]
		assert server_sample['source1'] == sample[2]
		assert server_sample['source2'] == sample[3]
	
	# Submit some ratings
	expected_ratings = []

	for _ in range(32):
		resp = requests.get(f"{SERVER_URL}/project/get_sample", headers={"Authorization": f"Bearer {project_id}"})
		resp.raise_for_status()
		sample_id = resp.json()["id"]
		rating = random.randint(0, 1)
		resp = requests.post(f"{SERVER_URL}/project/new_rating", headers={"Authorization": f"Bearer {project_id}"}, json={"sample_id": sample_id, "rating": rating})
		resp.raise_for_status()
		expected_ratings.append([sample_id, rating])
	
	# Verify the ratings
	resp = requests.get(f"{SERVER_URL}/project/get_ratings", headers={"Authorization": f"Bearer {project_id}"})
	resp.raise_for_status()
	server_ratings = resp.json()

	for server_rating in server_ratings:
		expected_ratings.remove([server_rating['sample_id'], server_rating['rating']])
	
	# Test get_my_ratings
	# Note: This doesn't fully test get_my_ratings functionality
	resp = requests.get(f"{SERVER_URL}/project/get_my_ratings", headers={"Authorization": f"Bearer {project_id}"})
	resp.raise_for_status()
	assert(set((x['id'], x['rating'], x['sample_id']) for x in resp.json()) == set((x['id'], x['rating'], x['sample_id']) for x in server_ratings))


if __name__ == '__main__':
	main()