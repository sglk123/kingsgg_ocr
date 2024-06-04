import requests

url = "http://localhost:8000/upload/"

# 文件路径
file_path = r"D:\ocr_umi\Umi-OCR_Rapid_v2.1.1\sglk_sample2.png"

# 打开文件，以二进制模式读取
with open(file_path, "rb") as file:
    # 构造 multipart/form-data 格式的数据
    files = {"file": file}
    # 发送 POST 请求
    response = requests.post(url, files=files)

# 输出响应内容
print(response.json())