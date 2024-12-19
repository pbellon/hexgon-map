import base64
from locust import HttpUser, TaskSet, task, between
import random
import string


# Utility function to generate a random user ID
def random_username(length=8):
    return "".join(random.choices(string.ascii_letters + string.digits, k=length))


RADIUS = 80


# TaskSet to define user behavior
class UserBehavior(TaskSet):
    user_id: str
    token: str

    def on_start(self):
        """Executed when a virtual user starts. Simulates user login."""
        username = random_username()

        # won't be used here but on app start the game fetches settings and users list
        self.fetch_settings()
        self.fetch_users()

        user = self.login(username)

        if user is not None:
            self.user_id = user["id"]
            self.token = user["token"]
            basic_enc = base64.b64encode(
                f"{self.user_id}:{self.token}".encode("utf-8")
            ).decode("utf-8")

            self.auth_headers = {"Authorization": f"Basic {basic_enc}"}

    def fetch_settings(self):
        with self.client.get("/settings", catch_response=True) as response:
            if response.status_code == 200:
                settings = response.json()
                response.success()
                return settings
            else:
                response.failure("Failed to fetch settings")
                return None

    def fetch_users(self):
        with self.client.get("/users", catch_response=True) as response:
            if response.status_code == 200:
                users = response.json()
                response.success()
                return users
            else:
                response.failure("Failed to fetch settings")
                return None

    def login(self, username):
        """Login user and get a user ID."""
        payload = {"username": username}  # Simulate unique usernames
        with self.client.post("/login", json=payload, catch_response=True) as response:
            if response.status_code == 200:
                user = response.json()
                response.success()
                return user
            else:
                response.failure("Failed to login")
                return None

    @task(3)  # This task will run more frequently
    def click_random_tile(self):
        """Simulate clicking on a random tile."""
        if not self.user_id:
            print("User ID not set. Skipping tile click.")
            return

        # Generate random tile coordinates
        q = random.randint(-RADIUS, RADIUS)
        r = random.randint(-RADIUS, RADIUS)
        url = f"/tile/{q}/{r}"

        with self.client.post(
            url,
            self.user_id,
            headers=self.auth_headers,
            catch_response=True,
        ) as response:
            if response.status_code == 200:
                response.success()
            else:
                response.failure(
                    f"Failed to click tile ({q}, {r}). Status: {response.status_code}"
                )


# Define a Locust User class
class TileUser(HttpUser):
    tasks = [UserBehavior]  # Assign the task set
    wait_time = between(
        0.3, 2.5
    )  # Simulate user "think time" between 30ms and 2.5 seconds
