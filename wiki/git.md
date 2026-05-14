# Git

## Authentication

Storing Git credentials is more complicated than I thought, there is some options:

- Github CLI (not working like I want)
- Git Credential Manager (designed for Windows so complicated on Linux)
- Git cache, but each time cache expires we should generate new token, because Github does not allow usage of password to login
- Git plain text, not very secure, even if it's a token

The only simple and kind of more secure solution I found is using git-credential-oauth

References:
- [https://git-scm.com/doc/credential-helpers]
- [https://git-scm.com/docs/gitcredentials]

