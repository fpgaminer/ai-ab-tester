// Get the hash of the current page
var g_project_id = location.hash.substring(1);

async function api_get_random_sample(project_id) {
	const response = await fetch('/project/get_sample', {
		headers: {
			'Authorization': 'Bearer ' + project_id,
		}
	});

	// Check return status
	if (response.status != 200) {
		if (response.status == 401) {
			alert("OOPS: Unknown study");
			return null;
		}

		// Log the error message
		alert("OOPS: I had trouble talking to the server. Please try again later.");
		console.log('Error: ' + response.statusText);
		return null;
	}

	return response.json();
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


async function get_random_sample(project_id) {
	console.debug("Getting new sample...");

	const sample = await api_get_random_sample(project_id);
	if (sample === null) {
		return;
	}

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
	// Verify the project id
	if (await api_get_random_sample(g_project_id) === null) {
		document.querySelector('.study_id').textContent = "UNKNOWN";
		return;
	}

	document.querySelector('.study_id').textContent = g_project_id.substring(0, 8);

	// Hook up the keypress handler
	document.onkeyup = (event) => onKeyUp(event);

	// Fetch the first sample
	await get_random_sample(g_project_id);
}

main();