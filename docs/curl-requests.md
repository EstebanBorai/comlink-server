# cURL Request Snippets for Endpoints

## Log In User

```bash
curl -i  -u user:user http://127.0.0.1:3000/auth/login
```

## Sign Up User

```bash
curl -i -H "Content-Type: application/json" -d '{"name": "user", "password": "user"}' http://127.0.0.1:3000/auth/signup
```

## Upload User Avatar

```bash
curl -i -X POST -H "Content-Type: multipart/form-data" -F "image=@test.png" http://127.0.0.1:3000/api/v1/users/avatar/89d5de0e-4108-447d-aff1-0f8d0dfa0284
```