from fastapi import FastAPI, File, UploadFile
from fastapi.responses import JSONResponse
import shutil
import os
from paddleocr import PaddleOCR
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # 可以根据需求指定特定的域名
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

UPLOAD_FOLDER = "uploads"
ocr = PaddleOCR(use_angle_cls=True, lang='ch')

# 确保上传文件夹存在
if not os.path.exists(UPLOAD_FOLDER):
    os.makedirs(UPLOAD_FOLDER)

@app.post("/upload/")
async def upload_file(file: UploadFile = File(...)):
    file_location = f"{UPLOAD_FOLDER}/{file.filename}"
    with open(file_location, "wb") as buffer:
        shutil.copyfileobj(file.file, buffer)

    # 使用 PaddleOCR 进行 OCR 处理
    result = ocr.ocr(file_location, cls=True)
    text_results = []
    
    if result:
        for line in result:
            if line:  # 检查 line 是否为 None 或空
                for box in line:
                    text_results.append(box[1][0])
    else:
        return JSONResponse(content={"error": "No OCR results found"}, status_code=400)

    return JSONResponse(content={"text": text_results})
if __name__ == "__main__":
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
