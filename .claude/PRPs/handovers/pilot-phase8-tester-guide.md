# Brehon Governance Pilot — Tester Guide

**Pilot server:** `https://lemmy.agentgrey.app` (public, HTTPS — works from any device including Voyager)
**LAN fallback:** `http://192.168.1.157:1236` (home network) or `http://100.81.145.58:1236` (Tailscale)
**Status:** closed pilot — registration requires admin approval

---

## 1. Getting access

The site requires admin approval to register.

**Browser:** go to `https://lemmy.agentgrey.app`
**Voyager app (iOS/Android):** open Voyager → tap the instance field → type `lemmy.agentgrey.app` → Sign Up

1. Click / tap **Sign Up**
3. Fill in: username (pick something), password, and the **application answer** field — type anything (e.g. "household pilot tester")
4. Submit — you'll be in a pending state
5. **Tell the admin** (Barry) you've registered — they'll approve you via the admin panel
6. Once approved, log in with your username and password

---

## 2. What to try: normal user flow

Once logged in:

- **Subscribe to the community:** find "Test Governance Community" in the sidebar or at `/c/test_governance`. Subscribe.
- **Post something:** click Create Post in the community. Title + optional body/link. Submit.
- **Comment** on others' posts.
- **Report a post you find objectionable:** use the three-dot menu on any post → Report. Pick a reason. This is how governance starts — a reported post opens a moderation case.

---

## 3. What happens when you report something (governance flow)

When a post is reported:

1. The admin reviews and **opens a formal case** (`POST /governance/report` — admin step, you don't need to do this)
2. The admin **assigns a jury** from the eligible pool — 5 jurors selected, excluding the reporter and the post author
3. Jurors **vote** on a decision: remove content, advisory label, no action, etc.
4. When enough votes align (quorum), the case is **decided** and a sanction is applied automatically

From a tester's seat: you mostly just post, browse, and report. The admin drives the jury assignment + voting cycle during the pilot using seeded juror accounts.

---

## 4. What the admin sees and does

**Admin account:** `lemmy` / `lemmylemmy`

**Approve registrations:**
```
GET  /api/v4/admin/registration_application/list?unread_only=true   (check queue)
PUT  /api/v4/admin/registration_application/approve  {id: N, approve: true}
```
Or use the Lemmy UI: Admin panel → Registration Applications.

**Open a case from a report:**
```
POST /api/v4/governance/report
  {reporter_id, target_type: "post", target_id: <post_id>, community_id: 2, reason_code: "spam"}
```

**Assign a jury:**
```
POST /api/v4/governance/admin/assign-jury  {case_id: N}
```

**Drive juror voting** (using seeded juror accounts 1–10, password `testpass123`):
```
POST /api/v4/governance/jury/accept  {case_id: N}   # as juror
POST /api/v4/governance/jury/vote    {case_id: N, decision: "remove_content", rationale: "..."}
```

Decisions available: `remove_content`, `no_action`, `advisory_label`, `warning`, `cooldown`, `suspend_community_member`, `suspend_local_user`

---

## 5. What NOT to test (pilot scope limits)

- **Matrix/Element client** — `matrix.agentgrey.app` is exposed but juror rooms require an invite from the admin. Ask Barry if you want Element access to a juror room.
- **Federation** — federation is enabled at the DB level but no other instance is federated. Posts stay local.
- **Real sanctions on real content** — the pilot is for exercising the governance flow, not for actually sanctioning anyone. Use throwaway test posts.
- **Password reset / email** — email is not wired. Forget your password → tell the admin to reset it via the API.

---

## 6. Rate limits

Current pilot limits (generous — raised for testing):
- Post/vote/governance: 500 requests / 10 min
- Register/login: 500 / 10 min
- Messages: 2000 / 1 min

These are much higher than a real Lemmy instance — intentional for the pilot so testing doesn't hit rate gates.

---

## 7. Reporting issues

Drop a message to Barry. Note:
- The URL you were on
- What you did
- What happened vs what you expected

---

## Admin reference: API quick-start

```bash
# Login
curl -s -X POST https://lemmy.agentgrey.app/api/v4/account/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"username_or_email":"lemmy","password":"lemmylemmy"}'
# → {"jwt": "eyJ..."}

# Approve a registration application (replace JWT and app ID)
curl -s -X PUT https://lemmy.agentgrey.app/api/v4/admin/registration_application/approve \
  -H 'Content-Type: application/json' \
  -H 'Authorization: Bearer <JWT>' \
  -d '{"id": <app_id>, "approve": true}'

# List pending applications
curl -s 'https://lemmy.agentgrey.app/api/v4/admin/registration_application/list?unread_only=true' \
  -H 'Authorization: Bearer <JWT>'
```
