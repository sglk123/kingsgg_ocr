// server.js
const express = require('express');
const proxy = require('http-proxy-middleware');  
const path = require('path');
const app = express();
const port = 4000;

// 设置静态文件目录
app.use(express.static(path.join(__dirname, 'public')));

app.get('/', (req, res) => {
  res.sendFile(path.join(__dirname, 'public', 'login.html'));
});

// Proxy configuration for older http-proxy-middleware
app.use('/api', proxy({
    target: 'http://127.0.0.1:8080',  // Backend service address
    changeOrigin: true,
    pathRewrite: {
        '^/api': ''  // Rewrite path: remove `/api`
    },
}));

// 新增专用于上传的代理
app.use('/ocr/upload', proxy({
    target: 'http://127.0.0.1:8000',  // 后端服务地址，修改端口号为8080
    changeOrigin: true,
    pathRewrite: {
        '^/ocr/upload': '/upload/'  // 重写路径确保请求被正确转发
    }
}));

app.listen(port, () => {
  console.log(`Server is running at http://0.0.0.0:${port}`);
});
