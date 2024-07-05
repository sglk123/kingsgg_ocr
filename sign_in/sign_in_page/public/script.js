   // 检查本地存储中的 JWT 令牌
   const token = localStorage.getItem('token');
   if (!token) {
       // 如果没有令牌，重定向到登录页面
       window.location.href = 'login.html';
   }

document.getElementById('fileInput').addEventListener('change', () => {
    const fileInput = document.getElementById('fileInput');
    const fileLabel = document.getElementById('fileLabel');
    if (fileInput.files.length > 0) {
        fileLabel.textContent = fileInput.files[0].name;
    }
});

document.getElementById('uploadButton').addEventListener('click', async () => {
    const fileInput = document.getElementById('fileInput');
    const resultText = document.getElementById('resultText');
  const statusMessage = document.getElementById('statusMessage');

    if (fileInput.files.length === 0) {
        resultText.value = "Please select a file to upload.";
        return;
    }

    const file = fileInput.files[0];
    const formData = new FormData();
    formData.append('file', file);
 statusMessage.textContent = "In process...";

    try {
       const response = await fetch('/ocr/upload/', {
            method: 'POST',
            body: formData
        });

        if (!response.ok) {
            throw new Error(`HTTP error! status: ${response.status}`);
        }

        const data = await response.json();
        console.log('Received data:', data);
        resultText.value = data.text.join('\n');
         statusMessage.textContent = "Upload complete.";
    } catch (error) {
        resultText.value = `Error: ${error.message}`;
statusMessage.textContent = "Upload failed.";
    }
});
