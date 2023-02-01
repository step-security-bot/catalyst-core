window.SIDEBAR_ITEMS = {"constant":[["API_TOKEN_HEADER","Header where token should be present in requests"]],"fn":[["api_token_filter","A warp filter that checks authorization through API tokens. The header `API_TOKEN_HEADER` should be present and valid otherwise the request is rejected."]],"struct":[["ApiToken","API Token wrapper type"],["ApiTokenManager","API token manager is an abstraction on the API tokens for the service The main idea is to keep the service agnostic of what kind of backend we are using such task. Right now we rely on a SQLlite connection. But in the future it maybe be something else like a REDIS, or some other hybrid system."]]};