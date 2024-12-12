## TODO

- [x] server skeleton
- [x] simple frontend (basic grid drawing, click handle)
- [x] algorithms with cubes coords, storage with axial coords
- [x] handle game logic when clicking on tile (front+back)
- [x] user registration
- [x] basic websocket communication
- [ ] handle a new player appearance, maybe add color to ws messages
- [ ] reduce user ID size if possible
- [ ] make color more random
- [ ] (front/api.ts) handle localStorage to avoid losing auth
- [ ] (front+backend) handle token-based registration and token expiration that will be helpful to clean the map from inactive users (maybe start with 1h validity => after one hour of inactivity you must log again)
- [ ] benchmark lots of concurrent users to see how things behave
- [ ] see what broke

## Resources

- https://discourse.threejs.org/t/hexagonal-grid-formation/18396
- https://github.com/vonWolfehaus/von-grid?tab=readme-ov-file
