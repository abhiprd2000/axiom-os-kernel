# Security Policy

## Overview
Axiom OS is a research prototype focused on proportional-cost per-read provenance and file integrity. While security is a first-class citizen in the design of the provenance layer, the kernel is currently in an **alpha/research state**. 

We welcome security research, vulnerability reports, and feedback that help strengthen the architecture.

## Supported Versions

Currently, security updates and patches are only provided for the latest development branch.

| Version | Supported          |
| ------- | ------------------ |
| v0.3.0  | :white_check_mark: |
| < v0.3.0| :x:                |

## Reporting a Vulnerability

**Please do not report security vulnerabilities through public GitHub issues.** 

If you discover a security vulnerability, please report it through one of the following methods:

1. **Email:** 8292aniarc@gmail.com
2. **GitHub Private Vulnerability Reporting:** Please use the "Report a vulnerability" button under the **Security and Quality** tab.

### What to Include
To help us triage the report quickly, please include:
- A description of the vulnerability and its potential impact.
- Steps to reproduce the issue (ideally within the QEMU environment).
- Any proof-of-concept (PoC) code or shell commands (e.g., using the `tamper` command to show a bypass).

## Our Disclosure Process
Once a report is received:
1. We will acknowledge receipt of your report within 48-72 hours.
2. We will investigate the issue and confirm the vulnerability.
3. If confirmed, we will work on a fix in a private branch.
4. We will coordinate a disclosure date with you before making the fix public.

## Threat Model & Scope
Please refer to the **Threat Model** section in the `README.md` for the current security assumptions. 

### In-Scope for Research:
- Bypassing the BLAKE3 block-level integrity verification.
- Escalating privileges from the Shell/User mode to Kernel mode.
- Breaking the CR3-based process isolation.
- Vulnerabilities in the Mitra DSL execution path.

### Out-of-Scope:
- Hardware-level attacks (e.g., Rowhammer) that are explicitly excluded from the current threat model.
- Denial of Service (DoS) attacks that simply crash the kernel (since it is a single-core research prototype).
- Issues requiring physical access or DMA-capable peripherals.

## Acknowledgments
We value the work of independent security researchers. If you report a valid vulnerability that leads to a fix, we are happy to credit you.