source ~/env/bin/activate
llama-server -m ~/Downloads/models/gemma-4-E2B-it-Q4_0.gguf --mmproj ~/Downloads/models/mmproj-gemma-4-E2B-it-Q8_0.gguf -c 32768 -ngl 100 --port 8000 -ub 1024
