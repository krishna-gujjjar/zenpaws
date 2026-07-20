# 01 - Project Overview

## What is ZenPaws

ZenPaws is a desktop application for LAN-only, serverless group chat with an
interactive desktop-pet layer on top. It's aimed at small trusted networks -
an office, a LAN party, a home network - where installing a chat server isn't
worth it, but people still want presence, chat, file sharing, and a bit of
personality on their desktop.

## The three systems

1. **Messenger** - shared-room and direct-message chat, search, file and
   image sharing.
2. **Desktop Pet Platform** - transparent, always-on-top animated companions
   that react to chat activity and the user's cursor.
3. **LAN Networking Platform** - peer discovery, hostless replication,
   messaging transport, and file transfer that the other two systems sit on.

These are loosely coupled through an event bus (`25_EVENT_BUS_ARCHITECTURE.md`)
so the pet platform keeps working even if chat is disabled.

## Target users

Small trusted LANs, up to roughly 50 concurrent peers. No system
administrator, no server to stand up - every participant just runs the app.

## Supported platforms

Windows, Linux, macOS - one codebase, via Tauri v2.

## Target hardware

Dual-core Intel i3-equivalent CPU, 4GB RAM, HDD, integrated GPU. See
`13_PERFORMANCE_GUIDELINES.md` for the concrete budgets this implies.

## What this document set is

Each numbered doc in `docs/` owns one concern. When in doubt about a
decision, its rationale lives in exactly one of these files - check
`04_SYSTEM_ARCHITECTURE.md` if unsure which one.
