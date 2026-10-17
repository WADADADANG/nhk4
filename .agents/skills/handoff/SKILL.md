---
name: handoff
description: Compact the current conversation into a handoff document for another agent or a fresh chat session to seamlessly pick up.
argument-hint: "What will the next session be used for?"
disable-model-invocation: true
---

# Handoff Workflow

When the user requests a `handoff` or wants to prepare for a new chat session:

1. **Write a Handoff Document** summarizing the state of the conversation and codebase.
2. **Save Location**:
   - Save to `.agents/handoff_latest.md` in the current workspace.

3. **Required Sections in Handoff Document**:
   - **Executive Summary**: High-level overview of the session goals.
   - **Key Achievements & Solved Issues**: What was fixed, modified, or added in this session.
   - **Critical Architecture Context**: Files modified, important line numbers, design decisions (e.g. Rust Native Core Engine, Axum API, 29 nodes).
   - **Active Files & Status**: Which files are currently in progress or recently edited.
   - **Next Steps / Pending Tasks**: Clear, bulleted action items for the next agent to immediately tackle.
   - **Instructions for Fresh Agent**: Prompt template for the user to provide in the new chat.

4. **Response to User**:
   - Confirm the handoff document path.
   - Give the user the exact one-liner to paste into the new chat:
     `อ่านไฟล์ .agents/handoff_latest.md แล้วพร้อมลุยงานต่อ`
