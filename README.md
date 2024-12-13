## TODO

- [x] server skeleton
- [x] simple frontend (basic grid drawing, click handle)
- [x] algorithms with cubes coords, storage with axial coords
- [x] handle game logic when clicking on tile (front+back)
- [x] user registration
- [x] basic websocket communication
- [x] handle a new player appearance, maybe add color to ws messages
- [ ] (front) improve state management, we need a single source of truth
- [ ] (front+back) handle user scores
- [ ] (back) reduce user ID size if possible?
- [ ] (back) make color more random (use generated uuid ?)
- [ ] change the way the frontend app is initialized  
       0. set loading mode on login page  
       1. load game data  
       2. start rendering in the background and start listenning for WebSocket events  
       3. once initialized and a first render occured, exit loading mode  
       4. if user was previously logged (check localStorage) and token did not expired (not sure how)
      then directly allow play  
       5. if not logged (or kicked out) let user enters its information (if not previously logged and
      token did not expired)
- [ ] (front+backend) handle token-based registration and token expiration that will be helpful to
      clean the map from inactive users (maybe start with 1h validity => after one hour of
      inactivity you must log again)
- [ ] (front+back) handle user clean (when kicked out after inactivity)
- [ ] (front/api.ts) handle localStorage to avoid losing auth
- [ ] benchmark lots of concurrent users to see how things behave
- [ ] see what broke
- [ ] add credits where due
      -> Thanks ThreeJS for the wonderful lib
      -> Thanks Red Blob games for the algorithms
      -> Thanks ToneJS (if we use it)

## Resources

- https://discourse.threejs.org/t/hexagonal-grid-formation/18396
- https://github.com/vonWolfehaus/von-grid?tab=readme-ov-file
