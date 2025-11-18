# Project Sovereign Core (V2.1)

## Consolidated Handoff Prompt (Final Version)

We are beginning the creation of "Project Sovereign," a self-hosted digital ecosystem using only **commodity cloud services** (Cloudflare, AWS S3) and focusing entirely on **user freedom, simplicity, and low-cost maintenance**. Our goal is to make it easy for an individual to deploy their own digital presence.

We are specifically targeting services that currently trap users into proprietary single sign-on (SSO) systems (Google, Facebook, Amazon, Apple) and hosted social media. We are **not interested** in recreating existing, well-tested open-source tools like full email servers.

The current focus on the single-user ActivityPub/Bluesky server is **Step One**—a sample implementation to validate the architecture. The **long-term goal** is to brainstorm and prioritize other simple, cheap, and easily deployable services that enhance personal digital sovereignty.

---

## 1. Architectural Mandate & Stack

* **Goal:** A CLI-first, single-user system with "no-installer" portable binaries. We are **actively not interested in trying to make things scale**.
* **Core Logic (Appliance):** C#/.NET 9+ (using NativeAOT for main CLI, Server, and UI).
* **Security Layer (Integrity):** Rust (via FFI) for low-level cryptography only.
* **Extension Layer (Flexibility):** Python for high-level scripting, demos, and automation.

---

## 2. Python Isolation Mandate (CRITICAL)

Python scripts must **ALWAYS** be developed and executed within **isolated environments** (e.g., venv, pipx). Dependencies must be minimized and limited to stable, popular libraries (e.g., requests, click). Python must not populate the host system.

---

## 3. Required Functionality & Interactions

* **Identity Lifecycle:** The C# CLI must include `sovereign identity setup <domain>` and `cleanup` commands that automate DNS records (A, TXT) via the Cloudflare API.
* **Protocol Implementation:** AT Protocol uses C# libraries. ActivityPub **MUST** call the Rust bridge for all cryptographic signing operations.

---

## 4. Explicit Exclusions (DO NOT Implement)

* Do not use **Python for the core server/CLI/UI logic.**
* Do not use **Kubernetes, Docker Swarm, or complex scaling/load-balancing logic.**
* Do not attempt to implement **custom low-level crypto or signatures** in C# or Python; delegate this to the Rust bridge.
* Do not include **SMTP/Email hosting.**

---

## 5. Future Trajectory (The Next Steps)

Once the core server is stable, the project's focus will shift to **brainstorming and prioritizing** future components that are **simple to build, cheap and easy to host**, and align with personal interest and user freedom (e.g., decentralized contacts/calendar, private photo archive, lightweight personal search).

---

## Current Status

**We have the design complete, but no code has been generated yet.**

### Next Decision Point

Based on this comprehensive plan, the next step is to decide:

1. **Proceed with code generation** - Generate detailed component outlines and code structure
2. **Expand design further** - Detail additional requirements (e.g., file sync protocols)
