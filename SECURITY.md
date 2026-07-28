# Security Policy

## Reporting a vulnerability

Do not disclose a suspected vulnerability in a public issue.

Use GitHub private vulnerability reporting for `dornglut/runen-spatial` when available. Otherwise contact the repository owner privately through the contact method on the owner's GitHub profile. Include the affected revision, impact, reproduction conditions, and any proposed mitigation.

## Supported versions

RunenSpatial is private, unpublished, and pre-release. Security fixes are applied to the default branch and to exact revisions explicitly consumed by downstream projects. No broader support window is promised until a release policy is accepted.

## Scope

Relevant reports include memory-safety defects, validation bypasses, denial-of-service behavior from untrusted inputs or backend events, identity/correlation failures, arithmetic overflow that changes spatial identity, dependency compromise, and violations of documented deterministic or containment guarantees.
