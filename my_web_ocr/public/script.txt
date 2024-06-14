document.getElementById('uploadButton').addEventListener('click', async () => {
    const fileInput = document.getElementById('fileInput');
    const resultText = document.getElementById('resultText');

    if (fileInput.files.length === 0) {
        alert('Please select a file first.');
        return;
    }

    const file = fileInput.files[0];
    const formData = new FormData();
    formData.append('file', file);

    try {
        const response = await fetch('https://api.your-ocr-service.com/parse', {
            method: 'POST',
            body: formData
        });

        if (!response.ok) {
            throw new Error('Network response was not ok');
        }

        const result = await response.json();
        resultText.value = result.text;
    } catch (error) {
        console.error('Error:', error);
        resultText.value = 'An error occurred while processing the file.';
    }
});
