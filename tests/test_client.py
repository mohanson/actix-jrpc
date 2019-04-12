import requests

r = requests.post('http://127.0.0.1:8080/', json={
    'jsonrpc': '2.0',
    'method': 'ping',
    'params': [],
    'id': 1
})
print(r.json())
