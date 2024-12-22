import http from "k6/http";
import encoding from "k6/encoding";

import { sleep, check } from "k6";

const HOST = __ENV.BASE_URL || "http://localhost:8080";

export let options = {
  stages: [
    { duration: "5m", target: 5000 }, // Ramp-up to 5k  users,
    { duration: "1m", target: 7000 }, // Ramp-up to 7k users
    { duration: "1m", target: 10000 }, // surge at 10k
    { duration: "1m", target: 5000 }, // ramp-down to 5k users
    { duration: "1m", target: 0 }, // Ramp-down to 0 users
  ],
};

const RADIUS = 80;

function randomUsername() {
  return Math.random().toString(36).substring(2, 10);
}

function randomTile() {
  return {
    q: Math.floor(Math.random() * (RADIUS * 2 + 1)) - RADIUS,
    r: Math.floor(Math.random() * (RADIUS * 2 + 1)) - RADIUS,
  };
}

export default function () {
  const username = randomUsername();

  // Login
  const loginRes = http.post(`${HOST}/login`, JSON.stringify({ username }), {
    headers: { "Content-Type": "application/json" },
  });
  check(loginRes, { "login success": (r) => r.status === 200 });

  const { id: userId, token } = loginRes.json();
  const authHeader = `Basic ${encoding.b64encode(`${userId}:${token}`)}`;

  // Fetch Settings
  const settingsRes = http.get(`${HOST}/settings`, {
    headers: { Authorization: authHeader },
  });
  check(settingsRes, { "fetch settings success": (r) => r.status === 200 });

  // Fetch Users
  const usersRes = http.get(`${HOST}/users`, {
    headers: { Authorization: authHeader },
  });
  check(usersRes, { "fetch users success": (r) => r.status === 200 });

  // Click random tiles
  for (let i = 0; i < 10; i++) {
    const { q, r } = randomTile();
    const tileRes = http.post(`${HOST}/tile/${q}/${r}`, null, {
      headers: { Authorization: authHeader },
    });
    check(tileRes, { "click tile success": (r) => r.status === 200 });
    sleep(2); // Simulate user think time
  }
}
