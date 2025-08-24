import requests

system_prompt="""You are a helpful assistant that generates concise, descriptive filenames based on document content. 
Generate a filename that:
- Is descriptive and meaningful
- Uses underscores instead of spaces
- Is professional and organized
- Does not include special characters except underscores and hyphens
- Is between 10-50 characters (excluding extension)
- Captures the main topic/purpose of the document

Respond with ONLY the filename base (without extension), nothing else.

Based on this document content, generate a descriptive filename:
"""

url="http://localhost:11434"
embedding_model="dengcao/Qwen3-Embedding-0.6B:Q8_0"
llm_model="qwen3:4b-instruct-2507-q4_K_M"
                
embedding_url=url+"/api/embeddings"
llm_url=url+"/api/"

embedding_model=embedding_model
llm_model=llm_model

requests.post(f"{llm_url}generate", 
             json={
                 "model": llm_model,
                 "prompt": "",
                 "stream": False,
                 "keep_alive": "5m"  # 모델을 메모리에 유지
             })
    
def get_embedding(texts):
    embeddings = []
    for text in texts:
        r = requests.post("http://localhost:11434/api/embeddings",
                            json={"model": embedding_model, "prompt": text})
        embeddings.append(r.json()['embedding'])
    return embeddings

def get_filename(key_chunks, original_filename, file_extension):
    content_summary = " ".join(key_chunks)[:1000]
    full_prompt = system_prompt + f"Original filename: {original_filename}" + f"Content summary: {content_summary}" + "Generate filename (without extension):\n"
    
    r = requests.post(f"{llm_url}generate", 
                     json={
                         "model": llm_model,
                         "prompt": full_prompt, 
                         "stream": False,
                         "options": {"temperature": 0}
                     })

    generated_name = r.json()['response'].strip()
    print(generated_name)
    return f"{generated_name}.{file_extension}"