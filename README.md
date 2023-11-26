# Secure Programming Project

## Web Chat Application

This web-based chat application is designed to be used for real-time communication between users that have joined the channel. Different secure practices are used to make this possible without compromising user security. From the website the users registers, logins, and joins a chat room where they can chat with other users. 
The main features include: 
* Authentication: registration and login 
*	Session management & authorization: only authorized people can join the chat room 
*	Secure chatting: No user identifying information obtainable, only username; no messages are stored on the backend. 
*	Others: rate limiting, input sanitation, max users 

The program is implemented using **Rust** and **Axum** web framework, with other smaller libraries like Tera for html templating. There is no separate front-end, but rather the method of server-side rendering is used. This way the web pages are rendered on the server and sent to the clients. 

### Registration

The registration starts from the client, from where the plaintext username and plaintext password are sent to the server using HTTP. This communication path is not encrypted so there is a known security risk here. In production this would require HTTPS. When the server receives the credentials. 
1.	Username and password lengths are checked, minimum of 4 and minimum of 8 characters, respectively. 
    1. Username minimum length 4
    2. Password minimum length 8
    3. Letter according to regex r"^[a-zA-Z0-9_]+$” (https://owasp.org/www-projectproactive-controls/v3/en/c5-validate-inputs) 
2.	It is checked that there is not already the same username in use. 
3.	The user is created. A cryptographically secure salt is generated using library OsRng, the salt is appended to the password and hashed using **SHA-256**. Then the username, password, and salt are stored in a hash-map (memory). During server restart these are lost. 
4.	A response is given to the client. 

### Login and session management

Similarly, the username and password are sent with a HTTP request to the backend. 
1.	Username and password is validated both on the frontend and backend, similarly as in registration. 
2.	Code checks if there is a user with that name and validates the password. The password is validated by taking the salt, appending it to the given password and hashing it with SHA256. Then the stored hash is compared to the calculated hash. 
3.	In case of a correct password, a JWT token is generated. The token contains an expiration time (expires in 1 hour) and the username as the subject field in the Claims part of the token. The auth token creation uses HS256 symmetric encryption. Only the server knows the secret key, currently just a constant in the code file. 
4.	The generated token is added to the cookie header value as, for example:  
    1. Authorization=Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOiJoamtna2hqZ2hqa2doamtnIiwiZX hwIjoxNjg0MTg2MzkzNzIxfQ.PYyJWcENIawes5wPAb1PLFVVOa4q3bBTQqeNEZzbZA M 
5.	The response is sent to the client and stored in the browser cookies. The token is sent in further requests. 

![image](https://github.com/Mandariini/secure_programming_project/assets/60143063/27326541-3540-49bd-81db-3da4c10a36c0)

### Chat communication

The chat communication is implemented using WebSockets. During the connection handshake 
1.	The client sends the JWT token as the first message. 
2.	The token is decoded using the secret key that the server knows.  
    1. If the token is invalid, not existent, or expired, the websocket connection is closed and user gets a message “Token expired, please relogin”. If successful, the server gets the username from the claims part of the token as explained earlier and joins the client to the channel. 
3.	A welcome message is shown to the user, other users get a message “<username> joined.” 
4.	When the user leaves the chat, the connection is closed. 
When a client sends a message, the server forwards it to the other clients using a broadcast type of queue (multi-producer, multi-consumer). The only thing forwarded from one user to another is their username and the message they typed. The message is sanitized:

![image](https://github.com/Mandariini/secure_programming_project/assets/60143063/1f26850c-4370-46d6-9f29-9e9d5cac6d60)

The maximum message limit is 100 characters. Each client can send a message once a second, otherwise an error message will be shown only to that client. This check is done on the backend. There is a check also on the frontend that prevents from typing in the chat box for 2 seconds after sending a message. This is to prevent constantly getting these errors when typing fast. 

Maximum amount users in the chat has been capped to 20. If another tries to join, they will get the following message when pressing “Join Chat”. 

![image](https://github.com/Mandariini/secure_programming_project/assets/60143063/71f78633-2d69-4fe8-b9c7-cadf656e372d)

### Example

![image](https://github.com/Mandariini/secure_programming_project/assets/60143063/173dc7e2-1db2-49a2-b880-d38c700bccf2)

The code consists of Src: 
*	Main.rs: The main entry point. Starts the server, routes requests, and instantiates the needed structures 
*	Handlers.rs: Defines HTTP handlers for the requests 
*	Models.rs: Contains the request types, response types, and the UserInfo that implements password hashing and verification 
*	Auth.rs: Contains the logic for the JWT token creation, verification and decoding.
*	Websocket.rs: The main chat logic using WebSockets

Resources: 
*	Base.html: HTML template file 
*	Chat.html: Chat page 
*	Index.html: Welcome page 
*	Login.html: Login page 
*	Registration.html: Register page 
