// Get the hash of the current page
var g_project_id = location.hash.substring(1);
var g_samples = [];

async function api_get_samples() {
	const response = await fetch('/project/get_samples', {
		headers: {
			'Authorization': 'Bearer ' + g_project_id,
		},
	});

	if (response.status != 200) {
		if (response.status == 401) {
			alert("OOPS: Unknown study");
			return null;
		}
		alert("OOPS: I had trouble talking to the server. Please try again later.");
		return null;
	}

	return await response.json();
}


async function api_get_my_ratings() {
	const response = await fetch('/project/get_my_ratings', {
		headers: {
			'Authorization': 'Bearer ' + g_project_id,
		},
	});

	if (response.status != 200) {
		if (response.status == 401) {
			alert("OOPS: Unknown study");
			return null;
		}

		alert("OOPS: I had trouble talking to the server. Please try again later.");
		return;
	}

	return await response.json();
}


async function api_rate_sample(project_id, sample_id, rating) {
	const response = await fetch('/project/new_rating', {
		method: 'POST',
		headers: {
			'Authorization': 'Bearer ' + project_id,
			'Content-Type': 'application/json',
		},
		body: JSON.stringify({
			sample_id: sample_id,
			rating: rating,
		}),
	});

	if (response.status != 200) {
		alert("OOPS: I had trouble talking to the server. Please try again later.");
		return false;
	}

	return true;
}


async function get_my_samples() {
	const samples = await api_get_samples();
	const my_ratings = await api_get_my_ratings();

	if (samples === null || my_ratings === null) {
		return null;
	}

	// Samples as a dictionary using the sample id as the key
	const samples_dict = {};
	for (const sample of samples) {
		samples_dict[sample.id] = sample;
	}

	for (const rating of my_ratings) {
		delete samples_dict[rating.sample_id];	
	}

	return Object.values(samples_dict);
}


async function onButtonClicked(sample, rating) {
	console.debug("Rating sample " + sample.id + " with rating " + rating + " which is '" + (rating == 0 ? sample.text1 : sample.text2) + "'");
	document.querySelector('.btn-option-a').disabled = true;
	document.querySelector('.btn-option-b').disabled = true;

	// Submit the rating
	if (!await api_rate_sample(g_project_id, sample.id, rating)) {
		return;
	}

	// Get a new sample to rate
	await get_random_sample(g_project_id);
}


function onKeyUp(event) {
	if (event.key == 'a') {
		document.querySelector('.btn-option-a').click();
	}

	if (event.key == 'b') {
		document.querySelector('.btn-option-b').click();
	}
}


async function get_random_sample() {
	console.debug("Getting new sample...");

	// Take a random sample from g_samples and remove it
	if (g_samples.length == 0) {
		alert("No more samples to rate. Thank you for your participation!");
		return;
	}

	const sample_idx = Math.floor(Math.random() * g_samples.length);
	const sample = g_samples[sample_idx];
	g_samples.splice(sample_idx, 1);

	// Randomly flip the text
	const flip = Math.random() < 0.5;

	// Update the page
	document.querySelector('.option-text1').textContent = flip ? sample.text2 : sample.text1;
	document.querySelector('.option-text2').textContent = flip ? sample.text1 : sample.text2;

	// Register click handlers
	document.querySelector('.btn-option-a').onclick = () => onButtonClicked(sample, flip ? 1 : 0);
	document.querySelector('.btn-option-b').onclick = () => onButtonClicked(sample, flip ? 0 : 1);

	document.querySelector('.btn-option-a').disabled = false;
	document.querySelector('.btn-option-b').disabled = false;

	return sample.id;
}


async function main() {
	const samples = await get_my_samples();

	if (samples === null) {
		return;
	}

	g_samples = samples;

	document.querySelector('.study_id').textContent = g_project_id.substring(0, 8);

	// Hook up the keypress handler
	document.onkeyup = (event) => onKeyUp(event);

	// Fetch the first sample
	await get_random_sample(g_project_id);
}

main();