# Testing BioFocus (v0.1.0) — guide for outside testers

Thanks for trying BioFocus! This takes about **30–45 minutes**. You need a Mac
(Apple Silicon or Intel, macOS 13+). An iPhone is optional.

BioFocus is a **personal, local-first** self-tracking app. Everything stays on
your Mac: there is no account, no cloud and no telemetry. The numbers are
**estimates for self-reflection**, not medical or performance judgements.

---

## 1. Install & run

You'll get either a built `BioFocus.app` or the source code.

**A. Built app**: drag `BioFocus.app` into Applications and open it. If macOS
says it can't verify the developer, right-click the app → **Open** → **Open**.

**B. From source**:

```bash
# Prerequisites: Xcode Command Line Tools, Rust (rustup), Node.js ≥ 20, pnpm
xcode-select --install          # if needed
cd apps/desktop
pnpm install
pnpm tauri dev                  # first build takes a few minutes
```

BioFocus lives in the **menu bar** (top-right of the screen). Click its icon to
open the panel; use **Open Dashboard** for charts and metrics.

### Optional data sources (all off by default)

Set them before starting (`export NAME=1`, then `pnpm tauri dev`), or ask us for a
build that has them turned on:

| Variable | What it adds | Permission it may ask for |
|---|---|---|
| `BIOFOCUS_INPUT_AGGREGATES=1` | Typing rate (counts only, never keys) | Accessibility |
| `BIOFOCUS_GIT_ACTIVITY=1` | Commits/checkouts in folders you add under **Git folders** | none |
| `BIOFOCUS_CALENDAR=1` + `BIOFOCUS_CALENDAR_ICS=/path/file.ics` | Meeting busy time (no titles) | none |
| `BIOFOCUS_NOTIFICATION_EVENTS=1` | Notification counts (never content) | Full Disk Access |
| `BIOFOCUS_BROWSER_CATEGORIES=1` | Coarse site categories (no URLs) | Automation |

Without any of these you still get **app switching** from the active window.

---

## 2. What to try

Leave BioFocus running and work normally for **at least 20 minutes**. The
metrics use 15-minute windows, so nothing shows up in the first few minutes.

1. **First launch**: does the menu-bar panel say *Ready / Running locally*? Is
   anything confusing?
2. **Dashboard → Snapshot**: click a metric to expand it. Does the explanation
   make sense? Do values marked **rough** or **one source** look less trustworthy
   to you, as intended?
3. **Focus vs. switching**: spend ~15 min in one app (writing or reading), then ~15 min
   hopping between many apps. **Focus** should drop and **App switches** /
   **Combined demand** should rise. Neither should be stuck at exactly 0 or 100.
4. **Quiet reading**: read a long document without typing for 15 min. Focus
   should **not** collapse just because you aren't typing.
5. **Charts**: switch the chart range (1h / 8h / 1d …). Anything empty or odd?
6. **Insights / Suggestions**: do they cite evidence and feel calm (not preachy)?
7. **Life events**: log one (e.g. a walk) and check it appears under *Recent*.
8. **Report**: click **Generate report**. A local AI section only appears if a
   local model is configured. Without one, the offline report should still work.
9. **Quit & reopen**: is your data still there?

---

## 3. Pair an iPhone (optional)

The iPhone **Companion** app sends Apple Health data (heart rate, HRV, steps,
sleep) to your Mac **over your local Wi‑Fi only**. Getting the Companion onto
your phone currently means building it with Xcode: open
`apps/companion/ios/BioFocusCompanion.xcodeproj`, choose your iPhone, set your
Team under *Signing & Capabilities*, then press ⌘R.

1. On the Mac, open the menu-bar panel → **Companion**.
2. Tick **Enable LAN ingest for physical iPhone**.
3. **Quit and reopen BioFocus.** The card should then say **Ready to pair**.
   - *LAN is off — this Mac only* → the box isn't ticked yet.
   - *Restart BioFocus to apply* → you ticked it but haven't restarted.
   - *LAN is on, but no network address was found* → check that Wi‑Fi is
     connected, then press **Reload**.
   - *Phone sync is not running* → the message says why (e.g. port in use).
4. Make sure the iPhone is on the **same Wi‑Fi** (guest or office networks that
   isolate devices won't work).
5. In the Companion app, enter the **Base URL** (`http://192.168.x.x:8787`) and the
   **token** (use **Copy**, or **Show QR** to scan it). Allow **Local Network** and
   **Health** access when iOS asks.
6. Tap **Test connection**, then sync. Within a few minutes, *Recovery* /
   *Stress index* should appear in the Dashboard.

If it can't connect, allow incoming connections for BioFocus in
*System Settings → Network → Firewall*, then try again.

**Security notes:** the token works like a password, so don't share it. Phone
sync uses plain HTTP on your local network. Use it on a home network you trust,
not on public Wi‑Fi.

---

## 4. How to report issues

Please report each problem separately, with:

- **What you did** (steps), **what you expected**, **what happened**
- A **screenshot** of the panel or Dashboard
- macOS version, Mac model, and (if relevant) iPhone/iOS version
- Which optional data sources you turned on
- Roughly what time it happened (so we can match the logs)

Send reports to **the email or chat the BioFocus team gave you**, or open a
GitHub issue on the project repository if you have access.

**Never include** your pairing token or anything from `~/.biofocus/`
(`pairing_token`, `llm.env`, the database). Screenshots with the token
**hidden** (the default) are fine.

Feedback we especially want:

- Which metric explanations were unclear or felt wrong?
- Did any score feel stuck, jumpy or unbelievable?
- Was anything in the text worrying or judgmental?
- Would you keep using this? Why or why not?

---

## 5. Resetting / uninstalling

Quit BioFocus from the menu bar. Delete the app. To erase **all** local data,
delete the folder `~/.biofocus` (this also removes the pairing token, so the phone
will need to pair again).
