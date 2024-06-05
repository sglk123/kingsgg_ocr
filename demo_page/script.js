document.getElementById('uploadButton').addEventListener('click', async () => {
    const fileInput = document.getElementById('fileInput');
    const resultText = document.getElementById('resultText');

    if (fileInput.files.length === 0) {
        resultText.value = "Please select a file to upload.";
        return;
    }

    const file = fileInput.files[0];
    const formData = new FormData();
    formData.append('file', file);

    try {
        const response = await fetch('http://localhost:8000/upload/', {
            method: 'POST',
            body: formData
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();
        console.log('Received data:', data);
        resultText.value = data.text.join('\n');
    } catch (error) {
        resultText.value = `Error: ${error.message}`;
    }
});
