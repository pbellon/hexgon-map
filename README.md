## TODO

- [x] server skeleton
- [x] simple frontend (basic grid drawing, click handle)
- [x] algorithms with cubes coords, storage with axial coords
- [x] handle game logic when clicking on tile (front+back)
- [x] user registration
- [x] basic websocket communication
- [x] handle a new player appearance, maybe add color to ws messages
- [-] change the way the frontend app is initialized
  - [x] 0. set loading mode on login page
  - [x] 1. load game data
  - [x] 2. start rendering in the background and start listenning for WebSocket events
  - [x] 3. once initialized and a first render occured, exit loading mode
  - [ ] 4. if user was previously logged (check localStorage) and token did not expired (not sure how)
       then directly allow play
  - [x] 5. if not logged (or kicked out) let user enters its information (if not previously logged and
       token did not expired)
- [x] enforce `GameData::handle_click` algo by adding strong unit tests
  - => now core mechanics tested, see `tests/game_tests.rs`
- [ ] create new `GET /users` endpoint and use it
- [ ] create new `GET /settings` endpoint and use it
- [ ] create new `GET /tiles?batch={n}` endpoint and use it
      => split the grid in batches (10 ?) to avoid fetching the grid all at once
- [ ] no need to wait for the whole grid to load before rendering
- [ ] (front) improve state management, we need a single source of truth
- [ ] (front+back) handle user scores
- [x] (back) reduce user ID size if possible?
- [ ] (back) make color more random (use generated uuid ?)

- [x] (front+backend) handle token-based auth
- [ ] add token expiration and clean the map of inactive users tiles (maybe start with 1h validity => after one hour of
      inactivity you must log again)
- [ ] (front+back) handle user clean (when kicked out after inactivity)
- [ ] (front/api.ts) handle localStorage to avoid losing auth
- [x] benchmark lots of concurrent users to see how things behave
- [x] see what broke
      -> `GET /data`, pète fort
- [ ] add credits where due
      -> Thanks ThreeJS for the wonderful lib
      -> Thanks Red Blob games for the algorithms
      -> Thanks ToneJS (if we use it)

## General game rules

- a tile is either owned or not
- when owned a tile has a strength, this strength make the tile less likely to be disowned and will require additional click to be owned by someone else.
- this strength is a virtual attribute that is not stored directly
- instead the only counter that is stored is the "damage" one,
  - it can be increased when a user click on a tile it does not own
  - it can be decreased when a user click on a tile it owns
  - it will be reset to 0 when a tile owner changes
  - it's used to calculate a tile strength => strengh = number of contiguous tiles owned by user - damage
  - when this compute strength reach 0, the tile changes of owner

## Resources

- Red blob games incredible hexagons articles https://www.redblobgames.com/grids/hexagons/#coordinates
- https://discourse.threejs.org/t/hexagonal-grid-formation/18396
- https://github.com/vonWolfehaus/von-grid?tab=readme-ov-file
