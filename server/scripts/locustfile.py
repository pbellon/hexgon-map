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
        user = self.login(username)
        self.user_id = user["id"]
        self.token = user["token"]
        basic_enc = base64.b64encode(
            f"{self.user_id}:{self.token}".encode("utf-8")
        ).decode("utf-8")

        self.auth_headers = {"Authorization": f"Basic {basic_enc}"}

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
                print(f"Clicked tile ({q}, {r}) successfully.")
            else:
                response.failure(
                    f"Failed to click tile ({q}, {r}). Status: {response.status_code}"
                )

    # @task(1)  # Less frequent task
    # def perform_multiple_tile_changes(self):
    #     """Simulate sequential tile clicks."""
    #     if not self.user_id:
    #         print("User ID not set. Skipping multiple tile clicks.")
    #         return

    #     for _ in range(5):  # Simulate clicking on 5 tiles in sequence
    #         q = random.randint(-10, 10)
    #         r = random.randint(-10, 10)
    #         payload = {"user_id": self.user_id, "action": "click"}
    #         url = f"/tile/{q}/{r}"

    #         with self.client.post(url, json=payload, catch_response=True) as response:
    #             if response.status_code == 200:
    #                 response.success()
    #                 print(f"Sequential click on tile ({q}, {r}) succeeded.")
    #             else:
    #                 response.failure(f"Sequential click on tile ({q}, {r}) failed.")


# Define a Locust User class
class TileUser(HttpUser):
    tasks = [UserBehavior]  # Assign the task set
    wait_time = between(1, 3)  # Simulate user "think time" between 1 and 3 seconds
