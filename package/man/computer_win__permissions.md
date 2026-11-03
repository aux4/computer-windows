#### Description

Reports what this process can reach. Windows has no permission prompts for UI Automation or screen capture; what limits an agent is:

- **desktop** — an interactive, unlocked desktop. From a Windows service, a non-interactive scheduled task or an SSH session there is none, and every UI command fails with a clear error.
- **elevated** — whether this process runs as administrator. A non-elevated process can't send input or UI Automation actions into an elevated app (UIPI); commands on such an app fail with `"uipi": true` and explain how to fix it.
- **uiAccess** — the signed-binary flag that also allows driving elevated apps; normally off.

Each entry has `granted` and `guidance`. `AUX4_COMPUTER_DENY_PERMISSIONS=1` reports everything as not granted (for testing).

#### Usage

```bash
aux4 computer win permissions
```

#### Example

```bash
aux4 computer win permissions
```

```json
{"desktop":{"granted":true,"guidance":"An interactive, unlocked desktop is available: UI Automation, input and screen capture can reach apps."},"elevated":{"granted":false,"integrity":"medium","guidance":"This process is not elevated: Windows (UIPI) blocks UI Automation actions and input into apps that run as administrator; normal apps work. To drive an elevated app, run aux4 from an elevated terminal (Run as administrator)."},"uiAccess":{"granted":false,"guidance":"uiAccess is not set (it needs a signed binary installed under Program Files); only needed to drive elevated apps from a non-elevated process."}}
```
