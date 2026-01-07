# Disclaimer: Vibe-coded by Gemini 3 Pro!

import getpass
import requests
from typing import Any


def main() -> None:
    base_url: str = (
        input("API Base URL [http://localhost:8080/api]: ").strip()
        or "http://localhost:8080/api"
    )
    admin_user: str = input("SysAdmin Username: ").strip()
    admin_pass: str = getpass.getpass("SysAdmin Password: ")

    with requests.Session() as session:
        try:
            login_resp = session.post(
                f"{base_url}/login",
                json={"username": admin_user, "password": admin_pass},
            )
            login_resp.raise_for_status()
            print("Login successful.")

            username: str = input("New Username: ").strip()
            password: str = getpass.getpass("New Password: ")
            role: str = input(
                "Role (admin, student, teacher, defense_board, office): "
            ).strip()

            payload: dict[str, Any] = {
                "username": username,
                "password": password,
                "role": role,
            }

            if role in ("student", "teacher"):
                payload["name"] = input("Real Name: ").strip()

            if role == "student":
                major_id_str = input("Major ID: ").strip()
                if major_id_str:
                    payload["major_id"] = int(major_id_str)

            create_resp = session.post(f"{base_url}/user", json=payload)
            create_resp.raise_for_status()

            print("\nUser Created Successfully:")
            print(create_resp.json())

        except requests.exceptions.RequestException as e:
            print(f"API Error: {e}")
            if hasattr(e, "response") and e.response is not None:
                print(f"Details: {e.response.text}")
        except ValueError as e:
            print(f"Input Error: {e}")
        finally:
            try:
                session.post(f"{base_url}/logout")
                print("Logged out.")
            except:
                pass


if __name__ == "__main__":
    main()
